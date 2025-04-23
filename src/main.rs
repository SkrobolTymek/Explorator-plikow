use eframe::{
    egui::{self, CentralPanel, Context, Grid, ScrollArea, TextEdit, Ui, Visuals, Widget},
    run_native, App, CreationContext, NativeOptions,
};
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};
use chrono::{DateTime, Local};
use dirs;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use human_sort::compare;

static LAST_PATH: Lazy<Mutex<PathBuf>> = Lazy::new(|| {
    Mutex::new(dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")))
});

#[derive(Debug, Clone)]
struct FileItem {
    path: PathBuf,
    name: String,
    is_dir: bool,
    size: u64,
    modified: SystemTime,
}

impl FileItem {
    fn from_entry(entry: DirEntry) -> Option<Self> {
        let path = entry.path();
        let metadata = entry.metadata().ok()?;
        let name = path.file_name()?.to_string_lossy().into_owned();

        Some(Self {
            path,
            name,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().ok()?,
        })
    }
}

struct FileExplorer {
    current_path: PathBuf,
    items: Vec<FileItem>,
    search_query: String,
    search_results: Vec<FileItem>,
    searching: bool,
    show_hidden: bool,
    sort_by: SortBy,
    sort_ascending: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SortBy {
    Name,
    Size,
    Modified,
    Type,
}

impl Default for FileExplorer {
    fn default() -> Self {
        let current_path = LAST_PATH.lock().clone();
        let mut explorer = Self {
            current_path,
            items: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            searching: false,
            show_hidden: false,
            sort_by: SortBy::Name,
            sort_ascending: true,
        };
        explorer.load_items();
        explorer
    }
}

impl FileExplorer {
    fn open_file(&self, path: &Path) {
        #[cfg(target_os = "windows")]
        Command::new("cmd").arg("/C").arg("start").arg(path).spawn().ok();

        #[cfg(target_os = "linux")]
        Command::new("xdg-open").arg(path).spawn().ok();

        #[cfg(target_os = "macos")]
        Command::new("open").arg(path).spawn().ok();
    }

    fn load_items(&mut self) {
        *LAST_PATH.lock() = self.current_path.clone();
        self.items.clear();

        if let Ok(entries) = fs::read_dir(&self.current_path) {
            self.items = entries
                .filter_map(|e| e.ok())
                .filter(|e| self.show_hidden || !e.file_name().to_string_lossy().starts_with('.'))
                .filter_map(FileItem::from_entry)
                .collect();

            self.sort_items();
        }
    }

    fn sort_items(&mut self) {
        match self.sort_by {
            SortBy::Name => {
                self.items.sort_by(|a, b| {
                    let order = compare(&a.name, &b.name);
                    if self.sort_ascending {
                        order
                    } else {
                        order.reverse()
                    }
                });
            },
            SortBy::Size => self.items.sort_by(|a, b| a.size.cmp(&b.size)),
            SortBy::Modified => self.items.sort_by(|a, b| a.modified.cmp(&b.modified)),
            SortBy::Type => self.items.sort_by(|a, b| {
                let a_ext = a.path.extension().map(|s| s.to_string_lossy());
                let b_ext = b.path.extension().map(|s| s.to_string_lossy());
                a_ext.cmp(&b_ext)
            }),
        }
    }

    fn search_files(&mut self) {
        self.search_results.clear();
        if self.search_query.trim().is_empty() {
            self.searching = false;
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.current_path) {
            self.search_results = entries
                .filter_map(|e| e.ok())
                .filter_map(FileItem::from_entry)
                .filter(|item| {
                    item.name
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                })
                .collect();
        }

        self.searching = true;
    }

    fn render_path_navigator(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut path_str = self.current_path.to_string_lossy().to_string();
            if ui.text_edit_singleline(&mut path_str).changed() {
                if let Ok(path) = PathBuf::from(path_str).canonicalize() {
                    self.current_path = path;
                    self.load_items();
                }
            }

            if ui.button("↻").on_hover_text("Refresh").clicked() {
                self.load_items();
            }
        });
    }

    fn render_search_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search...")
                    .desired_width(ui.available_width() - 100.0),
            );

            if ui.button("🔍 Search").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search_files();
            }
        });
    }

    fn render_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("⬆ Up").clicked() {
                if let Some(parent) = self.current_path.parent() {
                    self.current_path = parent.to_path_buf();
                    self.load_items();
                }
            }

            ui.checkbox(&mut self.show_hidden, "Show hidden");
            
            ui.menu_button("Sort By", |ui| {
                if ui.selectable_label(self.sort_by == SortBy::Name, "Name").clicked() {
                    self.sort_by = SortBy::Name;
                    self.sort_items();
                }
                if ui.selectable_label(self.sort_by == SortBy::Size, "Size").clicked() {
                    self.sort_by = SortBy::Size;
                    self.sort_items();
                }
                if ui.selectable_label(self.sort_by == SortBy::Modified, "Modified").clicked() {
                    self.sort_by = SortBy::Modified;
                    self.sort_items();
                }
                if ui.selectable_label(self.sort_by == SortBy::Type, "Type").clicked() {
                    self.sort_by = SortBy::Type;
                    self.sort_items();
                }
                ui.separator();
                if ui.selectable_label(self.sort_ascending, "Ascending").clicked() {
                    self.sort_ascending = true;
                    self.sort_items();
                }
                if ui.selectable_label(!self.sort_ascending, "Descending").clicked() {
                    self.sort_ascending = false;
                    self.sort_items();
                }
            });
        });
    }

    fn render_file_table(&mut self, ui: &mut Ui) {
        Grid::new("file_grid")
            .striped(true)
            .min_col_width(100.0)
            .show(ui, |ui| {
                ui.label("Name");
                ui.label("Size");
                ui.label("Modified");
                ui.label("Actions");
                ui.end_row();

                let items = if self.searching {
                    self.search_results.clone()
                } else {
                    self.items.clone()
                };

                for item in items {
                    let icon = if item.is_dir { "📁" } else { "📄" };
                    ui.label(format!("{} {}", icon, item.name));

                    ui.label(if item.is_dir {
                        "--".to_string()
                    } else {
                        humansize::format_size(item.size, humansize::DECIMAL)
                    });

                    let modified: DateTime<Local> = item.modified.into();
                    ui.label(modified.format("%Y-%m-%d %H:%M").to_string());

                    if ui.button("Open").clicked() {
                        if item.is_dir {
                            self.current_path = item.path.clone();
                            self.load_items();
                        } else {
                            self.open_file(&item.path);
                        }
                    }

                    ui.end_row();
                }
            });
    }
}

impl App for FileExplorer {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());
        
        CentralPanel::default().show(ctx, |ui| {
            self.render_path_navigator(ui);
            self.render_search_bar(ui);
            self.render_toolbar(ui);
            
            ScrollArea::vertical().show(ui, |ui| {
                if self.searching && self.search_results.is_empty() {
                    ui.label("No results found");
                } else {
                    self.render_file_table(ui);
                }
            });
        });
    }

    fn on_close_event(&mut self) -> bool {
        *LAST_PATH.lock() = self.current_path.clone();
        true
    }
}

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    run_native(
        "Modern File Explorer",
        options,
        Box::new(|_cc| Box::new(FileExplorer::default())),
    )
}
use dirs;
use eframe::{
    egui::{self, CentralPanel, ScrollArea, TextEdit, Visuals},
    run_native, App, NativeOptions,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct FileExplorer {
    current_path: PathBuf,
    items: Vec<PathBuf>,
    search_query: String,
    search_results: Vec<PathBuf>,
    searching: bool,
}

impl Default for FileExplorer {
    fn default() -> Self {
        let mut explorer = Self {
            current_path: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            items: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            searching: false,
        };
        explorer.load_items();
        explorer
    }
}

impl FileExplorer {
    fn open_file(&self, path: &PathBuf) {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd").arg("/C").arg("start").arg(path).spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open").arg(path).spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open").arg(path).spawn();
        }
    }

    fn load_items(&mut self) {
        self.items.clear();
        if let Ok(entries) = fs::read_dir(&self.current_path) {
            for entry in entries.flatten() {
                self.items.push(entry.path());
            }
        }
    }

    fn search_files(path: &PathBuf, query: &str, results: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    
                    if entry_path.read_dir().is_ok() {
                        FileExplorer::search_files(&entry_path, query, results);
                    }
                } else if entry_path
                    .file_name()
                    .map_or(false, |name| name.to_string_lossy().contains(query))
                {
                    results.push(entry_path);
                }
            }
        }
    }

    fn perform_search(&mut self) {
        self.search_results.clear(); 
        if self.search_query.trim().is_empty() {
            self.searching = false;
            return;
        }

        let mut results = Vec::new();
        FileExplorer::search_files(&self.current_path, &self.search_query, &mut results);
        self.search_results = results;
        self.searching = true;
    }
}

impl App for FileExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());
        CentralPanel::default().show(ctx, |ui| {
            ui.label(format!(
                "Current Directory: {}",
                self.current_path.display()
            ));
            ui.horizontal(|ui| {
                ui.add(TextEdit::singleline(&mut self.search_query).hint_text("Search..."));
                if ui.button("Search").clicked() {
                    self.perform_search();
                }
            });

            ScrollArea::vertical().show(ui, |ui| {
                if self.searching {
                    ui.label("Search Results:");
                    if self.search_results.is_empty() {
                        ui.label("Searching...");
                    } else {
                        for result in &self.search_results {
                            let name = result.file_name().unwrap_or_default().to_string_lossy();
                            ui.label(format!("📄 {}", name));
                            if ui.button("Open").clicked() {
                                self.open_file(result);
                            }
                        }
                    }
                } else {
                    self.load_items();
                    let items: Vec<PathBuf> = self.items.clone();
                    for item in items.iter() {
                        let name = item.file_name().unwrap_or_default().to_string_lossy();
                        if item.is_dir() {
                            if ui.button(format!("📁 {}", name)).clicked() {
                                self.current_path = item.clone();
                                self.load_items();
                            }
                        } else {
                            ui.label(format!("📄 {}", name));
                            if ui.button("Open").clicked() {
                                self.open_file(item);
                            }
                        }
                    }
                }
                if self.current_path.parent().is_some() {
                    if ui.button("⬆ Back").clicked() {
                        if let Some(parent) = self.current_path.parent() {
                            self.current_path = parent.to_path_buf();
                            self.load_items();
                        }
                    }
                }
            });
        });

        ctx.request_repaint(); 
    }
}

fn main() -> eframe::Result<()> {
    let win_option = NativeOptions::default();
    run_native(
        "Blazingly Fast File Explorer",
        win_option,
        Box::new(|_cc| Ok(Box::new(FileExplorer::default()))),
    )
}

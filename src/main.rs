use eframe::{egui::{self, CentralPanel, ScrollArea, Visuals}, run_native, NativeOptions, App};
use std::fs;
use std::path::{PathBuf, Path};

struct Folders {
    current_path: PathBuf,
    files_and_folders: Vec<PathBuf>,
    query: String,
}

impl Default for Folders {
    fn default() -> Self {
        Self {
            current_path: PathBuf::from("C:\\"), 
            files_and_folders: Vec::new(),
            query: String::new(),
        }
    }
}

impl Folders {
    fn load_files_and_folders(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.current_path) {
            self.files_and_folders.clear();
            for entry in entries.flatten() {
                let path = entry.path();
                self.files_and_folders.push(path);
            }
        }
    }

    fn search_files(&mut self) {
        let query = self.query.trim().to_lowercase();
        if query.is_empty() {
            self.load_files_and_folders();
        } else {
            self.files_and_folders.retain(|path| {
                path.to_string_lossy().to_lowercase().contains(&query)
            });
        }
    }
}

impl App for Folders {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());

        CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("Current Directory: {}", self.current_path.display()));

            ui.horizontal(|ui| {
                ui.label("Search: ");
                ui.text_edit_singleline(&mut self.query);
                if ui.button("Search").clicked() {
                    self.search_files(); 
                }
            });

            ScrollArea::vertical().show(ui, |ui| {
                self.load_files_and_folders(); 

                // Wersja bez konfliktu borrowingu
                let files_and_folders_clone = self.files_and_folders.clone(); 
                for path in files_and_folders_clone {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if path.is_dir() {
                        if ui.button(format!("📁 {}", name)).clicked() {
                            self.current_path = path.clone();
                            self.load_files_and_folders(); 
                        }
                    } else {
                        if ui.button(format!("📄 {}", name)).clicked() {
                            
                        }
                    }
                }

                if self.current_path.parent().is_some() {
                    if ui.button("Back").clicked() {
                        if let Some(parent) = self.current_path.parent() {
                            self.current_path = parent.to_path_buf();
                            self.load_files_and_folders(); 
                        }
                    }
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let win_option = NativeOptions::default();

    run_native(
        "Blazingly Fast File Explorer",
        win_option,
        Box::new(|_cc| Ok(Box::new(Folders::default()))),
    )
}

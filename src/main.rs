use eframe::{egui::{self, CentralPanel, ScrollArea, Visuals}, run_native, NativeOptions, App};
use std::fs;
use std::path::PathBuf;

struct Folders {
    current_path: PathBuf,
    subfolders: Vec<PathBuf>,
}

impl Default for Folders {
    fn default() -> Self {
        Self {
            current_path: PathBuf::from("C:\\"),
            subfolders: Vec::new(),
        }
    }
}

impl App for Folders {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());

        CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("Current Directory: {}", self.current_path.display()));

            ScrollArea::vertical().show(ui, |ui| {
                if let Ok(entries) = fs::read_dir(&self.current_path) {
                    self.subfolders.clear();

                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            self.subfolders.push(path);
                        }
                    }
                }

                for folder in &self.subfolders {
                    let folder_name = folder.file_name().unwrap_or_default().to_string_lossy();
                    if ui.button(folder_name).clicked() {
                        self.current_path = folder.clone();
                    }
                }
                if self.current_path.parent().is_some() {
                    if ui.button("Back").clicked() {
                        if let Some(parent) = self.current_path.parent() {
                            self.current_path = parent.to_path_buf();
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

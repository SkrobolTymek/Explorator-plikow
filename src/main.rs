use eframe::{egui::CentralPanel, egui , epi::App, run_native, NativeOptions};
use std::fs;

struct Folders;

impl App for Folders {
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>){
        CentralPanel::default().show(ctx, |ui| {
            let paths = fs::read_dir("C:\\").unwrap();
            
            for path in paths{ 
                let path = path.unwrap().path();
                let path = path.display();
                ui.add(egui::Label::new(path));
            }
        });
    }
    fn name(&self) -> &str {
        "Blazingly Fast File Explorer"
    }
}



fn main() {
    let app = Folders;
    let win_option = NativeOptions::default();
   
    run_native(Box::new(app), win_option);
}
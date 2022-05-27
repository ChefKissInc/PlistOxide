#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod serialise;
mod value;

fn main() {
    eframe::run_native(
        "PlistOxide",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app::App::new())),
    )
}

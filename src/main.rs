#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod serialise;
mod value_type;

fn main() {
    eframe::run_native(
        "Xplist",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app::Xplist::new())),
    )
}

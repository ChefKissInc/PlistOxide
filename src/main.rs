#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::App;
use eframe::{run_native, NativeOptions};

mod app;
mod widgets;

fn main() {
    run_native(
        "PlistOxide",
        NativeOptions::default(),
        Box::new(|_cc| Box::new(App::new())),
    )
}

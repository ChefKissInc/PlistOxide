#![deny(
    warnings,
    clippy::cargo,
    clippy::nursery,
    unused_extern_crates,
    rust_2021_compatibility
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use app::App;
use clap::Parser;
use eframe::{run_native, IconData, NativeOptions};

mod app;
mod widgets;

static ICON: &[u8; 95126] = include_bytes!("app_icon/icon512x512@2x.png");

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    run_native(
        "com.ChefKissInc.PlistOxide",
        NativeOptions {
            #[cfg(target_os = "macos")]
            fullsize_content: true,
            icon_data: Some(IconData {
                rgba: ICON.to_vec(),
                width: 1024,
                height: 1024,
            }),
            drag_and_drop_support: true,
            ..Default::default()
        },
        Box::new(|_cc| Box::new(App::new(cli.path))),
    );
}

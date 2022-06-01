#![deny(warnings, clippy::cargo, unused_extern_crates, rust_2021_compatibility)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(let_chains)]

use app::App;
use eframe::{run_native, IconData, NativeOptions};

mod app;
mod widgets;

static ICON: &[u8; 95126] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/AppIcon.xcassets/icon512x512@2x.png"
));

fn main() {
    let mut args = std::env::args();
    let file = if args.len() > 1 {
        let path = std::path::PathBuf::from(args.nth(1).unwrap());
        if (path.exists() && path.is_file()) || path.ends_with(".plist") {
            Some(path)
        } else {
            None
        }
    } else {
        None
    };

    run_native(
        "com.ChefKissInc.PlistOxide",
        NativeOptions {
            icon_data: Some(IconData {
                rgba: ICON.to_vec(),
                width: 1024,
                height: 1024,
            }),
            drag_and_drop_support: true,
            ..Default::default()
        },
        Box::new(|_cc| Box::new(App::new(file))),
    )
}

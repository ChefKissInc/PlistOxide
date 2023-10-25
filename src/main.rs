//!  Copyright © 2022-2023 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//!  See LICENSE for details.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![cfg_attr(target_os = "macos", feature(sync_unsafe_cell))]

use std::path::PathBuf;

use app::PlistOxide;
use eframe::{IconData, NativeOptions};

mod app;
mod style;
mod utils;
mod widgets;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

fn main() {
    eframe::run_native(
        "Untitled.plist",
        NativeOptions {
            icon_data: Some(IconData {
                rgba: include_bytes!("app_icon/icon512x512@2x.png").to_vec(),
                width: 1024,
                height: 1024,
            }),
            #[cfg(target_os = "macos")]
            fullsize_content: true,
            drag_and_drop_support: true,
            app_id: Some("com.ChefKissInc.PlistOxide".into()),
            ..Default::default()
        },
        Box::new(|cc| {
            Box::new(PlistOxide::new(
                cc,
                std::env::args().nth(1).map(PathBuf::from),
            ))
        }),
    )
    .unwrap();
}

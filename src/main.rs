//! Copyright © 2022-2024 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//! See LICENSE for details.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::multiple_crate_versions)] // Nothing that can be done to fix this.
#![cfg_attr(target_os = "macos", feature(sync_unsafe_cell))]

use std::path::PathBuf;

use app::PlistOxide;
use eframe::NativeOptions;
use egui::ViewportBuilder;

mod app;
mod style;
mod utils;
mod widgets;

fn main() {
    eframe::run_native(
        "PlistOxide",
        NativeOptions {
            viewport: ViewportBuilder::default()
                .with_icon(
                    eframe::icon_data::from_png_bytes(include_bytes!(
                        "app_icon/icon512x512@2x.png"
                    ))
                    .unwrap(),
                )
                .with_fullsize_content_view(true)
                .with_titlebar_shown(false)
                .with_app_id("com.ChefKissInc.PlistOxide"),
            ..Default::default()
        },
        Box::new(|cc| {
            Ok(Box::new(PlistOxide::new(
                cc,
                std::env::args().nth(1).map(PathBuf::from),
            )))
        }),
    )
    .unwrap();
}

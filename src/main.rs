//! Copyright © 2022-2025 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
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

fn run_native(renderer: eframe::Renderer) -> eframe::Result {
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
                .with_app_id("org.ChefKiss.PlistOxide"),
            renderer,
            ..Default::default()
        },
        Box::new(|cc| {
            Ok(Box::new(PlistOxide::new(
                cc,
                std::env::args().nth(1).map(PathBuf::from),
            )))
        }),
    )
}

fn main() {
    if let Err(e) = run_native(eframe::Renderer::Wgpu) {
        eprintln!("Failed to run with wgpu renderer, trying glow. ({e})");
    }
    if let Err(e) = run_native(eframe::Renderer::Glow) {
        eprintln!("Failed to run with wgpu and glow renderer: ({e})");
    }
}

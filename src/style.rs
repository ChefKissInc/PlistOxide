//! Copyright © 2022-2024 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//! See LICENSE for details.

use egui::{FontData, FontDefinitions, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};

fn get_system_font(family: FamilyName) -> Option<Vec<u8>> {
    match SystemSource::new()
        .select_best_match(&[family], &Properties::new())
        .ok()?
    {
        Handle::Path {
            path,
            font_index: _,
        } => std::fs::read(path).ok(),
        Handle::Memory {
            bytes,
            font_index: _,
        } => Some(bytes.to_vec()),
    }
}

pub fn get_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    if let Some(sans_serif) = get_system_font(FamilyName::SansSerif) {
        fonts.font_data.insert(
            "System Sans Serif".into(),
            FontData::from_owned(sans_serif).into(),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "System Sans Serif".into());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "System Sans Serif".into());
    }
    if let Some(monospace) = get_system_font(FamilyName::Monospace) {
        fonts.font_data.insert(
            "System Monospace".into(),
            FontData::from_owned(monospace).into(),
        );
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "System Monospace".into());
    }

    fonts
}

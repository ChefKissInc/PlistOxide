//!  Copyright © 2022-2023 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//!  See LICENSE for details.

use egui::{FontData, FontDefinitions, FontFamily};

pub fn get_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Helvetica".into(),
        FontData::from_static(include_bytes!("../assets/fonts/Helvetica.ttf")),
    );
    fonts.font_data.insert(
        "JetBrainsMonoNerdFont".into(),
        FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMonoNerdFont.ttf")),
    );
    fonts.font_data.insert(
        "Symbol".into(),
        FontData::from_static(include_bytes!("../assets/fonts/Symbol.ttf")),
    );
    fonts.font_data.insert(
        "Apple Symbols".into(),
        FontData::from_static(include_bytes!("../assets/fonts/Apple Symbols.ttf")),
    );

    let ent = fonts.families.entry(FontFamily::Proportional).or_default();
    ent.insert(0, "Helvetica".into());
    ent.insert(1, "Symbol".into());
    ent.insert(2, "Apple Symbols".into());

    let ent = fonts.families.entry(FontFamily::Monospace).or_default();
    ent.insert(0, "JetBrainsMonoNerdFont".into());
    ent.insert(1, "Symbol".into());
    ent.insert(2, "Apple Symbols".into());
    ent.push("Helvetica".into());

    fonts
}

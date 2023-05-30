use egui::{FontData, FontDefinitions, FontFamily};

pub fn get_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Helvetica".into(),
        FontData::from_static(include_bytes!("../assets/fonts/Helvetica.ttf")),
    );
    fonts.font_data.insert(
        "Iosevka NF".into(),
        FontData::from_static(include_bytes!("../assets/fonts/Iosevka NF.ttf")),
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
    ent.insert(0, "Iosevka NF".into());
    ent.insert(1, "Symbol".into());
    ent.insert(2, "Apple Symbols".into());
    ent.push("Helvetica".into());

    fonts
}

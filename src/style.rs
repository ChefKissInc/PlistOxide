use egui::{FontData, FontDefinitions, FontFamily};

pub fn get_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Helvetica".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Helvetica.ttf")),
    );
    fonts.font_data.insert(
        "Iosevka NF".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Iosevka NF.ttf")),
    );
    fonts.font_data.insert(
        "Symbol".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Symbol.ttf")),
    );
    fonts.font_data.insert(
        "Apple Symbols".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Apple Symbols.ttf")),
    );

    let ent = fonts.families.entry(FontFamily::Proportional).or_default();
    ent.insert(0, "Helvetica".to_owned());
    ent.insert(1, "Symbol".to_owned());
    ent.insert(2, "Apple Symbols".to_owned());

    let ent = fonts.families.entry(FontFamily::Monospace).or_default();
    ent.insert(0, "Iosevka NF".to_owned());
    ent.insert(1, "Symbol".to_owned());
    ent.insert(2, "Apple Symbols".to_owned());
    ent.push("Helvetica".to_owned());

    fonts
}

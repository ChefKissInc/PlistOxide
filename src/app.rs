use std::path::PathBuf;

use egui::{Key, Modifiers, ScrollArea};
use plist::Value;

pub struct App {
    path: Option<PathBuf>,
    root: Value,
}

impl App {
    pub fn new() -> Self {
        Self {
            path: None,
            root: Value::Dictionary(plist::Dictionary::default()),
        }
    }

    fn open(&mut self) {
        self.path = rfd::FileDialog::new()
            .add_filter("Property List", &["plist"])
            .pick_file();

        if let Some(path) = &self.path {
            self.root = plist::from_file(path).unwrap();
        }
    }

    fn save(&mut self) {
        if let Some(path) = &self.path {
            plist::to_file_xml(path, &self.root).unwrap();
        } else {
            self.path = rfd::FileDialog::new()
                .add_filter("Property List", &["plist"])
                .save_file();

            if let Some(path) = &self.path {
                plist::to_file_xml(path, &self.root).unwrap();
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open();
                        ui.close_menu();
                    }

                    if ui.button("Save").clicked() {
                        self.save();
                        ui.close_menu();
                    }
                });
            });
        });

        if ctx.input_mut().consume_key(Modifiers::COMMAND, Key::O) {
            self.open();
        }

        if ctx.input_mut().consume_key(Modifiers::COMMAND, Key::S) {
            self.save();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(path) = &self.path {
                frame.set_window_title(path.to_str().unwrap());
            } else {
                frame.set_window_title("Untitled.plist");
            }

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    crate::serialise::serialise(ui, "Root", &mut self.root);
                });
        });
    }
}

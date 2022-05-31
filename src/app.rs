use std::{collections::HashMap, path::PathBuf};

use egui::{Id, Key, Modifiers, ScrollArea};
use either::Either::Right;
use plist::Value;

use crate::widgets::value::render_value;

#[derive(Default)]
pub struct State {
    pub data_store: HashMap<Id, Option<String>>,
    pub auto_id: u64,
}

impl State {
    pub fn get_next_id(&mut self) -> u64 {
        let id = self.auto_id;
        self.auto_id = self.auto_id.wrapping_add(1);
        id
    }
}

pub struct App {
    path: Option<PathBuf>,
    root: Value,
    state: State,
}

impl App {
    pub fn new() -> Self {
        Self {
            path: None,
            root: Value::Dictionary(plist::Dictionary::default()),
            state: State::default(),
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

            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.state.auto_id = 0;
                    render_value(&mut self.state, ui, "Root", &mut Right(&mut self.root));
                });
        });
    }
}

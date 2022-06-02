use std::{collections::HashMap, path::PathBuf};

use egui::{Align2, Id, Key, Modifiers, ScrollArea};
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
    open: bool,
    root: Value,
    state: State,
    error: Option<plist::Error>,
}

impl App {
    pub fn new(path: Option<PathBuf>) -> Self {
        let open = path.is_some();
        Self {
            path,
            open,
            root: Value::Dictionary(plist::Dictionary::default()),
            state: State::default(),
            error: None,
        }
    }

    fn handle_error(&mut self, action: &str, ctx: &egui::Context) {
        if self.error.is_some() {
            egui::Window::new(format!("🗙 Error while {} plist", action))
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0., 0.])
                .show(ctx, move |ui| {
                    ui.label(self.error.as_ref().unwrap().to_string());
                    if ui.button("Ok").clicked() {
                        self.error = None;
                    }
                });
        }
    }

    fn open(&mut self) {
        self.path = rfd::FileDialog::new().pick_file();

        if self.path.is_some() {
            self.open = true;
        }
    }

    fn save(&mut self, ctx: &egui::Context) {
        if let Some(path) = &self.path {
            plist::to_file_xml(path, &self.root).unwrap();
        } else {
            self.path = rfd::FileDialog::new().save_file();

            if let Some(path) = &self.path {
                if let Err(e) = plist::to_file_xml(path, &self.root) {
                    self.error = Some(e);
                    self.handle_error("saving", ctx);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.open {
            if let Some(path) = &self.path {
                if path.exists() && path.is_file() {
                    self.root = match plist::from_file(path) {
                        Ok(v) => {
                            self.error = None;
                            v
                        }
                        Err(e) => {
                            self.error = Some(e);
                            Value::Dictionary(plist::Dictionary::default())
                        }
                    };
                }
                self.open = false;
            }
        }

        self.handle_error("opening", ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open();
                        ui.close_menu();
                    }

                    if ui.button("Save").clicked() {
                        self.save(ctx);
                        ui.close_menu();
                    }
                });
            });
        });

        if ctx.input_mut().consume_key(Modifiers::COMMAND, Key::O) {
            self.open();
        }

        if ctx.input_mut().consume_key(Modifiers::COMMAND, Key::S) {
            self.save(ctx);
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
                    render_value(&mut self.state, ui, "Root", &mut self.root, true, false);
                });
        });
    }
}

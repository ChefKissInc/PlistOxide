use std::{collections::HashMap, path::PathBuf, sync::Once};

use egui::{style::Margin, Align2, Frame, Id, Key, Modifiers, ScrollArea};
use plist::Value;
use serde::{Deserialize, Serialize};

use crate::widgets::value::render_value;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    #[serde(skip)]
    pub data_store: HashMap<Id, Option<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlistOxideApp {
    path: Option<PathBuf>,
    root: Value,
    state: State,
    #[serde(skip, default = "Once::new")]
    pub open_file: Once,
    #[serde(skip)]
    error: Option<plist::Error>,
}

impl PlistOxideApp {
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        let v = Self {
            path,
            root: Value::Dictionary(plist::Dictionary::default()),
            state: State::default(),
            open_file: Once::new(),
            error: None,
        };
        cc.egui_ctx.set_fonts(crate::style::get_fonts());
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or(v)
        } else {
            v
        }
    }

    fn handle_error(&mut self, action: &str, ctx: &egui::Context) {
        if self.error.is_some() {
            egui::Window::new(format!("\u{1F5D9} Error while {action} plist"))
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

    fn open_file(&mut self) {
        self.path = rfd::FileDialog::new().pick_file();

        if self.path.is_some() {
            self.open_file = Once::new();
        }
    }

    fn save_file(&mut self, ctx: &egui::Context) {
        self.path = self
            .path
            .clone()
            .or_else(|| rfd::FileDialog::new().save_file());

        if let Some(path) = &self.path {
            if let Err(e) = plist::to_file_xml(path, &self.root) {
                self.error = Some(e);
                self.handle_error("saving", ctx);
            }
        }
    }
}

impl eframe::App for PlistOxideApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.open_file.call_once(|| {
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
            }
        });

        self.handle_error("opening", ctx);

        #[cfg(not(target_os = "macos"))]
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.set_min_height(25.0);

            ui.centered_and_justified(|ui| {
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
        });

        if ctx.input_mut().consume_key(Modifiers::COMMAND, Key::O) {
            self.open_file();
        }

        if ctx.input_mut().consume_key(Modifiers::COMMAND, Key::S) {
            self.save_file(ctx);
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
                    #[cfg(target_os = "macos")]
                    ui.add_space(12.5);

                    Frame::none().inner_margin(Margin::same(5.)).show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            render_value(&mut self.state, ui, "Root", &mut self.root, true, false);
                        })
                    })
                });
        });
    }
}

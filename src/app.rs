//  Copyright © 2022-2023 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for
//  details.

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
        cc.egui_ctx.set_fonts(crate::style::get_fonts());
        cc.storage
            .and_then(|v| eframe::get_value(v, eframe::APP_KEY))
            .unwrap_or(Self {
                path,
                root: Value::Dictionary(plist::Dictionary::default()),
                state: State::default(),
                open_file: Once::new(),
                error: None,
            })
    }

    fn handle_error(&mut self, action: &str, ctx: &egui::Context) {
        let Some(error) = self.error.as_ref().map(|v| v.to_string()) else {
            return;
        };
        egui::Window::new(format!("\u{1F5D9} Error while {action} plist"))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, move |ui| {
                ui.label(error);
                if ui.button("Ok").clicked() {
                    self.error = None;
                }
            });
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

        let Some(path) = &self.path else {
            return;
        };
        self.error = plist::to_file_xml(path, &self.root).err();
        self.handle_error("saving", ctx);
    }
}

impl eframe::App for PlistOxideApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.open_file.call_once(|| {
            let Some(path) = &self.path else {
                return;
            };
            if !path.exists() || !path.is_file() {
                return;
            }
            self.root = match plist::from_file(path) {
                Ok(v) => {
                    frame.set_window_title(&format!(
                        "{} - PlistOxide",
                        self.path
                            .as_ref()
                            .and_then(|v| v.to_str())
                            .unwrap_or("Untitled.plist"),
                    ));
                    self.error = None;
                    v
                }
                Err(e) => {
                    self.error = Some(e);
                    Value::Dictionary(plist::Dictionary::default())
                }
            };
        });

        self.handle_error("opening", ctx);

        #[cfg(not(target_os = "macos"))]
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.set_min_height(25.0);

            ui.centered_and_justified(|ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            self.open_file();
                            ui.close_menu();
                        }

                        if ui.button("Save").clicked() {
                            self.save_file(ctx);
                            ui.close_menu();
                        }
                    });
                });
            });
        });

        if ctx.input_mut(|v| v.consume_key(Modifiers::COMMAND, Key::O)) {
            self.open_file();
        }

        if ctx.input_mut(|v| v.consume_key(Modifiers::COMMAND, Key::S)) {
            self.save_file(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Frame::none()
                        .inner_margin(Margin::same(5.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                render_value(
                                    &mut self.state,
                                    ui,
                                    "Root",
                                    &mut self.root,
                                    true,
                                    false,
                                );
                            })
                        })
                });
        });
    }
}

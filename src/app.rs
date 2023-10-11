//!  Copyright © 2022-2023 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//!  See LICENSE for details.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, Once},
};

use egui::{Align, Key, KeyboardShortcut, Layout, Modifiers};
use egui_extras::{Column, TableBuilder};
use plist::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlistOxide {
    path: Option<PathBuf>,
    root: Arc<Mutex<Value>>,
    #[serde(skip, default = "Once::new")]
    pub open_file: Once,
    #[serde(skip)]
    error: Option<plist::Error>,
}

impl PlistOxide {
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        cc.egui_ctx.set_fonts(crate::style::get_fonts());
        cc.storage
            .and_then(|v| eframe::get_value(v, eframe::APP_KEY))
            .unwrap_or(Self {
                path,
                root: Mutex::new(Value::Dictionary(plist::Dictionary::default())).into(),
                open_file: Once::new(),
                error: None,
            })
    }

    fn handle_error(&mut self, action: &str, ctx: &egui::Context) {
        let Some(error) = self.error.as_ref().map(std::string::ToString::to_string) else {
            return;
        };
        let dialog = egui_modal::Modal::new(ctx, "Modal");
        dialog.show(|ui| {
            dialog.title(ui, format!("Error while {action} plist"));
            dialog.frame(ui, |ui| {
                dialog.body_and_icon(ui, error, egui_modal::Icon::Error);
            });
            dialog.buttons(ui, |ui| {
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    if dialog.button(ui, "Okay").clicked() {
                        self.error = None;
                        self.path = None;
                    }
                });
            });
        });
        dialog.open();
    }

    fn open_file(&mut self) {
        self.path = rfd::FileDialog::new().pick_file();

        if self.path.is_some() {
            self.open_file = Once::new();
        }
    }

    fn save_file(&mut self, ctx: &egui::Context) {
        self.path = self.path.clone().or_else(|| {
            rfd::FileDialog::new()
                .set_file_name("Untitled.plist")
                .save_file()
        });

        let Some(path) = &self.path else {
            return;
        };
        self.error = plist::to_file_xml(path, &self.root).err();
        self.handle_error("saving", ctx);
    }
}

impl eframe::App for PlistOxide {
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
                    Mutex::new(Value::Dictionary(plist::Dictionary::default())).into()
                }
            };
        });

        self.handle_error("opening", ctx);

        let open_shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
        let save_shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

        #[cfg(not(target_os = "macos"))]
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.set_min_height(25.0);

            ui.centered_and_justified(|ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui
                            .add(
                                egui::Button::new("Open")
                                    .shortcut_text(ui.ctx().format_shortcut(&open_shortcut)),
                            )
                            .clicked()
                        {
                            self.open_file();
                            ui.close_menu();
                        }

                        if ui
                            .add(
                                egui::Button::new("Save")
                                    .shortcut_text(ui.ctx().format_shortcut(&save_shortcut)),
                            )
                            .clicked()
                        {
                            self.save_file(ctx);
                            ui.close_menu();
                        }
                    });
                });
            });
        });

        if ctx.input_mut(|v| v.consume_shortcut(&open_shortcut)) {
            self.open_file();
        }

        if ctx.input_mut(|v| v.consume_shortcut(&save_shortcut)) {
            self.save_file(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::remainder())
                .column(Column::auto())
                .column(Column::remainder())
                .auto_shrink([false, false])
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Key");
                    });
                    header.col(|ui| {
                        ui.strong("Type");
                    });
                    header.col(|ui| {
                        ui.strong("Value");
                    });
                })
                .body(|mut body| {
                    crate::widgets::entry::PlistEntry::new(Arc::clone(&self.root), vec![])
                        .show(&mut body);
                });
        });
    }
}

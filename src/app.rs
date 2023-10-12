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

#[derive(Serialize, Deserialize)]
pub struct PlistOxide {
    path: Option<PathBuf>,
    root: Arc<Mutex<Value>>,
    #[serde(skip, default = "Once::new")]
    open_file: Once,
    #[serde(skip)]
    error: Option<plist::Error>,
    unsaved: bool,
    #[serde(skip)]
    dialogue: Option<egui_modal::Modal>,
    closing: bool,
    can_close: bool,
    #[serde(skip)]
    egui_ctx: egui::Context,
    title: String,
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
                unsaved: false,
                dialogue: Some(egui_modal::Modal::new(&cc.egui_ctx, "Modal")),
                can_close: false,
                closing: false,
                egui_ctx: cc.egui_ctx.clone(),
                title: "Untitled.plist".into(),
            })
    }

    fn handle_error(&mut self, action: &str) {
        let Some(error) = self.error.as_ref().map(std::string::ToString::to_string) else {
            return;
        };
        let dialogue = self.dialogue.as_mut().unwrap();
        dialogue.show(|ui| {
            dialogue.title(ui, format!("Error while {action} plist"));
            dialogue.frame(ui, |ui| {
                dialogue.body_and_icon(ui, error, egui_modal::Icon::Error);
            });
            dialogue.buttons(ui, |ui| {
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    if dialogue.button(ui, "Okay").clicked() {
                        self.error = None;
                        self.path = None;
                    }
                });
            });
        });
        dialogue.open();
    }

    fn open_file(&mut self) {
        self.path = rfd::FileDialog::new().pick_file();

        if self.path.is_some() {
            self.open_file = Once::new();
        }
    }

    fn update_title(&mut self, frame: &mut eframe::Frame) {
        self.title = format!(
            "{}{}",
            self.path
                .as_ref()
                .and_then(|v| v.to_str())
                .unwrap_or("Untitled.plist"),
            if self.unsaved { " *" } else { "" }
        );
        frame.set_window_title(&self.title);
    }

    fn save_file(&mut self, frame: &mut eframe::Frame) {
        self.path = self.path.clone().or_else(|| {
            rfd::FileDialog::new()
                .set_file_name("Untitled.plist")
                .save_file()
        });

        let Some(path) = &self.path else {
            return;
        };
        self.error = plist::to_file_xml(path, &self.root).err();
        self.unsaved = self.error.is_some();
        self.handle_error("saving");
        self.update_title(frame);
    }
}

impl eframe::App for PlistOxide {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn on_close_event(&mut self) -> bool {
        if self.unsaved && !self.can_close {
            self.closing = true;
            self.egui_ctx.request_repaint();
            return false;
        }
        true
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.closing {
            let dialogue = self.dialogue.as_mut().unwrap();
            dialogue.show(|ui| {
                dialogue.title(ui, "Are you sure you want to exit?");
                dialogue.frame(ui, |ui| {
                    dialogue.body_and_icon(
                        ui,
                        "You have unsaved changes",
                        egui_modal::Icon::Warning,
                    );
                });
                dialogue.buttons(ui, |ui| {
                    if dialogue.caution_button(ui, "Yes").clicked() {
                        self.can_close = true;
                        frame.close();
                    }
                    if dialogue.button(ui, "No").clicked() {
                        self.closing = false;
                    }
                });
            });
            dialogue.open();
        }
        let mut new_title = None;
        self.open_file.call_once(|| {
            let Some(path) = &self.path else {
                return;
            };
            if !path.exists() || !path.is_file() {
                return;
            }
            self.root = match plist::from_file(path) {
                Ok(v) => {
                    let title: String = self
                        .path
                        .as_ref()
                        .and_then(|v| v.to_str())
                        .unwrap_or("Untitled.plist *")
                        .into();
                    frame.set_window_title(&title);
                    new_title = Some(title);
                    self.error = None;
                    v
                }
                Err(e) => {
                    self.error = Some(e);
                    Mutex::new(Value::Dictionary(plist::Dictionary::default())).into()
                }
            };
        });
        if let Some(new_title) = new_title {
            self.title = new_title;
        }

        self.handle_error("opening");

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
                            self.save_file(frame);
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
            self.save_file(frame);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(target_os = "macos")]
            ui.add_sized((ui.available_width(), 14.0), egui::Label::new(&self.title));

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
                    let changed =
                        crate::widgets::entry::PlistEntry::new(Arc::clone(&self.root), vec![])
                            .show(&mut body);
                    self.unsaved |= changed.is_some();
                    self.update_title(frame);
                });
        });
    }
}

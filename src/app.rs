//! Copyright © 2022-2025 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//! See LICENSE for details.

use egui::ViewportCommand;
use egui_extras::{Column, TableBuilder};
#[cfg(target_os = "macos")]
use objc2::{MainThreadOnly, define_class, msg_send, rc::Retained, sel};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
#[cfg(target_os = "macos")]
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, ns_string};
use plist::{Dictionary, Value};
use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use std::{cell::SyncUnsafeCell, mem::MaybeUninit};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, Once},
};

#[derive(Serialize, Deserialize)]
pub struct PersistentState {
    path: Option<PathBuf>,
    root: Arc<Mutex<Value>>,
    unsaved: bool,
}

impl PersistentState {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            root: Arc::new(Mutex::new(Value::Dictionary(Dictionary::new()))),
            unsaved: false,
        }
    }
}

pub struct PlistOxide {
    state: PersistentState,
    open_file: Once,
    error: Option<String>,
    closing: bool,
    can_close: bool,
    #[cfg(target_os = "macos")]
    _menu: Retained<PlistOxideMenu>,
}

#[cfg(target_os = "macos")]
static EGUI_CTX: SyncUnsafeCell<MaybeUninit<egui::Context>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());

#[cfg(target_os = "macos")]
static OPENING_FILE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[cfg(target_os = "macos")]
static SAVING_FILE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[cfg(target_os = "macos")]
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct PlistOxideMenu;

    unsafe impl NSObjectProtocol for PlistOxideMenu {}

    impl PlistOxideMenu {
        #[unsafe(method(openingFile))]
        fn opening_file(&self) {
            *OPENING_FILE.lock().unwrap() = true;
            unsafe { (*EGUI_CTX.get()).assume_init_mut().request_repaint() };
        }

        #[unsafe(method(savingFile))]
        fn saving_file(&self) {
            *SAVING_FILE.lock().unwrap() = true;
            unsafe { (*EGUI_CTX.get()).assume_init_mut().request_repaint() };
        }
    }
);

impl PlistOxide {
    #[cfg(target_os = "macos")]
    fn opening_file_false() {
        *OPENING_FILE.lock().unwrap() = false;
    }

    #[cfg(target_os = "macos")]
    fn saving_file_false() {
        *SAVING_FILE.lock().unwrap() = false;
    }

    #[cfg(target_os = "macos")]
    fn new_global_menu(cc: &eframe::CreationContext<'_>) -> Retained<PlistOxideMenu> {
        unsafe { (*EGUI_CTX.get()).write(cc.egui_ctx.clone()) };
        let mtm = MainThreadMarker::new().unwrap();
        let file_menu = unsafe { NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!("File")) };

        let menu: Retained<PlistOxideMenu> = unsafe { msg_send![PlistOxideMenu::alloc(mtm), init] };

        let file_open = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                ns_string!("Open..."),
                Some(sel!(openingFile)),
                ns_string!("o"),
            )
        };
        unsafe { file_open.setTarget(Some(&menu)) };
        file_menu.addItem(&file_open);

        file_menu.addItem(&NSMenuItem::separatorItem(mtm));

        let file_save = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                ns_string!("Save..."),
                Some(sel!(savingFile)),
                ns_string!("s"),
            )
        };
        unsafe { file_save.setTarget(Some(&menu)) };
        file_menu.addItem(&file_save);

        let file_item = NSMenuItem::new(mtm);
        file_item.setSubmenu(Some(&file_menu));
        unsafe {
            NSApplication::sharedApplication(mtm)
                .mainMenu()
                .unwrap()
                .addItem(&file_item)
        };
        menu
    }

    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        #[cfg(target_os = "macos")]
        let menu = Self::new_global_menu(cc);
        cc.egui_ctx.set_fonts(crate::style::get_fonts());
        let state = cc
            .storage
            .and_then(|v| eframe::get_value(v, eframe::APP_KEY))
            .unwrap_or_else(|| PersistentState::new(path));
        Self {
            state,
            open_file: Once::new(),
            error: None,
            can_close: false,
            closing: false,
            #[cfg(target_os = "macos")]
            _menu: menu,
        }
    }

    fn handle_error(&mut self, ctx: &egui::Context, action: &str) {
        let Some(error) = self.error.as_ref() else {
            return;
        };
        let mut error_acked = false;
        egui::Modal::new(egui::Id::new("ErrorModal")).show(ctx, |ui| {
            ui.heading(format!("Error while {action} plist"));
            ui.separator();
            ui.label(error);
            ui.separator();
            egui::Sides::new().show(
                ui,
                |_| {},
                |ui| {
                    error_acked = ui.button("Okay").clicked();
                },
            )
        });
        if error_acked {
            self.error = None;
            self.state.path = None;
        }
    }

    fn open_file(&mut self) {
        self.state.path = rfd::FileDialog::new().pick_file();

        if self.state.path.is_some() {
            self.open_file = Once::new();
        }
    }

    fn update_title(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Title(format!(
            "{}{}",
            self.state
                .path
                .as_ref()
                .and_then(|v| v.to_str())
                .unwrap_or("Untitled.plist"),
            if self.state.unsaved { " *" } else { "" }
        )));
    }

    fn save_file(&mut self, ctx: &egui::Context) {
        self.state.path = self.state.path.clone().or_else(|| {
            rfd::FileDialog::new()
                .set_file_name("Untitled.plist")
                .save_file()
        });

        let Some(path) = &self.state.path else {
            return;
        };
        self.error = plist::to_file_xml(path, &self.state.root)
            .err()
            .map(|v| v.to_string());
        self.state.unsaved = self.error.is_some();
        self.handle_error(ctx, "saving");
        self.update_title(ctx);
    }
}

impl eframe::App for PlistOxide {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && self.state.unsaved && !self.can_close {
            self.closing = true;
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }

        if self.closing {
            egui::Modal::new(egui::Id::new("ExitUnsaved")).show(ctx, |ui| {
                ui.heading("Are you sure you want to exit?");
                ui.separator();
                ui.label("You have unsaved changes");
                ui.separator();
                egui::Sides::new().show(
                    ui,
                    |_| {},
                    |ui| {
                        if ui.button("Yes").clicked() {
                            self.can_close = true;
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                        if ui.button("No").clicked() {
                            self.closing = false;
                        }
                    },
                );
            });
        }

        self.open_file.call_once(|| {
            let Some(path) = &self.state.path else {
                return;
            };
            if !path.exists() || !path.is_file() {
                return;
            }
            self.state.root = match plist::from_file(path) {
                Ok(v) => {
                    ctx.send_viewport_cmd(ViewportCommand::Title(
                        self.state
                            .path
                            .as_ref()
                            .and_then(|v| v.to_str())
                            .unwrap_or("Untitled.plist *")
                            .into(),
                    ));
                    self.error = None;
                    v
                }
                Err(e) => {
                    self.error = Some(e.to_string());
                    Mutex::new(Value::Dictionary(plist::Dictionary::default())).into()
                }
            };
        });

        self.handle_error(ctx, "opening");

        #[cfg(not(target_os = "macos"))]
        let open_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::O);
        #[cfg(not(target_os = "macos"))]
        let save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);

        #[cfg(not(target_os = "macos"))]
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.set_min_height(25.0);

            ui.centered_and_justified(|ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui
                            .add(
                                egui::Button::new("Open")
                                    .shortcut_text(ui.ctx().format_shortcut(&open_shortcut)),
                            )
                            .clicked()
                        {
                            self.open_file();
                            ui.close();
                        }

                        if ui
                            .add(
                                egui::Button::new("Save")
                                    .shortcut_text(ui.ctx().format_shortcut(&save_shortcut)),
                            )
                            .clicked()
                        {
                            self.save_file(ctx);
                            ui.close();
                        }
                    });
                });
            });
        });

        #[cfg(target_os = "macos")]
        let opening = *OPENING_FILE.lock().unwrap();
        #[cfg(not(target_os = "macos"))]
        let opening = ctx.input_mut(|v| v.consume_shortcut(&open_shortcut));
        if opening {
            self.open_file();
            #[cfg(target_os = "macos")]
            Self::opening_file_false();
        }

        #[cfg(target_os = "macos")]
        let saving = *SAVING_FILE.lock().unwrap();
        #[cfg(not(target_os = "macos"))]
        let saving = ctx.input_mut(|v| v.consume_shortcut(&save_shortcut));
        if saving {
            self.save_file(ctx);
            #[cfg(target_os = "macos")]
            Self::saving_file_false();
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
                    let state = crate::widgets::entry::PlistEntry::new(
                        Arc::clone(&self.state.root),
                        vec![],
                    )
                    .show(&mut body);
                    self.state.unsaved |= state != crate::widgets::entry::ChangeState::Unchanged;
                    self.update_title(ctx);
                });
        });
    }
}

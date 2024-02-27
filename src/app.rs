//!  Copyright © 2022-2024 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//!  See LICENSE for details.

use egui::{Align, Layout, ViewportCommand};
use egui_extras::{Column, TableBuilder};
#[cfg(target_os = "macos")]
use icrate::{
    objc2::{
        declare_class, msg_send_id, mutability::MainThreadOnly, rc::Id, sel, ClassType,
        DeclaredClass,
    },
    AppKit::{NSApplication, NSMenu, NSMenuItem},
    Foundation::{ns_string, MainThreadMarker, NSObject, NSObjectProtocol},
};
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
    error: Option<plist::Error>,
    dialogue: Option<egui_modal::Modal>,
    closing: bool,
    can_close: bool,
    #[cfg(target_os = "macos")]
    _menu: Id<PlistOxideMenu>,
}

#[cfg(target_os = "macos")]
static EGUI_CTX: SyncUnsafeCell<MaybeUninit<egui::Context>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());

#[cfg(target_os = "macos")]
static OPENING_FILE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[cfg(target_os = "macos")]
static SAVING_FILE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[cfg(target_os = "macos")]
declare_class!(
    struct PlistOxideMenu;

    unsafe impl ClassType for PlistOxideMenu {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "PlistOxideMenu";
    }

    impl DeclaredClass for PlistOxideMenu {}

    unsafe impl NSObjectProtocol for PlistOxideMenu {}

    unsafe impl PlistOxideMenu {
        #[method(openingFile)]
        unsafe fn opening_file(&self) {
            *OPENING_FILE.lock().unwrap() = true;
            (*EGUI_CTX.get()).assume_init_mut().request_repaint();
        }

        #[method(savingFile)]
        unsafe fn saving_file(&self) {
            *SAVING_FILE.lock().unwrap() = true;
            (*EGUI_CTX.get()).assume_init_mut().request_repaint();
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
    unsafe fn new_global_menu(cc: &eframe::CreationContext<'_>) -> Id<PlistOxideMenu> {
        (*EGUI_CTX.get()).write(cc.egui_ctx.clone());
        let mtm = MainThreadMarker::new().unwrap();
        let file_menu = NSMenu::initWithTitle(mtm.alloc(), ns_string!("File"));

        let menu: Id<PlistOxideMenu> = unsafe { msg_send_id![mtm.alloc(), init] };

        let file_open = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("Open..."),
            Some(sel!(openingFile)),
            ns_string!("o"),
        );
        file_open.setTarget(Some(&menu));
        file_menu.addItem(&file_open);

        file_menu.addItem(&NSMenuItem::separatorItem(mtm));

        let file_save = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("Save..."),
            Some(sel!(savingFile)),
            ns_string!("s"),
        );
        file_save.setTarget(Some(&menu));
        file_menu.addItem(&file_save);

        let file_item = NSMenuItem::new(mtm);
        file_item.setSubmenu(Some(&file_menu));
        NSApplication::sharedApplication(mtm)
            .mainMenu()
            .unwrap()
            .addItem(&file_item);
        menu
    }

    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        #[cfg(target_os = "macos")]
        let menu = unsafe { Self::new_global_menu(cc) };
        cc.egui_ctx.set_fonts(crate::style::get_fonts());
        let state = cc
            .storage
            .and_then(|v| eframe::get_value(v, eframe::APP_KEY))
            .unwrap_or_else(|| PersistentState::new(path));
        Self {
            state,
            open_file: Once::new(),
            error: None,
            dialogue: Some(egui_modal::Modal::new(&cc.egui_ctx, "Modal")),
            can_close: false,
            closing: false,
            #[cfg(target_os = "macos")]
            _menu: menu,
        }
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
                        self.state.path = None;
                    }
                });
            });
        });
        dialogue.open();
    }

    fn open_file(&mut self) {
        self.state.path = rfd::FileDialog::new().pick_file();

        if self.state.path.is_some() {
            self.open_file = Once::new();
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
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
        self.error = plist::to_file_xml(path, &self.state.root).err();
        self.state.unsaved = self.error.is_some();
        self.handle_error("saving");
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
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                    if dialogue.button(ui, "No").clicked() {
                        self.closing = false;
                    }
                });
            });
            dialogue.open();
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
                    self.error = Some(e);
                    Mutex::new(Value::Dictionary(plist::Dictionary::default())).into()
                }
            };
        });

        self.handle_error("opening");

        #[cfg(not(target_os = "macos"))]
        let open_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::O);
        #[cfg(not(target_os = "macos"))]
        let save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);

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
            #[cfg(target_os = "macos")]
            ui.add_space(18.0);

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
                    let changed = crate::widgets::entry::PlistEntry::new(
                        Arc::clone(&self.state.root),
                        vec![],
                    )
                    .show(&mut body);
                    self.state.unsaved |= changed.is_some();
                    self.update_title(ctx);
                });
        });
    }
}

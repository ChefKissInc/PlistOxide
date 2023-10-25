//!  Copyright © 2022-2023 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//!  See LICENSE for details.

use egui::{Align, Layout};
use egui_extras::{Column, TableBuilder};
#[cfg(target_os = "macos")]
use objc::{
    declare::ClassDecl,
    runtime::{Object, Sel},
};
#[cfg(target_os = "macos")]
use objc_foundation::{INSString, NSString};
#[cfg(target_os = "macos")]
use objc_id::Id;
use plist::Value;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use std::{cell::SyncUnsafeCell, mem::MaybeUninit};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, Once},
};

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

#[cfg(target_os = "macos")]
static EGUI_CTX: SyncUnsafeCell<MaybeUninit<egui::Context>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());

#[cfg(target_os = "macos")]
static OPENING_FILE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[cfg(target_os = "macos")]
static SAVING_FILE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[cfg(target_os = "macos")]
extern "C" fn opening_file(_: &mut Object, _: Sel) {
    unsafe {
        *OPENING_FILE.lock().unwrap() = true;
        (*EGUI_CTX.get()).assume_init_mut().request_repaint();
    }
}

#[cfg(target_os = "macos")]
extern "C" fn saving_file(_: &mut Object, _: Sel) {
    unsafe {
        *SAVING_FILE.lock().unwrap() = true;
        (*EGUI_CTX.get()).assume_init_mut().request_repaint();
    }
}

impl PlistOxide {
    #[cfg(target_os = "macos")]
    unsafe fn new_menu_target(opening_file_sel: Sel, saving_file_sel: Sel) -> *mut Object {
        let mut decl = ClassDecl::new("POxideNSMenuTarget", class!(NSObject)).unwrap();
        decl.add_method(
            opening_file_sel,
            opening_file as extern "C" fn(&mut Object, Sel),
        );
        decl.add_method(
            saving_file_sel,
            saving_file as extern "C" fn(&mut Object, Sel),
        );
        let cls = decl.register();
        let target: *mut Object = msg_send![cls, alloc];
        msg_send![target, init]
    }

    #[cfg(target_os = "macos")]
    unsafe fn new_menu(title: &str) -> Id<Object> {
        let v: *mut Object = msg_send![class!(NSMenu), alloc];
        msg_send![v, initWithTitle: NSString::from_str(title)]
    }

    #[cfg(target_os = "macos")]
    unsafe fn new_submenu_item(title: &str, action: Sel, key_equivalent: &str) -> Id<Object> {
        let v: *mut Object = msg_send![class!(NSMenuItem), alloc];
        msg_send![v, initWithTitle: NSString::from_str(title) action: action keyEquivalent: NSString::from_str(key_equivalent)]
    }

    #[cfg(target_os = "macos")]
    unsafe fn new_submenu_separator() -> Id<Object> {
        msg_send![class!(NSMenuItem), separatorItem]
    }

    #[cfg(target_os = "macos")]
    fn opening_file_false() {
        *OPENING_FILE.lock().unwrap() = false;
    }

    #[cfg(target_os = "macos")]
    fn saving_file_false() {
        *SAVING_FILE.lock().unwrap() = false;
    }

    #[cfg(target_os = "macos")]
    unsafe fn init_global_menu(cc: &eframe::CreationContext<'_>) {
        (*EGUI_CTX.get()).write(cc.egui_ctx.clone());
        let file_menu = Self::new_menu("File");

        let opening_file_sel = sel!(openingFile);
        let saving_file_sel = sel!(savingFile);
        let target = Self::new_menu_target(opening_file_sel, saving_file_sel);

        let file_open = Self::new_submenu_item("Open...", opening_file_sel, "o");
        let _: () = msg_send![file_open, setTarget: &*target];
        let _: () = msg_send![file_menu, addItem: file_open];

        let _: () = msg_send![file_menu, addItem: Self::new_submenu_separator()];

        let file_save = Self::new_submenu_item("Save...", saving_file_sel, "s");
        let _: () = msg_send![file_save, setTarget: &*target];
        let _: () = msg_send![file_menu, addItem: file_save];

        let file_item: Id<Object> = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![file_item, setSubmenu: file_menu];
        let app: Id<Object> = msg_send![class!(NSApplication), sharedApplication];
        let main_menu: Id<Object> = msg_send![app, mainMenu];
        let _: () = msg_send![main_menu, addItem: file_item];
    }

    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        #[cfg(target_os = "macos")]
        unsafe {
            Self::init_global_menu(cc);
        }
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
                            self.save_file(frame);
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
            self.save_file(frame);
            #[cfg(target_os = "macos")]
            Self::saving_file_false();
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

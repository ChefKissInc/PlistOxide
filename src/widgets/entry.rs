//! Copyright © 2022-2025 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//! See LICENSE for details.

use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use egui::{ComboBox, Context, Id, Label, Response, Sense, TextEdit, pos2, vec2};
use egui_extras::TableBody;
use plist::Value;
use serde::{Deserialize, Serialize};

use super::{click_text_edit::ClickableTextEdit, value::PlistValue};
use crate::utils::{ValueType, child_keys, pv_mut};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct State {
    expanded: bool,
}

impl State {
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(id))
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_persisted(id, self));
    }

    pub fn openness(&self, id: Id, ctx: &Context) -> f32 {
        if ctx.memory(egui::Memory::everything_is_visible) {
            1.0
        } else {
            ctx.animate_bool(id, self.expanded)
        }
    }
}

pub struct PlistEntry {
    data: Arc<Mutex<Value>>,
    path: Vec<String>,
    id: Id,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeState {
    Unchanged,
    Changed,
    Removed,
}

impl std::ops::BitOr for ChangeState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.max(rhs)
    }
}

impl std::ops::BitOrAssign for ChangeState {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl PlistEntry {
    pub fn new(data: Arc<Mutex<Value>>, path: Vec<String>) -> Self {
        let id = Id::new(&path);
        Self { data, path, id }
    }

    #[must_use]
    fn get_new_key(keys: plist::dictionary::Keys, k: &str) -> String {
        keys.filter(|v| (v.as_str() == k) || (v.starts_with(k) && v.ends_with("Duplicate")))
            .last()
            .map_or_else(|| "New Child".into(), |v| format!("{v} Duplicate"))
    }

    #[must_use]
    fn render_menu(resp: &Response, path: &[String], p: &mut Value) -> ChangeState {
        let mut ret = ChangeState::Unchanged;

        egui::Popup::context_menu(resp).show(|ui| {
            match ValueType::from_val(path, p) {
                ValueType::Dictionary => {
                    if ui.button("Add child").clicked() {
                        let dict = pv_mut(path, p).as_dictionary_mut().unwrap();
                        dict.insert(
                            Self::get_new_key(dict.keys(), "New Child"),
                            Value::String(String::new()),
                        );
                        ui.close();
                        ret |= ChangeState::Changed;
                    }
                    if ui.button("Sort").clicked() {
                        pv_mut(path, p).as_dictionary_mut().unwrap().sort_keys();
                        ui.close();
                        ret |= ChangeState::Changed;
                    }
                }
                ValueType::Array => {
                    if ui.button("Add child").clicked() {
                        pv_mut(path, p)
                            .as_array_mut()
                            .unwrap()
                            .push(Value::String(String::new()));
                        ui.close();
                        ret |= ChangeState::Changed;
                    }
                }
                _ => {}
            }

            let Some(k) = path.last() else {
                return;
            };

            if ui.button("Duplicate").clicked() {
                match pv_mut(&path[..path.len() - 1], p) {
                    Value::Dictionary(v) => {
                        v.insert(Self::get_new_key(v.keys(), k), v.get(k).unwrap().clone());
                    }
                    Value::Array(v) => {
                        v.push(v.get(k.parse::<usize>().unwrap()).unwrap().clone());
                    }
                    _ => unreachable!(),
                }
                ui.close();
                ret |= ChangeState::Changed;
            }

            if ui.button("Remove").clicked() {
                match pv_mut(&path[..path.len() - 1], p) {
                    Value::Dictionary(v) => {
                        v.remove(k);
                    }
                    Value::Array(v) => {
                        v.remove(k.parse::<usize>().unwrap());
                    }
                    _ => unreachable!(),
                }
                ui.close();
                ret |= ChangeState::Removed;
            }
        });

        ret
    }

    fn show_immutable_key(
        ui: &mut egui::Ui,
        mut s: &str,
        path: &[String],
        p: &mut Value,
    ) -> ChangeState {
        let resp = ui.add(
            TextEdit::singleline(&mut s)
                .desired_width(f32::INFINITY)
                .frame(false),
        );
        Self::render_menu(&resp, path, p)
    }

    pub fn show(self, body: &mut TableBody) -> ChangeState {
        let Self { data, mut path, id } = self;
        let mut state = State::load(body.ui_mut().ctx(), id).unwrap_or_default();
        let mut ty = ValueType::from_val(&path, &data.lock().unwrap());
        let keys = if ty.is_expandable() {
            child_keys(&path, &data.lock().unwrap())
        } else {
            vec![]
        };
        let mut ret = ChangeState::Unchanged;
        body.row(20.0, |mut row| {
            let resp = row
                .col(|ui| {
                    let prev_item_spacing = ui.spacing().item_spacing;
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add_space(ui.spacing().indent * path.len() as f32);
                    if ty.is_expandable() {
                        let size = vec2(ui.spacing().indent, ui.spacing().icon_width);
                        let (_id, rect) = ui.allocate_space(size);
                        let response = ui.interact(rect, self.id, Sense::click());
                        if response.clicked() {
                            state.expanded = !state.expanded;
                            ui.ctx().request_repaint();
                        }

                        let (mut icon_rect, _) = ui.spacing().icon_rectangles(response.rect);
                        icon_rect.set_center(pos2(
                            response.rect.left() + ui.spacing().indent / 2.0,
                            response.rect.center().y,
                        ));
                        let small_icon_response = response.with_new_rect(icon_rect);
                        egui::collapsing_header::paint_default_icon(
                            ui,
                            state.openness(id, ui.ctx()),
                            &small_icon_response,
                        );
                    }
                    let mut data = data.lock().unwrap();
                    if path.is_empty() {
                        ret |= Self::show_immutable_key(ui, "Root", &path, &mut data);
                        return;
                    }
                    let name = path.last().unwrap().clone();
                    let Some(dict) = pv_mut(&path[..path.len() - 1], &mut data).as_dictionary_mut()
                    else {
                        ret |= Self::show_immutable_key(ui, name.as_str(), &path, &mut data);
                        return;
                    };
                    let dict_clone = dict.clone();
                    let resp = ui.add(ClickableTextEdit::from_get_set(
                        |v| {
                            v.map_or_else(
                                || name.clone(),
                                |val| {
                                    if !dict.contains_key(&val) {
                                        dict.insert(val.clone(), dict.get(&name).unwrap().clone());
                                        path.last_mut().unwrap().clone_from(&val);
                                        dict.remove(&name);
                                    }
                                    val
                                },
                            )
                        },
                        |v| name == v || !dict_clone.contains_key(v),
                        false,
                    ));
                    ui.spacing_mut().item_spacing = prev_item_spacing;
                    ret |= Self::render_menu(&resp, &path, &mut data);
                })
                .1;
            if ret == ChangeState::Removed {
                return;
            }
            ret |= Self::render_menu(&resp, &path, &mut data.lock().unwrap());
            row.col(|ui| {
                let prev_type = ty;
                ComboBox::from_id_salt(id.with("type"))
                    .selected_text(format!("{ty:?}"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut ty, ValueType::Array, "Array");
                        ui.selectable_value(&mut ty, ValueType::Dictionary, "Dictionary");
                        if !path.is_empty() {
                            ui.selectable_value(&mut ty, ValueType::Boolean, "Boolean");
                            ui.selectable_value(&mut ty, ValueType::Data, "Data");
                            ui.selectable_value(&mut ty, ValueType::Real, "Real");
                            ui.selectable_value(&mut ty, ValueType::Integer, "Integer");
                            ui.selectable_value(&mut ty, ValueType::String, "String");
                        }
                    });
                if prev_type != ty {
                    let mut data = data.lock().unwrap();
                    *pv_mut(&path, &mut data) = match ty {
                        ValueType::Array => Value::Array(Default::default()),
                        ValueType::Dictionary => Value::Dictionary(Default::default()),
                        ValueType::Boolean => Value::Boolean(Default::default()),
                        ValueType::Data => Value::Data(Default::default()),
                        ValueType::Date => Value::Date(SystemTime::now().into()),
                        ValueType::Real => Value::Real(Default::default()),
                        ValueType::Integer => Value::Integer(0.into()),
                        ValueType::String => Value::String(Default::default()),
                    };
                    ret |= if prev_type.is_expandable() || ty.is_expandable() {
                        ChangeState::Removed
                    } else {
                        ChangeState::Changed
                    };
                }
            });
            if ret == ChangeState::Removed {
                return;
            }
            row.col(|ui| {
                if !ty.is_expandable() {
                    if PlistValue::new(&path, Arc::clone(&data)).show(ui) {
                        ret |= ChangeState::Changed;
                    }

                    return;
                }
                let len = keys.len();
                let s = if len == 1 { "" } else { "s" };
                match ty {
                    ValueType::Array => {
                        ui.add_enabled(false, Label::new(format!("{len} ordered object{s}")));
                    }
                    ValueType::Dictionary => {
                        ui.add_enabled(false, Label::new(format!("{len} key/value pair{s}")));
                    }
                    _ => unreachable!(),
                }
            });
        });
        if ret == ChangeState::Removed {
            return ret;
        }
        if state.expanded {
            for k in keys {
                ret |= Self::new(
                    Arc::clone(&data),
                    path.iter().chain(std::iter::once(&k)).cloned().collect(),
                )
                .show(body);
                if ret == ChangeState::Removed {
                    break;
                }
            }
        }
        state.store(body.ui_mut().ctx(), id);
        ret
    }
}

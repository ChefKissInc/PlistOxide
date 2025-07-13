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

#[must_use]
fn get_new_key(keys: plist::dictionary::Keys, k: &str) -> String {
    keys.filter(|v| (v.as_str() == k) || (v.starts_with(k) && v.ends_with("Duplicate")))
        .last()
        .map_or_else(|| "New Child".into(), |v| format!("{v} Duplicate"))
}

#[must_use]
pub fn render_menu(resp: &Response, path: &[String], p: &mut Value) -> Option<bool> {
    let mut ret = None;

    resp.context_menu(|ui| {
        match ValueType::from_val(path, p) {
            ValueType::Dictionary => {
                if ui.button("Add child").clicked() {
                    let dict = pv_mut(path, p).as_dictionary_mut().unwrap();
                    dict.insert(
                        get_new_key(dict.keys(), "New Child"),
                        Value::String(String::new()),
                    );
                    ui.close();
                    ret = Some(false);
                }
                if ui.button("Sort").clicked() {
                    pv_mut(path, p).as_dictionary_mut().unwrap().sort_keys();
                    ui.close();
                    ret = Some(false);
                }
            }
            ValueType::Array => {
                if ui.button("Add child").clicked() {
                    pv_mut(path, p)
                        .as_array_mut()
                        .unwrap()
                        .push(Value::String(String::new()));
                    ui.close();
                    ret = Some(false);
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
                    v.insert(get_new_key(v.keys(), k), v.get(k).unwrap().clone());
                }
                Value::Array(v) => {
                    v.push(v.get(k.parse::<usize>().unwrap()).unwrap().clone());
                }
                _ => unreachable!(),
            }
            ui.close();
            ret = Some(false);
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
            ret = Some(true);
        }
    });

    ret
}

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

impl PlistEntry {
    pub fn new(data: Arc<Mutex<Value>>, path: Vec<String>) -> Self {
        let id = Id::new(&path);
        Self { data, path, id }
    }

    pub fn show(self, body: &mut TableBody) -> Option<bool> {
        let Self { data, mut path, id } = self;
        let mut state = State::load(body.ui_mut().ctx(), id).unwrap_or_default();
        let mut ty = ValueType::from_val(&path, &data.lock().unwrap());
        let keys = if ty.is_expandable() {
            child_keys(&path, &data.lock().unwrap())
        } else {
            vec![]
        };
        let mut ret = None;
        body.row(20.0, |mut row| {
            let resp = row
                .col(|ui| {
                    let prev_item_spacing = ui.spacing().item_spacing;
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if !path.is_empty() {
                        ui.add_space(ui.spacing().indent * path.len() as f32);
                    }
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
                    if path.is_empty() {
                        let resp = ui.add(
                            TextEdit::singleline(&mut "Root")
                                .desired_width(f32::INFINITY)
                                .frame(false),
                        );
                        let v = render_menu(&resp, &path, &mut data.lock().unwrap());
                        ret = ret.map_or(v, |vv| Some(v.unwrap_or_default() || vv));
                        return;
                    }
                    let name = path.last().unwrap().clone();
                    let k = &name;
                    let mut data = data.lock().unwrap();
                    let Some(dict) = pv_mut(&path[..path.len() - 1], &mut data).as_dictionary_mut()
                    else {
                        let mut s = k.as_str();
                        let resp = ui.add(
                            TextEdit::singleline(&mut s)
                                .desired_width(f32::INFINITY)
                                .frame(false),
                        );
                        let v = render_menu(&resp, &path, &mut data);
                        ret = ret.map_or(v, |vv| Some(v.unwrap_or_default() || vv));
                        return;
                    };
                    let dict_clone = dict.clone();
                    let resp = ui.add(ClickableTextEdit::from_get_set(
                        |v| {
                            v.map_or_else(
                                || k.clone(),
                                |val| {
                                    if !dict.contains_key(&val) {
                                        dict.insert(val.clone(), dict.get(k).unwrap().clone());
                                        path.last_mut().unwrap().clone_from(&val);
                                        dict.remove(k);
                                    }
                                    val
                                },
                            )
                        },
                        move |v| k == v || !dict_clone.contains_key(v),
                        false,
                    ));
                    ui.spacing_mut().item_spacing = prev_item_spacing;
                    let v = render_menu(&resp, &path, &mut data);
                    drop(data);
                    ret = ret.map_or(v, |vv| Some(v.unwrap_or_default() || vv));
                })
                .1;
            if ret == Some(true) {
                return;
            }
            let v = render_menu(&resp, &path, &mut data.lock().unwrap());
            ret = ret.map_or(v, |vv| Some(v.unwrap_or_default() || vv));
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
                        ValueType::Array => Value::Array(vec![]),
                        ValueType::Dictionary => Value::Dictionary(plist::Dictionary::new()),
                        ValueType::Boolean => Value::Boolean(false),
                        ValueType::Data => Value::Data(vec![]),
                        ValueType::Date => Value::Date(plist::Date::from(SystemTime::now())),
                        ValueType::Real => Value::Real(0.0),
                        ValueType::Integer => Value::Integer(plist::Integer::from(0)),
                        ValueType::String => Value::String(String::new()),
                    };
                    ret = Some(prev_type.is_expandable() || ty.is_expandable());
                }
            });
            if ret == Some(true) {
                return;
            }
            row.col(|ui| {
                if !ty.is_expandable() {
                    if PlistValue::new(&path, Arc::clone(&data)).show(ui) {
                        ret = ret.or(Some(false));
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
        if ret == Some(true) {
            return ret;
        }
        if state.expanded {
            for k in keys {
                let v = Self::new(
                    Arc::clone(&data),
                    path.iter().chain(std::iter::once(&k)).cloned().collect(),
                )
                .show(body);
                ret = ret.map_or(v, |vv| Some(v.unwrap_or_default() || vv));
                if v == Some(true) {
                    break;
                }
            }
        }
        state.store(body.ui_mut().ctx(), id);
        ret
    }
}

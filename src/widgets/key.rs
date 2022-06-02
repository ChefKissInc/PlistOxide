use egui::{ComboBox, Grid, Label, Response, RichText, Sense, Ui};
use plist::{
    dictionary::{Entry, Keys},
    Value,
};

use super::click_text_edit::ClickableTextEdit;

#[derive(Debug, PartialEq, Eq)]
pub enum ValueType {
    String,
    Integer,
    Real,
    Boolean,
    Data,
    Array,
    Dictionary,
}

#[must_use]
#[inline]
pub fn pv<'a>(k: &str, p: &'a mut Value, is_root: bool) -> &'a Value {
    if !is_root {
        match p {
            Value::Dictionary(v) => &v[k],
            Value::Array(v) => &v[k.parse::<usize>().unwrap()],
            _ => unreachable!(),
        }
    } else {
        p
    }
}

#[must_use]
#[inline]
pub fn pv_mut<'a>(k: &str, p: &'a mut Value, is_root: bool) -> &'a mut Value {
    if !is_root {
        match p {
            Value::Dictionary(v) => &mut v[k],
            Value::Array(v) => &mut v[k.parse::<usize>().unwrap()],
            _ => unreachable!(),
        }
    } else {
        p
    }
}

#[must_use]
#[inline]
pub fn value_to_type(k: &str, p: &mut Value, is_root: bool) -> ValueType {
    match pv(k, p, is_root) {
        Value::String(_) => ValueType::String,
        Value::Integer(_) => ValueType::Integer,
        Value::Real(_) => ValueType::Real,
        Value::Boolean(_) => ValueType::Boolean,
        Value::Data(_) => ValueType::Data,
        Value::Array(_) => ValueType::Array,
        Value::Dictionary(_) => ValueType::Dictionary,
        _ => unreachable!(),
    }
}

#[must_use]
#[inline]
fn get_new_key(keys: Keys, k: &str) -> String {
    let keys = keys.filter(|v| (v.as_str() == k) || (v.starts_with(k) && v.ends_with("Duplicate")));

    if let Some(key) = keys.last() {
        key.clone() + " Duplicate"
    } else {
        "New Child".to_string()
    }
}

pub fn render_menu(resp: Response, k: &str, p: &mut Value, is_root: bool) -> bool {
    let mut changed = false;

    resp.context_menu(|ui| {
        match value_to_type(k, p, is_root) {
            ValueType::Dictionary => {
                if ui.button("Add child").clicked() {
                    let dict = pv_mut(k, p, is_root).as_dictionary_mut().unwrap();
                    dict.insert(
                        get_new_key(dict.keys(), "New Child"),
                        Value::String(String::new()),
                    );
                    ui.close_menu();
                }
                if ui.button("Sort").clicked() {
                    pv_mut(k, p, is_root)
                        .as_dictionary_mut()
                        .unwrap()
                        .sort_keys();
                    ui.close_menu();
                }
            }
            ValueType::Array => {
                if ui.button("Add child").clicked() {
                    pv_mut(k, p, is_root)
                        .as_array_mut()
                        .unwrap()
                        .push(Value::String(String::new()));
                    ui.close_menu();
                }
            }
            _ => {}
        }

        if !is_root {
            if ui.button("Duplicate").clicked() {
                match p {
                    Value::Dictionary(v) => {
                        v.insert(get_new_key(v.keys(), k), v.get(k).unwrap().clone());
                    }
                    Value::Array(v) => {
                        v.push(v.get(k.parse::<usize>().unwrap()).unwrap().clone());
                    }
                    _ => unreachable!(),
                }
                ui.close_menu();
            }

            if ui.button("Remove").clicked() {
                match p {
                    Value::Dictionary(v) => {
                        v.remove(k);
                    }
                    Value::Array(v) => {
                        v.remove(k.parse::<usize>().unwrap());
                    }
                    _ => unreachable!(),
                }
                ui.close_menu();
                changed = true;
            }
        }
    });

    changed
}

#[must_use]
pub fn render_key(
    state: &mut crate::app::State,
    ui: &mut Ui,
    k: &str,
    p: &mut Value,
    is_root: bool,
) -> bool {
    let mut changed = false;

    Grid::new(ui.id().with(k))
        .spacing([5., 5.])
        .min_col_width(0.)
        .show(ui, |ui| {
            Grid::new(ui.id().with(k)).spacing([5., 5.]).show(ui, |ui| {
                let mut ty = value_to_type(k, p, is_root);

                if !is_root {
                    if let Some(dict) = p.as_dictionary_mut() {
                        let auto_id = state.get_next_id();
                        let dict_clone = dict.clone();
                        let resp = ui.add(ClickableTextEdit::from_get_set(
                            |v| {
                                if let Some(val) = v {
                                    if !dict.contains_key(&val) {
                                        dict.insert(val.clone(), dict.get(k).unwrap().clone());
                                        if let Entry::Occupied(e) = dict.entry(k) {
                                            e.swap_remove();
                                        }

                                        changed = true;
                                    }
                                    val
                                } else {
                                    k.to_string()
                                }
                            },
                            move |v| k == v || !dict_clone.contains_key(v),
                            state
                                .data_store
                                .entry(ui.id())
                                .or_insert_with(|| Some(k.to_string())),
                            auto_id,
                            false,
                        ));
                        changed = changed || render_menu(resp, k, p, is_root);
                    } else {
                        changed = changed
                            || render_menu(
                                ui.add(
                                    Label::new(RichText::new(k).monospace()).sense(Sense::click()),
                                ),
                                k,
                                p,
                                is_root,
                            );
                    }
                } else {
                    changed = changed
                        || render_menu(
                            ui.add(Label::new(RichText::new(k).monospace()).sense(Sense::click())),
                            k,
                            p,
                            is_root,
                        );
                }

                if changed {
                    return;
                }

                let response = ComboBox::new(k, "")
                    .selected_text(format!("{:?}", ty))
                    .show_ui(ui, |ui| {
                        let mut set = |new_value: Value| {
                            *pv_mut(k, p, is_root) = new_value;
                            changed = true;
                        };

                        if ui
                            .selectable_value(&mut ty, ValueType::Array, "Array")
                            .changed()
                        {
                            set(Value::Array(vec![]));
                        }
                        if ui
                            .selectable_value(&mut ty, ValueType::Dictionary, "Dictionary")
                            .changed()
                        {
                            set(Value::Dictionary(plist::Dictionary::new()));
                        }
                        if !is_root {
                            if ui
                                .selectable_value(&mut ty, ValueType::Boolean, "Boolean")
                                .changed()
                            {
                                set(Value::Boolean(false));
                            }

                            if ui
                                .selectable_value(&mut ty, ValueType::Data, "Data")
                                .changed()
                            {
                                set(Value::Data(vec![]));
                            }

                            if ui
                                .selectable_value(&mut ty, ValueType::Integer, "Integer")
                                .changed()
                            {
                                set(Value::Integer(plist::Integer::from(0)));
                            }

                            if ui
                                .selectable_value(&mut ty, ValueType::Real, "Real")
                                .changed()
                            {
                                set(Value::Real(0.0));
                            }

                            if ui
                                .selectable_value(&mut ty, ValueType::String, "String")
                                .changed()
                            {
                                set(Value::String("".to_string()));
                            }
                        }
                    })
                    .response;

                changed = changed || render_menu(response, k, p, is_root);
            });
        });

    changed
}

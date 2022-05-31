use egui::{ComboBox, Grid, Id, Response, RichText, Ui};
use either::Either;
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
pub fn get_child(k: &str, p: &mut Either<&mut Value, &mut Value>) -> Option<Value> {
    match p {
        Either::Left(v) => {
            match v {
                Value::Dictionary(v) => v.get(k).cloned(),
                Value::Array(v) => v.get(k.parse::<usize>().unwrap()).cloned(),
                _ => unreachable!(),
            }
        }
        Either::Right(v) => Some(v.clone()),
    }
}

pub fn set_child(k: &str, p: &mut Either<&mut Value, &mut Value>, val: Value) {
    match p {
        Either::Left(v) => {
            match v {
                Value::Dictionary(v) => v[k] = val,
                Value::Array(v) => v[k.parse::<usize>().unwrap()] = val,
                _ => unreachable!(),
            }
        }
        Either::Right(v) => {
            **v = val;
        }
    }
}

#[must_use]
pub fn pv<'a>(k: &str, p: &'a mut Either<&mut Value, &mut Value>) -> &'a mut Value {
    match p {
        Either::Left(v) => {
            match v {
                Value::Dictionary(v) => &mut v[k],
                Value::Array(v) => &mut v[k.parse::<usize>().unwrap()],
                _ => unreachable!(),
            }
        }
        Either::Right(v) => v,
    }
}

#[must_use]
pub fn value_to_type(k: &str, p: &mut Either<&mut Value, &mut Value>) -> ValueType {
    match get_child(k, p).unwrap() {
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
fn get_new_key(keys: Keys, k: &str) -> String {
    let keys = keys.filter(|v| (v.as_str() == k) || (v.starts_with(k) && v.ends_with("Duplicate")));

    if let Some(key) = keys.last() {
        key.clone() + " Duplicate"
    } else {
        "New Child".to_string()
    }
}

#[must_use]
pub fn render_menu(
    ui: &mut Ui,
    id: Id,
    response: &Response,
    k: &str,
    p: &mut Either<&mut Value, &mut Value>,
) -> bool {
    let mut changed = false;

    egui::popup::popup_below_widget(ui, id, response, |ui| {
        ui.set_min_width(100.0);
        let ty = value_to_type(k, p);
        match ty {
            ValueType::Dictionary => {
                if ui.button("Add child").clicked() {
                    let dict = pv(k, p).as_dictionary_mut().unwrap();
                    dict.insert(
                        get_new_key(dict.keys(), "New Child"),
                        Value::String(String::new()),
                    );
                }
            }
            ValueType::Array => {
                if ui.button("Add child").clicked() {
                    pv(k, p)
                        .as_array_mut()
                        .unwrap()
                        .push(Value::String(String::new()));
                }
            }
            _ => {}
        }

        if let Either::Left(v) = p {
            if ui.button("Duplicate").clicked() {
                match v {
                    Value::Dictionary(v) => {
                        v.insert(get_new_key(v.keys(), k), v.get(k).unwrap().clone());
                    }
                    Value::Array(v) => {
                        v.push(v.get(k.parse::<usize>().unwrap()).unwrap().clone());
                    }
                    _ => unreachable!(),
                }
            }

            if ui.button("Remove").clicked() {
                match v {
                    Value::Dictionary(v) => {
                        v.remove(k);
                    }
                    Value::Array(v) => {
                        v.remove(k.parse::<usize>().unwrap());
                    }
                    _ => unreachable!(),
                }
                changed = true;
                return;
            }
        }

        if (ty == ValueType::Dictionary) && ui.button("Sort").clicked() {
            pv(k, p).as_dictionary_mut().unwrap().sort_keys();
        }
    });

    changed
}

#[must_use]
pub fn render_key(
    state: &mut crate::app::State,
    ui: &mut Ui,
    k: &str,
    p: &mut Either<&mut Value, &mut Value>,
) -> bool {
    let mut changed = false;

    Grid::new(ui.id().with(k))
        .spacing([5., 5.])
        .min_col_width(0.)
        .show(ui, |ui| {
            let mut ty = value_to_type(k, p);

            let resp = ui.button("...");
            let id = ui.make_persistent_id(format!("elem_opts_{}", k));
            if resp.secondary_clicked() || resp.clicked() {
                ui.memory().open_popup(id)
            }

            changed = render_menu(ui, id, &resp, k, p);

            Grid::new(ui.id().with(k)).spacing([5., 5.]).show(ui, |ui| {
                if changed {
                    return;
                }

                if let Either::Left(v) = p {
                    if let Some(dict) = v.as_dictionary_mut() {
                        let auto_id = state.get_next_id();
                        let dict_clone = dict.clone();
                        ui.add(ClickableTextEdit::new(
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
                    } else {
                        ui.label(RichText::new(k).monospace());
                    }
                } else {
                    ui.label(RichText::new(k).monospace());
                }

                if changed {
                    return;
                }

                let root = p.is_right();

                let mut set = |new_value: Value| {
                    set_child(k, p, new_value);
                    changed = true;
                };

                ComboBox::new(k, "")
                    .selected_text(format!("{:?}", ty))
                    .show_ui(ui, |ui| {
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
                        if !root {
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
                    });
            });
        });

    changed
}

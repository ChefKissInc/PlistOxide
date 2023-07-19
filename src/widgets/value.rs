//  Copyright © 2022-2023 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for
//  details.

use std::sync::{Arc, Mutex};

use egui::{DragValue, Ui, Widget};
use plist::Value;

use super::{click_text_edit::ClickableTextEdit, toggle::Toggle};
use crate::utils::{pv, pv_mut, ValueType};

pub struct PlistValue<'a> {
    path: &'a [String],
    data: Arc<Mutex<Value>>,
}

impl<'a> PlistValue<'a> {
    #[must_use]
    #[inline]
    pub fn new(path: &'a [String], data: Arc<Mutex<Value>>) -> Self {
        Self { path, data }
    }
}

impl<'a> Widget for PlistValue<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let Self { path, data } = self;

        let ty = ValueType::from_val(path, &data.lock().unwrap());
        let mut data = data.lock().unwrap();
        match ty {
            ValueType::String => {
                let Value::String(s) = pv_mut(path, &mut data) else {
                    unreachable!();
                };
                ui.add(ClickableTextEdit::new(s, |_| true, false))
            }
            ValueType::Integer => {
                let Some(i) = pv(path, &data).as_signed_integer().map(|v| v.to_string()) else {
                    unreachable!();
                };
                ui.add(ClickableTextEdit::from_get_set(
                    |v| {
                        if let Some(val) = &v {
                            if let Ok(val) = val.parse::<i64>() {
                                *pv_mut(path, &mut data) = Value::Integer(val.into());
                            }
                        }
                        v.unwrap_or_else(|| i.clone())
                    },
                    |v| v.parse::<i64>().is_ok(),
                    false,
                ))
            }
            ValueType::Real => {
                let Value::Real(v) = pv_mut(path, &mut data) else {
                    unreachable!();
                };
                ui.add(DragValue::new(v))
            }
            ValueType::Boolean => {
                let Value::Boolean(v) = pv_mut(path, &mut data) else {
                    unreachable!();
                };
                ui.add(Toggle::new(v))
            }
            ValueType::Data => {
                let Some(val) = pv(path, &data).as_data() else {
                    unreachable!();
                };
                let val = hex::encode_upper(val);
                ui.add(ClickableTextEdit::from_get_set(
                    |v| {
                        if let Some(val) = &v {
                            if let Ok(val) = hex::decode(val) {
                                *pv_mut(path, &mut data) = Value::Data(val);
                            }
                        }
                        v.unwrap_or_else(|| val.clone())
                    },
                    |v| v.len() % 2 == 0 && hex::decode(v).is_ok(),
                    false,
                ))
            }
            _ => ui.label("Not serialisable"),
        }
    }
}

//! Copyright © 2022-2024 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//! See LICENSE for details.

use std::sync::{Arc, Mutex};

use egui::{DragValue, Ui};
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
    pub const fn new(path: &'a [String], data: Arc<Mutex<Value>>) -> Self {
        Self { path, data }
    }

    #[must_use]
    pub fn show(self, ui: &mut Ui) -> bool {
        let Self { path, data } = self;

        let ty = ValueType::from_val(path, &data.lock().unwrap());
        let mut data = data.lock().unwrap();
        match ty {
            ValueType::String => {
                let Value::String(s) = pv_mut(path, &mut data) else {
                    unreachable!();
                };
                let mut changed = false;
                ui.add(ClickableTextEdit::from_get_set(
                    |v| {
                        if let Some(v) = v {
                            *s = v;
                            changed = true;
                        }
                        s.clone()
                    },
                    |_| true,
                    false,
                ));
                changed
            }
            ValueType::Integer => {
                let Some(i) = pv(path, &data).as_signed_integer().map(|v| v.to_string()) else {
                    unreachable!();
                };
                let mut changed = false;
                ui.add(ClickableTextEdit::from_get_set(
                    |v| {
                        if let Some(val) = &v {
                            if let Ok(val) = val.parse::<i64>() {
                                *pv_mut(path, &mut data) = Value::Integer(val.into());
                                changed = true;
                            }
                        }
                        v.unwrap_or_else(|| i.clone())
                    },
                    |v| v.parse::<i64>().is_ok(),
                    false,
                ));
                changed
            }
            ValueType::Real => {
                let Value::Real(value) = pv_mut(path, &mut data) else {
                    unreachable!();
                };
                let mut changed = false;
                ui.add(DragValue::from_get_set(|v: Option<f64>| {
                    if let Some(v) = v {
                        *value = v;
                        changed = true;
                    }
                    *value
                }));
                changed
            }
            ValueType::Boolean => {
                let Value::Boolean(v) = pv_mut(path, &mut data) else {
                    unreachable!();
                };
                ui.add(Toggle::new(v)).clicked()
            }
            ValueType::Data => {
                let Some(val) = pv(path, &data).as_data() else {
                    unreachable!();
                };
                let val = hex::encode_upper(val);
                let mut changed = false;
                ui.add(ClickableTextEdit::from_get_set(
                    |v| {
                        if let Some(val) = &v {
                            if let Ok(val) = hex::decode(val) {
                                *pv_mut(path, &mut data) = Value::Data(val);
                                changed = true;
                            }
                        }
                        v.unwrap_or_else(|| val.clone())
                    },
                    |v| v.len().is_multiple_of(2) && hex::decode(v).is_ok(),
                    false,
                ));
                changed
            }
            _ => {
                ui.label("Not serialisable");
                false
            }
        }
    }
}

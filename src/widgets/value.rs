use egui::{collapsing_header::CollapsingState, DragValue, RichText, Ui, Vec2};
use either::Either;
use plist::Value;

use super::key::{get_child, pv, render_key, set_child};
use crate::widgets::click_text_edit::ClickableTextEdit;

pub fn render_value(
    state: &mut crate::app::State,
    ui: &mut Ui,
    k: &str,
    p: &mut Either<&mut Value, &mut Value>,
) {
    let v = get_child(k, p);
    let auto_id = state.get_next_id();

    if let Some(v) = v {
        match v {
            Value::String(s) => {
                if !render_key(state, ui, k, p) {
                    ui.add(ClickableTextEdit::new(
                        |v| {
                            if let Some(val) = v {
                                set_child(k, p, Value::String(val))
                            }
                            s.clone()
                        },
                        |_| true,
                        state
                            .data_store
                            .entry(ui.id())
                            .or_insert_with(|| Some(s.clone())),
                        auto_id,
                    ));
                }
            }
            Value::Integer(i) => {
                if !render_key(state, ui, k, p) {
                    ui.add(ClickableTextEdit::new(
                        |v| {
                            if let Some(val) = v.clone() {
                                if let Ok(val) = val.parse::<i64>() {
                                    set_child(k, p, Value::Integer(val.into()))
                                }
                            }
                            v.unwrap_or_else(|| i.as_signed().unwrap().to_string())
                        },
                        |v| v.parse::<i64>().is_ok(),
                        state
                            .data_store
                            .entry(ui.id())
                            .or_insert_with(|| Some(i.as_signed().unwrap().to_string())),
                        auto_id,
                    ));
                }
            }
            Value::Real(val) => {
                if !render_key(state, ui, k, p) {
                    ui.add(DragValue::from_get_set(move |v| {
                        if let Some(val) = v {
                            set_child(k, p, Value::Real(val))
                        }
                        v.unwrap_or(val)
                    }));
                }
            }
            Value::Boolean(b) => {
                if !render_key(state, ui, k, p) {
                    let mut val = b;
                    if ui.checkbox(&mut val, "").changed() {
                        set_child(k, p, Value::Boolean(val));
                    }
                }
            }
            Value::Data(d) => {
                if !render_key(state, ui, k, p) {
                    let val = hex::encode_upper(d);
                    ui.add(ClickableTextEdit::new(
                        |v| {
                            if let Some(val) = v.clone() {
                                if let Ok(val) = hex::decode(val) {
                                    set_child(k, p, Value::Data(val))
                                }
                            }
                            v.unwrap_or_else(|| val.clone())
                        },
                        |v| v.len() % 2 == 0 && hex::decode(v).is_ok(),
                        state
                            .data_store
                            .entry(ui.id())
                            .or_insert_with(|| Some(val.clone())),
                        auto_id,
                    ));
                }
            }
            Value::Array(a) => {
                let mut changed = false;
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered_justified(|ui| {
                        CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(k), false)
                            .show_header(ui, |ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(10., 10.);
                                changed = render_key(state, ui, k, p);
                            })
                            .body(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    if !changed {
                                        let keys =
                                            (0..a.len()).map(|v| v.to_string()).collect::<Vec<_>>();
                                        let p = &mut Either::Left(pv(k, p));
                                        for k in &keys {
                                            ui.horizontal(|ui| render_value(state, ui, k, p));
                                        }
                                    }
                                });
                            });
                    });
                });
            }
            Value::Dictionary(d) => {
                let mut changed = false;
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered_justified(|ui| {
                        CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(k), false)
                            .show_header(ui, |ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(10., 10.);
                                changed = render_key(state, ui, k, p);
                            })
                            .body(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    if !changed {
                                        let keys = d.iter().map(|(k, _)| k).collect::<Vec<_>>();
                                        let p = &mut Either::Left(pv(k, p));
                                        for k in keys {
                                            ui.horizontal(|ui| render_value(state, ui, k, p));
                                        }
                                    }
                                });
                            });
                    });
                });
            }
            _ => {
                ui.label(RichText::new(k).strong());
                ui.label("Unserialisable");
            }
        }
    }
}

use egui::{collapsing_header::CollapsingState, style::Margin, DragValue, Frame, Ui};
use either::Either;
use plist::Value;

use super::{
    click_text_edit::ClickableTextEdit,
    key::{pv, pv_mut, render_key, value_to_type, ValueType},
};

pub fn render_value(
    state: &mut crate::app::State,
    ui: &mut Ui,
    k: &str,
    p: &mut Either<&mut Value, &mut Value>,
) {
    let auto_id = state.get_next_id();

    match value_to_type(k, p) {
        ValueType::String => {
            let s = pv(k, p).as_string().unwrap().to_string();
            if !render_key(state, ui, k, p) {
                ui.add(ClickableTextEdit::new(
                    |v| {
                        if let Some(val) = v {
                            *pv_mut(k, p) = Value::String(val);
                        }
                        s.clone()
                    },
                    |_| true,
                    state
                        .data_store
                        .entry(ui.id())
                        .or_insert_with(|| Some(s.clone())),
                    auto_id,
                    true,
                ));
            }
        }
        ValueType::Integer => {
            if !render_key(state, ui, k, p) {
                let i = pv(k, p).as_signed_integer().unwrap().to_string();
                ui.add(ClickableTextEdit::new(
                    |v| {
                        if let Some(val) = v.clone() {
                            if let Ok(val) = val.parse::<i64>() {
                                *pv_mut(k, p) = Value::Integer(val.into());
                            }
                        }
                        v.unwrap_or_else(|| i.clone())
                    },
                    |v| v.parse::<i64>().is_ok(),
                    state
                        .data_store
                        .entry(ui.id())
                        .or_insert_with(|| Some(i.clone())),
                    auto_id,
                    true,
                ));
            }
        }
        ValueType::Real => {
            if !render_key(state, ui, k, p) {
                if let Value::Real(v) = pv_mut(k, p) {
                    ui.add(DragValue::new(v));
                }
            }
        }
        ValueType::Boolean => {
            if !render_key(state, ui, k, p) {
                if let Value::Boolean(v) = pv_mut(k, p) {
                    ui.checkbox(v, "");
                }
            }
        }
        ValueType::Data => {
            if !render_key(state, ui, k, p) {
                let val = hex::encode_upper(pv(k, p).as_data().unwrap());
                ui.add(ClickableTextEdit::new(
                    |v| {
                        if let Some(val) = v.clone() {
                            if let Ok(val) = hex::decode(val) {
                                *pv_mut(k, p) = Value::Data(val);
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
                    true,
                ));
            }
        }
        ValueType::Array => {
            let mut changed = false;

            ui.vertical_centered_justified(|ui| {
                ui.set_min_width(ui.available_width());
                CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(k), false)
                    .show_header(ui, |ui| {
                        changed = render_key(state, ui, k, p);
                    })
                    .body(|ui| {
                        ui.vertical_centered_justified(|ui| {
                            ui.set_min_width(ui.available_width());
                            if !changed {
                                let a = pv(k, p).as_array().unwrap();
                                let keys = (0..a.len()).map(|v| v.to_string()).collect::<Vec<_>>();
                                let p = &mut Either::Left(pv_mut(k, p));
                                let mut fill = false;
                                for k in &keys {
                                    let fill_colour = if fill {
                                        ui.style().visuals.faint_bg_color
                                    } else {
                                        ui.style().visuals.window_fill()
                                    };
                                    Frame::none()
                                        .fill(fill_colour)
                                        .inner_margin(Margin::same(5.))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.horizontal(|ui| render_value(state, ui, k, p))
                                        });
                                    fill = !fill;
                                }
                            }
                        });
                    });
            });
        }
        ValueType::Dictionary => {
            let mut changed = false;

            ui.vertical_centered_justified(|ui| {
                ui.set_min_width(ui.available_width());
                CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(k), false)
                    .show_header(ui, |ui| {
                        changed = render_key(state, ui, k, p);
                    })
                    .body(|ui| {
                        ui.vertical_centered_justified(|ui| {
                            ui.set_min_width(ui.available_width());
                            if !changed {
                                let d = pv_mut(k, p);
                                let keys = d
                                    .as_dictionary()
                                    .unwrap()
                                    .keys()
                                    .cloned()
                                    .collect::<Vec<_>>();
                                let p = &mut Either::Left(d);
                                let mut fill = false;
                                for k in &keys {
                                    let fill_colour = if fill {
                                        ui.style().visuals.faint_bg_color
                                    } else {
                                        ui.style().visuals.window_fill()
                                    };
                                    Frame::none()
                                        .fill(fill_colour)
                                        .inner_margin(Margin::same(5.))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.horizontal(|ui| {
                                                render_value(state, ui, k, p);
                                            })
                                        });
                                    fill = !fill;
                                }
                            }
                        });
                    });
            });
        }
    }
}

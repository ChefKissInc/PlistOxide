use egui::{collapsing_header::CollapsingState, style::Margin, DragValue, Frame, Ui};
use plist::Value;

use super::{
    click_text_edit::ClickableTextEdit,
    key::{pv, pv_mut, render_key, render_menu, value_to_type, ValueType},
};

pub fn render_value(
    state: &mut crate::app::State,
    ui: &mut Ui,
    k: &str,
    p: &mut Value,
    is_root: bool,
) {
    let auto_id = state.get_next_id();

    match value_to_type(k, p, is_root) {
        ValueType::String => {
            if !render_key(state, ui, k, p, is_root) {
                if let Value::String(s) = pv_mut(k, p, is_root) {
                    ui.add(ClickableTextEdit::new(
                        s,
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
        }
        ValueType::Integer => {
            if !render_key(state, ui, k, p, is_root) {
                let i = pv(k, p, is_root).as_signed_integer().unwrap().to_string();
                ui.add(ClickableTextEdit::from_get_set(
                    |v| {
                        if let Some(val) = v.clone() {
                            if let Ok(val) = val.parse::<i64>() {
                                *pv_mut(k, p, is_root) = Value::Integer(val.into());
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
            if !render_key(state, ui, k, p, is_root) {
                if let Value::Real(v) = pv_mut(k, p, is_root) {
                    ui.add(DragValue::new(v));
                }
            }
        }
        ValueType::Boolean => {
            if !render_key(state, ui, k, p, is_root) {
                if let Value::Boolean(v) = pv_mut(k, p, is_root) {
                    ui.checkbox(v, "");
                }
            }
        }
        ValueType::Data => {
            if !render_key(state, ui, k, p, is_root) {
                let val = hex::encode_upper(pv(k, p, is_root).as_data().unwrap());
                ui.add(ClickableTextEdit::from_get_set(
                    |v| {
                        if let Some(val) = v.clone() {
                            if let Ok(val) = hex::decode(val) {
                                *pv_mut(k, p, is_root) = Value::Data(val);
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

            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());
                CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(k), false)
                    .show_header(ui, |ui| {
                        changed = render_key(state, ui, k, p, is_root);
                    })
                    .body(|ui| {
                        ui.vertical_centered_justified(|ui| {
                            ui.set_min_width(ui.available_width());
                            if !changed {
                                let len = pv(k, p, is_root).as_array().unwrap().len();
                                let keys = (0..len).map(|v| v.to_string()).collect::<Vec<_>>();
                                let p = pv_mut(k, p, is_root);
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
                                            ui.horizontal(|ui| render_value(state, ui, k, p, false))
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

            let response = ui
                .vertical(|ui| {
                    ui.set_min_width(ui.available_width());
                    CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(k), false)
                        .show_header(ui, |ui| {
                            changed = render_key(state, ui, k, p, is_root);
                        })
                        .body(|ui| {
                            ui.vertical_centered_justified(|ui| {
                                ui.set_min_width(ui.available_width());
                                if !changed {
                                    let p = pv_mut(k, p, is_root);
                                    let keys = p
                                        .as_dictionary()
                                        .unwrap()
                                        .keys()
                                        .cloned()
                                        .collect::<Vec<_>>();

                                    let mut fill = false;
                                    for k in &keys {
                                        let fill_colour = if fill {
                                            ui.style().visuals.faint_bg_color
                                        } else {
                                            ui.style().visuals.window_fill()
                                        };
                                        render_menu(
                                            Frame::none()
                                                .fill(fill_colour)
                                                .inner_margin(Margin::same(5.))
                                                .show(ui, |ui| {
                                                    ui.set_min_width(ui.available_width());
                                                    ui.horizontal(|ui| {
                                                        render_value(state, ui, k, p, false);
                                                    })
                                                })
                                                .response,
                                            k,
                                            p,
                                            is_root,
                                        );
                                        fill = !fill;
                                    }
                                }
                            });
                        });
                })
                .response;
            if is_root {
                render_menu(response, k, p, true);
            }
        }
    }
}

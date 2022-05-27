use egui::{collapsing_header::CollapsingState, DragValue, Grid, RichText, TextEdit, Ui, Vec2};
use plist::Value;

pub fn serialise(ui: &mut Ui, k: &str, v_: &mut Value) {
    match v_.clone() {
        Value::String(_) => {
            ui.label(RichText::new(k).strong());
            if !crate::value_type::dropdown(ui, k, v_) {
                TextEdit::singleline(if let Value::String(v) = v_ {
                    v
                } else {
                    unreachable!()
                })
                .code_editor()
                .show(ui);
            }
        }
        Value::Integer(v) => {
            ui.label(RichText::new(k).strong());
            if !crate::value_type::dropdown(ui, k, v_) {
                ui.label(RichText::new(v.to_string()).strong().code());
            }
        }
        Value::Real(_) => {
            ui.label(RichText::new(k).strong());
            if !crate::value_type::dropdown(ui, k, v_) {
                ui.add(DragValue::new(if let Value::Real(v) = v_ {
                    v
                } else {
                    unreachable!()
                }));
            }
        }
        Value::Boolean(_) => {
            ui.label(RichText::new(k).strong());
            if !crate::value_type::dropdown(ui, k, v_) {
                ui.checkbox(
                    if let Value::Boolean(v) = v_ {
                        v
                    } else {
                        unreachable!()
                    },
                    "",
                );
            }
        }
        Value::Data(v) => {
            ui.label(RichText::new(k).strong());
            if !crate::value_type::dropdown(ui, k, v_) {
                ui.label(
                    RichText::new(v.iter().map(|v| format!("{:02x}", v)).collect::<String>())
                        .strong()
                        .code(),
                );
            }
        }
        Value::Array(_) => {
            let mut changed = false;
            ui.vertical(|ui| {
                CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(k), false)
                    .show_header(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(10., 10.);
                        ui.label(RichText::new(k).strong());
                        changed = crate::value_type::dropdown(ui, k, v_);
                    })
                    .body(|ui| {
                        Grid::new(k)
                            .striped(true)
                            .spacing([10., 10.])
                            .show(ui, |ui| {
                                if !changed {
                                    let v = if let Value::Array(v) = v_ {
                                        v
                                    } else {
                                        unreachable!()
                                    };

                                    for (i, v) in v.iter_mut().enumerate() {
                                        serialise(ui, &i.to_string(), v);
                                        ui.end_row();
                                    }
                                }
                            });
                    });
            });
        }
        Value::Dictionary(_) => {
            let mut changed = false;
            ui.vertical(|ui| {
                CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(k), false)
                    .show_header(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(10., 10.);
                        ui.label(RichText::new(k).strong());
                        changed = crate::value_type::dropdown(ui, k, v_);
                    })
                    .body(|ui| {
                        Grid::new(k)
                            .striped(true)
                            .spacing([10., 10.])
                            .show(ui, |ui| {
                                if !changed {
                                    let v = if let Value::Dictionary(v) = v_ {
                                        v
                                    } else {
                                        unreachable!()
                                    };
                                    for (k, v) in v.iter_mut() {
                                        serialise(ui, k, v);
                                        ui.end_row();
                                    }
                                }
                            });
                    });
            });
        }
        _ => {}
    }
}

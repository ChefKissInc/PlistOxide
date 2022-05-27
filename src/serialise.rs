use egui::{collapsing_header::CollapsingState, DragValue, RichText, TextEdit, Ui, Vec2};
use either::*;
use plist::Value;

pub fn serialise(ui: &mut Ui, k: &str, p: &mut Either<&mut Value, &mut Value>) {
    let v = crate::value::get_child(k, p);

    match v {
        Value::String(s) => {
            if !crate::value::render_key(ui, k, p) {
                let mut val = s;
                if TextEdit::singleline(&mut val)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .show(ui)
                    .response
                    .changed()
                {
                    crate::value::set_child(k, p, Value::String(val));
                }
            }
        }
        Value::Integer(i) => {
            if !crate::value::render_key(ui, k, p) {
                let mut val = i.to_string();
                if TextEdit::singleline(&mut val)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .show(ui)
                    .response
                    .changed()
                {
                    crate::value::set_child(
                        k,
                        p,
                        Value::Integer(val.parse::<i64>().unwrap().into()),
                    );
                }
            }
        }
        Value::Real(val) => {
            if !crate::value::render_key(ui, k, p) {
                let mut val = val;
                if ui.add(DragValue::new(&mut val)).changed() {
                    crate::value::set_child(k, p, Value::Real(val));
                }
            }
        }
        Value::Boolean(b) => {
            if !crate::value::render_key(ui, k, p) {
                let mut val = b;
                if ui.checkbox(&mut val, "").changed() {
                    crate::value::set_child(k, p, Value::Boolean(val));
                }
            }
        }
        Value::Data(d) => {
            if !crate::value::render_key(ui, k, p) {
                TextEdit::singleline(
                    &mut d.iter().map(|v| format!("{:02X}", v)).collect::<String>(),
                )
                .code_editor()
                .desired_width(f32::INFINITY)
                .show(ui);
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
                            changed = crate::value::render_key(ui, k, p);
                        })
                        .body(|ui| {
                            ui.vertical_centered_justified(|ui| {
                                if !changed {
                                    let keys =
                                        (0..a.len()).map(|v| v.to_string()).collect::<Vec<_>>();
                                    let p = &mut Either::Left(crate::value::pv(k, p));
                                    for k in &keys {
                                        ui.horizontal(|ui| serialise(ui, k, p));
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

                            changed = crate::value::render_key(ui, k, p);
                        })
                        .body(|ui| {
                            ui.vertical_centered_justified(|ui| {
                                if !changed {
                                    let keys = d.iter().map(|(k, _)| k).collect::<Vec<_>>();
                                    let p = &mut Either::Left(crate::value::pv(k, p));
                                    for k in keys {
                                        ui.horizontal(|ui| serialise(ui, k, p));
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

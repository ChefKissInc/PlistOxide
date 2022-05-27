use egui::{ComboBox, Ui};
use plist::Value;

#[derive(Debug, PartialEq, Eq)]
enum ValueType {
    String,
    Integer,
    Real,
    Boolean,
    Data,
    Array,
    Dictionary,
}

#[must_use]
pub fn dropdown(ui: &mut Ui, k: &str, v: &mut Value) -> bool {
    let mut value = match v {
        Value::String(_) => ValueType::String,
        Value::Integer(_) => ValueType::Integer,
        Value::Real(_) => ValueType::Real,
        Value::Boolean(_) => ValueType::Boolean,
        Value::Data(_) => ValueType::Data,
        Value::Array(_) => ValueType::Array,
        Value::Dictionary(_) => ValueType::Dictionary,
        _ => unreachable!(),
    };
    let mut changed = false;

    ComboBox::new(k, "")
        .selected_text(format!("{:?}", value))
        .show_ui(ui, |ui| {
            if ui
                .selectable_value(&mut value, ValueType::Array, "Array")
                .changed()
            {
                *v = Value::Array(vec![]);
                changed = true;
            }
            if ui
                .selectable_value(&mut value, ValueType::Dictionary, "Dictionary")
                .changed()
            {
                *v = Value::Dictionary(plist::Dictionary::new());
                changed = true;
            }
            if k != "Root" {
                if ui
                    .selectable_value(&mut value, ValueType::Boolean, "Boolean")
                    .changed()
                {
                    *v = Value::Boolean(false);
                    changed = true;
                }
                if ui
                    .selectable_value(&mut value, ValueType::Data, "Data")
                    .changed()
                {
                    *v = Value::Data(vec![]);
                    changed = true;
                }
                if ui
                    .selectable_value(&mut value, ValueType::Integer, "Integer")
                    .changed()
                {
                    *v = Value::Integer(plist::Integer::from(0));
                    changed = true;
                }
                if ui
                    .selectable_value(&mut value, ValueType::Real, "Real")
                    .changed()
                {
                    *v = Value::Real(0.0);
                    changed = true;
                }
                if ui
                    .selectable_value(&mut value, ValueType::String, "String")
                    .changed()
                {
                    *v = Value::String("".to_string());
                    changed = true;
                }
            }
        });
    changed
}

//  Copyright © 2022-2023 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for
//  details.

use egui::{
    Color32, Context, CursorIcon, Id, Key, Response, RichText, TextEdit, TextStyle, Ui, Widget,
    WidgetInfo,
};

type GetSetValue<'a> = Box<dyn 'a + FnMut(Option<String>) -> String>;
type ValidateValue<'a> = Box<dyn 'a + FnMut(&str) -> bool>;

fn get(get_set_value: &mut GetSetValue<'_>) -> String {
    (get_set_value)(None)
}

fn set(get_set_value: &mut GetSetValue<'_>, value: String) {
    (get_set_value)(Some(value));
}

#[derive(Clone, Debug, Default, PartialEq)]
struct State {
    edit_string: String,
}

impl State {
    #[must_use]
    #[inline]
    pub const fn new(edit_string: String) -> Self {
        Self { edit_string }
    }

    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_temp(id))
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
}

#[must_use]
pub struct ClickableTextEdit<'a> {
    get_set_value: GetSetValue<'a>,
    validate_value: ValidateValue<'a>,
    frame: bool,
}

impl<'a> ClickableTextEdit<'a> {
    #[inline]
    pub fn new(
        value: &'a mut String,
        validate_value: impl 'a + FnMut(&str) -> bool,
        frame: bool,
    ) -> Self {
        Self::from_get_set(
            move |v| {
                if let Some(v) = v {
                    *value = v;
                }
                value.clone()
            },
            validate_value,
            frame,
        )
    }

    #[inline]
    pub fn from_get_set(
        get_set_value: impl 'a + FnMut(Option<String>) -> String,
        validate_value: impl 'a + FnMut(&str) -> bool,
        frame: bool,
    ) -> Self {
        Self {
            get_set_value: Box::new(get_set_value),
            validate_value: Box::new(validate_value),
            frame,
        }
    }
}

impl<'a> Widget for ClickableTextEdit<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            mut get_set_value,
            mut validate_value,
            frame,
        } = self;

        let old_value = get(&mut get_set_value);
        let state_id = ui.auto_id_with("state");
        let mut state =
            State::load(ui.ctx(), state_id).unwrap_or_else(|| State::new(old_value.clone()));

        let kb_edit_id = ui.auto_id_with("kb_edit");
        let popup_id = kb_edit_id.with("popup");
        let is_kb_editing = ui.memory(|v| v.has_focus(kb_edit_id));

        let mut response = if is_kb_editing {
            let response = ui.add(
                TextEdit::singleline(&mut state.edit_string)
                    .id(kb_edit_id)
                    .font(TextStyle::Monospace),
            );
            egui::popup::popup_below_widget(ui, popup_id, &response, |ui| {
                ui.set_min_width(100.0);
                ui.label(
                    RichText::new("Value is currently invalid")
                        .color(Color32::RED)
                        .strong(),
                );
            });

            if validate_value(state.edit_string.as_str()) {
                ui.memory_mut(egui::Memory::close_popup);
                if ui.input(|v| v.key_pressed(Key::Enter)) {
                    set(&mut get_set_value, state.edit_string.clone());
                    ui.memory_mut(|v| v.surrender_focus(kb_edit_id));
                    ui.data_mut(|d| d.remove::<State>(state_id));
                    return response;
                }
            } else {
                ui.memory_mut(|v| v.open_popup(popup_id));
                ui.memory_mut(|v| v.request_focus(kb_edit_id));
            }
            state.store(ui.ctx(), state_id);
            response
        } else {
            let mut s = old_value.as_str();
            let button = TextEdit::singleline(&mut s).frame(frame);

            let response = ui.add(button).on_hover_cursor(CursorIcon::Text);

            if response.double_clicked() {
                ui.memory_mut(|v| v.request_focus(kb_edit_id));
                ui.data_mut(|d| d.remove::<State>(state_id));
            }
            response
        };

        let value = get(&mut get_set_value);
        response.changed = value != old_value;

        response.widget_info(|| WidgetInfo::text_edit(old_value.as_str(), value.as_str()));
        response
    }
}

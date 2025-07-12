//! Copyright © 2022-2025 ChefKiss Inc. Licensed under the Thou Shalt Not Profit License version 1.5.
//! See LICENSE for details.

use egui::{
    Color32, Context, CursorIcon, Id, Key, PopupCloseBehavior, Response, RichText, TextEdit,
    TextStyle, Ui, Widget, WidgetInfo,
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

impl Widget for ClickableTextEdit<'_> {
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
                    .desired_width(f32::INFINITY)
                    .id(kb_edit_id)
                    .font(TextStyle::Monospace),
            );

            egui::Popup::from_response(&response)
                .layout(egui::Layout::top_down_justified(egui::Align::LEFT))
                .open_memory(None)
                .close_behavior(PopupCloseBehavior::IgnoreClicks)
                .id(popup_id)
                .align(egui::RectAlign::BOTTOM_START)
                .width(response.rect.width())
                .show(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(RichText::new("Invalid").color(Color32::RED).strong());
                });

            if validate_value(state.edit_string.as_str()) {
                egui::Popup::close_id(ui.ctx(), popup_id);
                if ui.input(|v| v.key_pressed(Key::Enter)) {
                    set(&mut get_set_value, state.edit_string.clone());
                    ui.memory_mut(|v| v.surrender_focus(kb_edit_id));
                    ui.data_mut(|d| d.remove::<State>(state_id));
                    return response;
                }
            } else {
                egui::Popup::open_id(ui.ctx(), popup_id);
                ui.memory_mut(|v| v.request_focus(kb_edit_id));
            }
            state.store(ui.ctx(), state_id);
            response
        } else {
            let mut s = old_value.as_str();
            let response = ui
                .add(
                    TextEdit::singleline(&mut s)
                        .desired_width(f32::INFINITY)
                        .frame(frame),
                )
                .on_hover_cursor(CursorIcon::Text);

            if response.double_clicked() {
                ui.memory_mut(|v| v.request_focus(kb_edit_id));
                ui.data_mut(|d| d.remove::<State>(state_id));
            }
            response
        };

        let value = get(&mut get_set_value);
        if value != old_value {
            response.mark_changed();
        }

        response.widget_info(|| {
            WidgetInfo::text_edit(ui.is_enabled(), old_value.as_str(), value.as_str(), "")
        });
        response
    }
}

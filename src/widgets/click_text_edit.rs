use egui::{
    Button, Color32, CursorIcon, Key, Response, RichText, TextEdit, TextStyle, Ui, Widget,
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

#[must_use]
pub struct ClickableTextEdit<'a> {
    get_set_value: GetSetValue<'a>,
    validate_value: ValidateValue<'a>,
    edit_string: &'a mut Option<String>,
    frame: bool,
}

impl<'a> ClickableTextEdit<'a> {
    pub fn new(
        value: &'a mut String,
        validate_value: impl 'a + FnMut(&str) -> bool,
        edit_string: &'a mut Option<String>,
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
            edit_string,
            frame,
        )
    }

    pub fn from_get_set(
        get_set_value: impl 'a + FnMut(Option<String>) -> String,
        validate_value: impl 'a + FnMut(&str) -> bool,
        edit_string: &'a mut Option<String>,
        frame: bool,
    ) -> Self {
        Self {
            get_set_value: Box::new(get_set_value),
            validate_value: Box::new(validate_value),
            edit_string,
            frame,
        }
    }
}

impl<'a> Widget for ClickableTextEdit<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            mut get_set_value,
            mut validate_value,
            edit_string,
            frame,
        } = self;

        let old_value = get(&mut get_set_value);

        let kb_edit_id = ui.auto_id_with("kb_edit");
        let popup_id = kb_edit_id.with("popup");
        let is_kb_editing = ui.memory().has_focus(kb_edit_id);

        let mut response = if is_kb_editing {
            let mut value_text = edit_string.take().unwrap_or_else(|| old_value.clone());
            let response = ui.add(
                TextEdit::singleline(&mut value_text)
                    .id(kb_edit_id)
                    .font(TextStyle::Monospace),
            );
            egui::popup::popup_below_widget(ui, popup_id, &response, |ui| {
                ui.set_min_width(100.0);
                ui.label(
                    RichText::new("Value seems to be invalid.\nPlease enter a valid value.")
                        .color(Color32::RED)
                        .strong(),
                );
            });

            if validate_value(value_text.as_str()) {
                ui.memory().close_popup();
                if ui.input().key_pressed(Key::Enter) {
                    set(&mut get_set_value, value_text.clone());
                    ui.memory().surrender_focus(kb_edit_id);
                    *edit_string = None;
                }
            } else {
                ui.memory().open_popup(popup_id);
                ui.memory().request_focus(kb_edit_id);
            }
            *edit_string = Some(value_text);
            response
        } else {
            let button = Button::new(RichText::new(old_value.as_str()).monospace())
                .wrap(false)
                .frame(frame);

            let response = ui.add(button).on_hover_cursor(CursorIcon::Text);

            if response.clicked() {
                ui.memory().request_focus(kb_edit_id);
                *edit_string = None;
            }
            response
        };

        let value = get(&mut get_set_value);
        response.changed = value != old_value;

        response.widget_info(|| WidgetInfo::text_edit(old_value.as_str(), value.as_str()));
        response
    }
}

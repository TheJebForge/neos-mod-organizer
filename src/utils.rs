use std::error::Error;
use std::str::FromStr;
use eframe::egui::{Color32, InnerResponse, Rect, Response, SelectableLabel, TextEdit, Ui, Vec2, Widget, WidgetText};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

#[inline]
pub fn place_in_middle<R>(ui: &mut Ui, desired_size: Vec2, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let position = ui.cursor().min;
    let available_size = ui.available_size();

    let offset = available_size / 2.0 - desired_size / 2.0;

    let place_position = position + offset;

    let rect = Rect::from_min_size(place_position, desired_size);

    ui.allocate_ui_at_rect(rect, add_contents)
}

#[inline]
pub fn handle_error<T, E: Error>(result: Result<T, E>, toasts: &mut Toasts) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            toasts.add(Toast {
                kind: ToastKind::Error,
                text: format!("Error\n{}", e).into(),
                options: ToastOptions::default()
                    .show_progress(true)
                    .duration_in_seconds(5.0),
            });

            None
        }
    }
}

#[inline]
pub fn selectable_value_with_size<Value: PartialEq>(
    ui: &mut Ui,
    size: impl Into<Vec2>,
    current_value: &mut Value,
    selected_value: Value,
    text: impl Into<WidgetText>,
) -> Response {
    let widget = SelectableLabel::new(*current_value == selected_value, text.into());

    let mut response = ui.add_sized(size.into(), widget);
    if response.clicked() && *current_value != selected_value {
        *current_value = selected_value;
        response.mark_changed();
    }

    response
}

pub fn text_field_with_label(ui: &mut Ui, label: impl Into<WidgetText>, width: f32, text: &mut String) -> bool {
    ui.horizontal_top(|ui| {
        let mut edit = TextEdit::singleline(text)
            .desired_width(width)
            .hint_text("Leave empty to ignore");

        let mut out = false;

        if edit.ui(ui).changed() {
            out = true;
        }

        ui.label(label);

        out
    }).inner
}

pub fn optioned_text_field_with_label(ui: &mut Ui, label: impl Into<WidgetText>, width: f32, text: &mut String, value: &mut Option<String>) -> bool {
    ui.horizontal_top(|ui| {
        let mut edit = TextEdit::singleline(text)
            .desired_width(width)
            .hint_text("Leave empty to ignore");

        let mut out = false;

        if edit.ui(ui).changed() {
            out = true;
            if text.is_empty() {
                *value = None;
            } else {
                *value = Some(text.clone());
            }
        }

        ui.label(label);

        out
    }).inner
}

pub fn validation_text_field_with_label<T: FromStr>(ui: &mut Ui, label: impl Into<WidgetText>, width: f32, text: &mut String, value: &mut Option<T>) -> bool {
    ui.horizontal_top(|ui| {
        let val = text.parse::<T>();

        let mut edit = TextEdit::singleline(text)
            .desired_width(width)
            .hint_text("Leave empty to ignore");

        if val.is_err() {
            edit = edit.text_color(Color32::from_rgba_premultiplied(225, 50, 50, 255));
        }

        let mut out = false;

        if edit.ui(ui).changed() {
            *value = text.parse::<T>().ok();
            out = true;
        }

        ui.label(label);

        out
    }).inner
}

pub fn find_filename_from_url(url: &str, ends_with: &str) -> Option<String> {
    if !url.ends_with(ends_with) {
        return None;
    }

    let index = url.rfind('/')?;

    if index + 1 >= url.len() {
        return None;
    }

    Some(url[(index + 1)..].to_string())
}
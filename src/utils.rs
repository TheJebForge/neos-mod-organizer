use std::error::Error;
use std::fs::FileType;
use std::io;
use std::ops::{Add, Mul, Sub};
use std::path::{Component, Path, PathBuf, StripPrefixError};
use std::str::FromStr;
use async_recursion::async_recursion;
use eframe::egui::{Color32, InnerResponse, Rect, Response, SelectableLabel, TextEdit, Ui, Vec2, Widget, WidgetText};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use sha2::{Sha256, Digest};
use sha2::digest::FixedOutput;
use tokio::fs;
use tokio::fs::File;

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

#[async_recursion::async_recursion]
pub async fn get_all_files_of_extension(location: PathBuf, extensions: &[&str]) -> Result<Vec<PathBuf>, io::Error> {
    let mut files = vec![];

    let mut directory = fs::read_dir(location).await?;

    while let Some(entry) = directory.next_entry().await? {
        let entry_type = entry.file_type().await?;
        let entry_path = entry.path();

        if entry_type.is_dir() {
            files.extend(get_all_files_of_extension(entry_path.clone(), extensions).await?);
        } else {
            if let Some(extension) = entry_path.extension() {
                let lossy_extension = extension.to_string_lossy().to_string();

                if extensions.iter().any(|x| lossy_extension == *x) {
                    files.push(entry_path);
                }
            }
        }
    }

    Ok(files)
}

pub async fn sha256_file(path: impl AsRef<Path>) -> Result<String, io::Error> {
    let data = fs::read(path).await?;

    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    Ok(hex::encode(hash))
}

pub fn append_relative_path(target: &mut PathBuf, path: impl AsRef<Path>) -> Result<(), StripPrefixError> {
    let path = path.as_ref();

    if path.has_root() {
        target.push(path.strip_prefix(Component::RootDir)?);
    } else {
        target.push(path);
    }

    Ok(())
}

#[derive(Copy, Clone, Debug)]
pub struct Colorf32 {
    r: f32,
    g: f32,
    b: f32,
    a: f32
}

impl From<Color32> for Colorf32 {
    fn from(value: Color32) -> Self {
        Self {
            r: value.r() as f32 / 255.0,
            g: value.g() as f32 / 255.0,
            b: value.b() as f32 / 255.0,
            a: value.a() as f32 / 255.0,
        }
    }
}

impl From<Colorf32> for Color32 {
    fn from(value: Colorf32) -> Self {
        Color32::from_rgba_premultiplied(
            (value.r.min(1.0).max(0.0) * 255.0) as u8,
            (value.g.min(1.0).max(0.0) * 255.0) as u8,
            (value.b.min(1.0).max(0.0) * 255.0) as u8,
            (value.a.min(1.0).max(0.0) * 255.0) as u8
        )
    }
}

impl Mul<f32> for Colorf32 {
    type Output = Colorf32;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl Mul<Colorf32> for Colorf32 {
    type Output = Colorf32;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
            a: self.a * rhs.a,
        }
    }
}

impl Add<Colorf32> for Colorf32 {
    type Output = Colorf32;

    fn add(self, rhs: Colorf32) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

impl Sub<Colorf32> for Colorf32 {
    type Output = Colorf32;

    fn sub(self, rhs: Colorf32) -> Self::Output {
        Self {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
            a: self.a - rhs.a,
        }
    }
}

pub fn lerp_color(a: &Color32, b: &Color32, t: f32) -> Color32 {
    let a = Colorf32::from(*a);
    let b = Colorf32::from(*b);
    let t = t.min(1.0).max(0.0);

    (a + (b - a) * t).into()
}
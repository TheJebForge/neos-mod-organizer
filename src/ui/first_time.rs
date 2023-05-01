use std::path::PathBuf;
use eframe::egui::{Align, Align2, Button, CentralPanel, Context, Label, Layout, RichText, TopBottomPanel, Vec2, Widget};
use egui_file::{FileDialog};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use crate::config::Config;
use crate::manager::validate_path;
use crate::utils::place_in_middle;

#[derive(Default)]
pub struct FirstTimeState {
    pub neos_path_picker: Option<PathBuf>,
    pub neos_path: String,
    pub picker_dialog: Option<FileDialog>,
    pub config: Option<Config>
}

pub fn first_time_ui(state: &mut FirstTimeState, ctx: &Context, toasts: &mut Toasts) -> Option<Config> {
    TopBottomPanel::top("top")
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                Label::new(RichText::from("First Time Setup").heading().size(30.0)).ui(ui)
            })
        });

    let path = TopBottomPanel::bottom("bottom")
        .show_separator_line(false)
        .min_height(40.0)
        .show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::Max).with_main_justify(true), |ui| {
                ui.set_enabled(!state.neos_path.is_empty());

                if Button::new(RichText::from("    Next    ").size(14.0))
                    .ui(ui).clicked() {
                    let path = state.neos_path.clone().into();

                    if validate_path(&path) {
                        return Some(path);
                    } else {
                        toasts.add(Toast {
                            kind: ToastKind::Error,
                            text: "NeosVR installation is invalid, please choose the actual installation of NeosVR\nOr if you can't find it, reinstall it either using Standalone launcher or Steam".into(),
                            options: ToastOptions::default()
                                .duration_in_seconds(5.0)
                                .show_progress(true),
                        });
                    }
                }

                None
            })
        });

    if path.inner.inner.is_some() {
        let path = path.inner.inner.unwrap();

        let config = if let Some(mut config) = state.config.clone() {
            config.neos_location = path;

            config
        } else {
            Config {
                neos_location: path,
                launch_options: Default::default(),
            }
        };

        return Some(config);
    }

    CentralPanel::default()
        .show(ctx, |ui| {
            place_in_middle(ui, Vec2::new(330.0, 60.0), |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Specify path to Neos.exe");

                    ui.add_space(5.0);

                    ui.horizontal_top(|ui| {
                        ui.text_edit_singleline(&mut state.neos_path);

                        if Button::new("Pick Path")
                            .min_size(Vec2::new(0.0, 20.0))
                            .ui(ui).clicked() {

                            let mut dialog = FileDialog::open_file(state.neos_path_picker.clone())
                                .filter(Box::new(|path| path.ends_with("Neos.exe")))
                                .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
                                .resizable(false)
                                .show_rename(false)
                                .show_new_folder(false);

                            dialog.open();

                            state.picker_dialog = Some(dialog);
                        }
                    })
                });
            });

            if let Some(dialog) = &mut state.picker_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(file) = dialog.path() {
                        state.neos_path = file.to_string_lossy().to_string();
                        state.neos_path_picker = Some(file);
                    }
                }
            }
        });

    None
}
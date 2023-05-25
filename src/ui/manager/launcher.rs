use std::sync::Arc;
use arc_swap::ArcSwap;
use dirs::desktop_dir;
use eframe::egui::{Align2, Button, CollapsingHeader, Color32, ComboBox, Context, Response, RichText, TextEdit, Ui, Vec2, Widget};
use egui_file::FileDialog;
use egui_toast::Toasts;
use strum::IntoEnumIterator;
use tokio::sync::mpsc::Sender;
use crate::config::Config;
use crate::launch::{CinematicTemporalAntiAliasing, Device, DroneCamera, JoinOptions, LaunchOptions, WindowType};
use crate::manager::ManagerCommand;
use crate::ui::manager::UIManagerState;
use crate::utils::{handle_error, optioned_text_field_with_label, text_field_with_label, validation_text_field_with_label};

fn mark_changed(state: &mut LauncherState, expr: bool) {
    if expr {
        state.cached_launch_options.1 = true;
    }
}

#[derive(Default)]
pub struct LauncherState {
    pub(crate) cached_launch_options: (LaunchOptions, bool),
    shortcut_dialog: Option<FileDialog>,
    pub(crate) enable_owo_str: String,
    pub(crate) resolution_width_str: String,
    pub(crate) resolution_height_str: String,
    pub(crate) bootstrap: String,
    pub(crate) data_path_str: String,
    pub(crate) cache_path_str: String,
    pub(crate) watchdog_str: String,
    pub(crate) config_str: String,
    pub(crate) temporal_edge_power_str: String,
    pub(crate) aptive_sharpness_str: String,
    pub(crate) enable_ctaa: bool,
    data_path_dialog: Option<FileDialog>,
    cache_path_dialog: Option<FileDialog>,
}

pub fn launcher_ui(state: &mut UIManagerState, config: &Arc<ArcSwap<Config>>, ui: &mut Ui, ctx: &Context, toasts: &mut Toasts, command: &Sender<ManagerCommand>) {
    let launcher_state = &mut state.launcher_state;

    let resp = ComboBox::from_label("Device to launch for")
        .selected_text(launcher_state.cached_launch_options.0.device.to_string())
        .width(200.0)
        .show_ui(ui, |ui| {
            for variant in Device::iter() {
                let label = variant.to_string();
                ui.selectable_value(&mut launcher_state.cached_launch_options.0.device, variant, label);
            }
        }).inner;
    mark_changed(launcher_state, resp.is_some());

    ui.add_space(5.0);

    if Button::new(RichText::from("    Launch Neos").size(40.0))
        .min_size(Vec2::new(300.0, 100.0))
        .ui(ui)
        .clicked() {

        save_launch_options(config, launcher_state.cached_launch_options.0.clone());
        handle_error(command.blocking_send(ManagerCommand::SaveConfig), toasts);

        launcher_state.cached_launch_options.1 = false;
        handle_error(command.blocking_send(ManagerCommand::LaunchNeos), toasts);
    }

    if Button::new("                                  Make Shortcut")
        .min_size(Vec2::new(300.0, 20.0))
        .ui(ui)
        .clicked() {
        save_launch_options(config, launcher_state.cached_launch_options.0.clone());
        handle_error(command.blocking_send(ManagerCommand::SaveConfig), toasts);

        launcher_state.cached_launch_options.1 = false;

        let mut dialog = FileDialog::save_file(desktop_dir())
            .filter(Box::new(|path| path.ends_with(".lnk")))
            .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
            .resizable(false)
            .show_rename(false);

        dialog.open();

        launcher_state.shortcut_dialog = Some(dialog);
    }

    ui.add_space(7.5);

    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.use_mods, "Use mods").changed();
    mark_changed(launcher_state, resp);

    ui.add_space(7.5);

    CollapsingHeader::new("Display Options")
        .default_open(false)
        .show(ui, |ui| {
            let resp = ComboBox::from_label("Display Mode")
                .selected_text(launcher_state.cached_launch_options.0.display_mode.to_string())
                .width(200.0)
                .show_ui(ui, |ui| {
                    for variant in WindowType::iter() {
                        let label = variant.to_string();
                        ui.selectable_value(&mut launcher_state.cached_launch_options.0.display_mode, variant, label);
                    }
                }).inner;
            mark_changed(launcher_state, resp.is_some());

            let resp = validation_text_field_with_label(
                ui,
                "Screen width",
                200.0,
                &mut launcher_state.resolution_width_str,
                &mut launcher_state.cached_launch_options.0.resolution_width
            );
            mark_changed(launcher_state, resp);

            let resp = validation_text_field_with_label(
                ui,
                "Screen height",
                200.0,
                &mut launcher_state.resolution_height_str,
                &mut launcher_state.cached_launch_options.0.resolution_height
            );
            mark_changed(launcher_state, resp);
        });

    CollapsingHeader::new("Data Path Options")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let mut edit = TextEdit::singleline(&mut launcher_state.data_path_str)
                    .desired_width(200.0)
                    .hint_text("Leave empty to ignore");

                if edit.ui(ui).changed() {
                    if launcher_state.data_path_str.is_empty() {
                        launcher_state.cached_launch_options.0.data_path = None;
                    } else {
                        launcher_state.cached_launch_options.0.data_path = launcher_state.data_path_str.parse().ok();
                    }
                    launcher_state.cached_launch_options.1 = true;
                }

                if ui.button(" Pick location ").clicked() {
                    let mut dialog = FileDialog::select_folder(launcher_state.cached_launch_options.0.data_path.clone())
                        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
                        .resizable(false)
                        .show_rename(false);

                    dialog.open();

                    launcher_state.data_path_dialog = Some(dialog);
                }

                ui.label("Data folder path");
            });

            ui.horizontal_top(|ui| {
                let mut edit = TextEdit::singleline(&mut launcher_state.cache_path_str)
                    .desired_width(200.0)
                    .hint_text("Leave empty to ignore");

                if edit.ui(ui).changed() {
                    if launcher_state.cache_path_str.is_empty() {
                        launcher_state.cached_launch_options.0.cache_path = None;
                    } else {
                        launcher_state.cached_launch_options.0.cache_path = launcher_state.cache_path_str.parse().ok();
                    }
                    launcher_state.cached_launch_options.1 = true;
                }

                if ui.button(" Pick location ").clicked() {
                    let mut dialog = FileDialog::select_folder(launcher_state.cached_launch_options.0.cache_path.clone())
                        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
                        .resizable(false)
                        .show_rename(false);

                    dialog.open();

                    launcher_state.cache_path_dialog = Some(dialog);
                }

                ui.label("Cache folder path");
            });
        });

    CollapsingHeader::new("Misc Options")
        .default_open(false)
        .show(ui, |ui| {
            let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.invisible, "Autoset status to Invisible").clicked();
            mark_changed(launcher_state, resp);

            let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.do_not_auto_load_cloud_home, "Don't Auto Open Cloud Home").clicked();
            mark_changed(launcher_state, resp);

            let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.skip_intro_tutorial, "Skip Intro Tutorial").clicked();
            mark_changed(launcher_state, resp);
        });

    CollapsingHeader::new("Advanced")
        .default_open(false)
        .show(ui, |ui| {
            CollapsingHeader::new("Repair Options")
                .default_open(false)
                .show(ui, |ui| {
                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.delete_unsynced_cloud_records, "Delete unsynced cloud records").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.force_sync_conflicting_cloud_records, "Force sync conflicting cloud records").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.repair_database, "Repair database").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.reset_dash, "Reset Dash").clicked();
                    mark_changed(launcher_state, resp);
                });

            CollapsingHeader::new("OWO Haptic vest")
                .default_open(false)
                .show(ui, |ui| {
                    let resp = optioned_text_field_with_label(ui, "OWO Vest IP address (enables if specified)", 200.0, &mut launcher_state.enable_owo_str, &mut launcher_state.cached_launch_options.0.enable_owo);
                    mark_changed(launcher_state, resp);
                });

            CollapsingHeader::new("Join Options")
                .default_open(false)
                .show(ui, |ui| {
                    let resp = ComboBox::from_label("Auto Join")
                        .selected_text(match launcher_state.cached_launch_options.0.auto_join {
                            JoinOptions::None => "None",
                            JoinOptions::JoinAuto => "Join Auto",
                            JoinOptions::Join(_) => "Join URL",
                            JoinOptions::Open(_) => "Open URL",
                        })
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut launcher_state.cached_launch_options.0.auto_join, JoinOptions::None, "None");
                            ui.selectable_value(&mut launcher_state.cached_launch_options.0.auto_join, JoinOptions::JoinAuto, "Join Auto");
                            ui.selectable_value(&mut launcher_state.cached_launch_options.0.auto_join, JoinOptions::Join(format!("")), "Join URL");
                            ui.selectable_value(&mut launcher_state.cached_launch_options.0.auto_join, JoinOptions::Open(format!("")), "Open URL");
                        }).inner;
                    mark_changed(launcher_state, resp.is_some());

                    let resp = match &mut launcher_state.cached_launch_options.0.auto_join {
                        JoinOptions::None => false,
                        JoinOptions::JoinAuto => false,
                        JoinOptions::Join(url) => text_field_with_label(ui, "URL", 200.0, url),
                        JoinOptions::Open(url) => text_field_with_label(ui, "URL", 200.0, url),
                    };
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.announce_home_on_lan, "Announce home on LAN").changed();
                    mark_changed(launcher_state, resp);

                    let resp = text_field_with_label(ui, "Bootstrap class", 200.0, &mut launcher_state.bootstrap);
                    mark_changed(launcher_state, resp);
                });

            CollapsingHeader::new("Networking Options")
                .default_open(false)
                .show(ui, |ui| {
                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.force_lan, "Force LAN Only").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.force_relay, "Force Relay").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.use_local_cloud, "Use Local Cloud").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.use_staging_cloud, "Use Staging Cloud").clicked();
                    mark_changed(launcher_state, resp);
                });

            CollapsingHeader::new("Drone Camera Options")
                .default_open(false)
                .show(ui, |ui| {
                    let resp = ComboBox::from_label("Drone Camera Presest")
                        .selected_text(launcher_state.cached_launch_options.0.drone_camera.to_string())
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            for variant in DroneCamera::iter() {
                                let label = variant.to_string();
                                ui.selectable_value(&mut launcher_state.cached_launch_options.0.drone_camera, variant, label);
                            }
                        }).inner;
                    mark_changed(launcher_state, resp.is_some());

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.use_neos_camera, "Use Neos Camera").clicked();
                    mark_changed(launcher_state, resp);
                });

            CollapsingHeader::new("Avatar Builder")
                .default_open(false)
                .show(ui, |ui| {
                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.force_no_voice, "Force No Voice").clicked();
                    mark_changed(launcher_state, resp);
                });

            CollapsingHeader::new("Post Processing Options")
                .default_open(false)
                .show(ui, |ui| {
                    let resp = ui.checkbox(&mut launcher_state.enable_ctaa, "Enable Cinematic Temporal Anti-Aliasing").clicked();
                    mark_changed(launcher_state, resp);
                    if resp {
                        launcher_state.cached_launch_options.0.ctaa = if launcher_state.enable_ctaa {
                            Some(CinematicTemporalAntiAliasing::default())
                        } else {
                            None
                        }
                    }

                    if let Some(ctaa) = &mut launcher_state.cached_launch_options.0.ctaa {
                        let resp = validation_text_field_with_label(ui, "Temporal Edge Power", 200.0, &mut launcher_state.temporal_edge_power_str, &mut ctaa.temporal_edge_power);
                        if resp { launcher_state.cached_launch_options.1 = true; }

                        let resp = validation_text_field_with_label(ui, "Aptive Sharpness", 200.0, &mut launcher_state.aptive_sharpness_str, &mut ctaa.aptive_sharpness);
                        if resp { launcher_state.cached_launch_options.1 = true; }

                        let resp = ui.checkbox(&mut ctaa.sharpness_enabled, "Sharpness Enabled").clicked();
                        if resp { launcher_state.cached_launch_options.1 = true; }
                    }
                });

            CollapsingHeader::new("Misc Options")
                .default_open(false)
                .show(ui, |ui| {
                    let resp = validation_text_field_with_label(ui, "Watchdog path", 200.0, &mut launcher_state.watchdog_str, &mut launcher_state.cached_launch_options.0.watchdog);
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.kiosk, "Kiosk").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.no_ui, "No UI").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.force_intro_tutorial, "Force Intro Tutorial").clicked();
                    mark_changed(launcher_state, resp);

                    let resp = validation_text_field_with_label(ui, "Config path", 200.0, &mut launcher_state.config_str, &mut launcher_state.cached_launch_options.0.config);
                    mark_changed(launcher_state, resp);

                    let resp = ui.checkbox(&mut launcher_state.cached_launch_options.0.force_reticle_above_horizon, "Force Reticle Above Horizon").clicked();
                    mark_changed(launcher_state, resp);
                });

            ui.add_space(1.0);

            ui.hyperlink_to(RichText::from("Explanation to these options can be found on Neos Wiki").size(12.0), "https://wiki.neos.com/Command_Line_Arguments");
        });

    ui.add_space(7.5);

    ui.label("Make sure to save changes if you want launch options to persist,\nlaunching the game does save launch options");

    if ui.add_enabled(launcher_state.cached_launch_options.1, Button::new(" Save changes ")).clicked() {
        save_launch_options(config, launcher_state.cached_launch_options.0.clone());
        handle_error(command.blocking_send(ManagerCommand::SaveConfig), toasts);

        launcher_state.cached_launch_options.1 = false;
    }
}

pub fn launcher_dialog(state: &mut UIManagerState, ctx: &Context, toasts: &mut Toasts, command: &Sender<ManagerCommand>) {
    if let Some(dialog) = &mut state.launcher_state.shortcut_dialog {
        if dialog.show(ctx).selected() {
            if let Some(file) = dialog.path() {
                handle_error(command.blocking_send(ManagerCommand::CreateShortcut(file.with_extension("lnk"))), toasts);
            }
        }
    }

    if let Some(dialog) = &mut state.launcher_state.data_path_dialog {
        if dialog.show(ctx).selected() {
            if let Some(folder) = dialog.path() {
                state.launcher_state.data_path_str = folder.to_string_lossy().to_string();
                state.launcher_state.cached_launch_options.0.data_path = Some(folder);
                state.launcher_state.cached_launch_options.1 = true;
            }
        }
    }

    if let Some(dialog) = &mut state.launcher_state.cache_path_dialog {
        if dialog.show(ctx).selected() {
            if let Some(folder) = dialog.path() {
                state.launcher_state.cache_path_str = folder.to_string_lossy().to_string();
                state.launcher_state.cached_launch_options.0.cache_path = Some(folder);
                state.launcher_state.cached_launch_options.1 = true;
            }
        }
    }
}

pub fn save_launch_options(config: &Arc<ArcSwap<Config>>, launch_options: LaunchOptions) {
    let mut config_str = config.load().as_ref().clone();

    config_str.launch_options = launch_options;

    config.swap(Arc::new(config_str));
}
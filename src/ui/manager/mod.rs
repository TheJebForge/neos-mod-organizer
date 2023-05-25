mod launcher;
mod tests;
mod mod_list;

use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use arc_swap::ArcSwap;
use eframe::egui::{Button, CentralPanel, CollapsingHeader, Color32, Context, RichText, ScrollArea, SidePanel, Style, Vec2};
use eframe::egui::panel::Side;
use eframe::egui::WidgetType::SelectableLabel;
use egui_file::FileDialog;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::TryRecvError;
use crate::config::Config;
use crate::install::ModMap;
use crate::launch::{Device, LaunchOptions};
use crate::manager::{ManagerCommand, ManagerEvent};
use crate::manifest::GlobalModList;
use crate::ui::manager::launcher::{launcher_dialog, launcher_ui, LauncherState};
use crate::ui::manager::mod_list::{mod_list_ui, ModListState};
use crate::ui::manager::tests::{test_ui, TestState};
use crate::utils::{handle_error, selectable_value_with_size};

pub struct UIManagerState {
    pub(crate) current_tab: ManagerTabs,
    pub(crate) launcher_state: LauncherState,
    pub(crate) mod_list_state: ModListState,
    pub(crate) test_state: TestState,
    pub(crate) manifest_mods: GlobalModList,
    pub(crate) mod_list: ModMap
}

fn handle_events(state: &mut UIManagerState, toasts: &mut Toasts, event_r: &mut Receiver<ManagerEvent>) {
    match event_r.try_recv() {
        Ok(val) => {
            match val {
                ManagerEvent::LaunchOptionsState(options) => {
                    state.launcher_state.enable_owo_str = options.enable_owo.clone().unwrap_or_else(|| "".to_string());
                    state.launcher_state.resolution_width_str = options.resolution_width.clone().map_or_else(|| "".to_string(), |x| x.to_string());
                    state.launcher_state.resolution_height_str = options.resolution_height.clone().map_or_else(|| "".to_string(), |x| x.to_string());
                    state.launcher_state.bootstrap = options.bootstrap.clone().unwrap_or_else(|| "".to_string());
                    state.launcher_state.data_path_str = options.data_path.clone().map_or_else(|| "".to_string(), |x| x.to_string_lossy().to_string());
                    state.launcher_state.cache_path_str = options.cache_path.clone().map_or_else(|| "".to_string(), |x| x.to_string_lossy().to_string());
                    state.launcher_state.watchdog_str = options.watchdog.clone().map_or_else(|| "".to_string(), |x| x.to_string_lossy().to_string());
                    state.launcher_state.config_str = options.config.clone().map_or_else(|| "".to_string(), |x| x.to_string_lossy().to_string());
                    state.launcher_state.enable_ctaa = options.ctaa.is_some();
                    state.launcher_state.temporal_edge_power_str = options.ctaa.as_ref().map_or_else(|| "".to_string(), |x| x.temporal_edge_power.as_ref().map_or_else(|| "".to_string(), |x| x.to_string()));
                    state.launcher_state.aptive_sharpness_str = options.ctaa.as_ref().map_or_else(|| "".to_string(), |x| x.aptive_sharpness.as_ref().map_or_else(|| "".to_string(), |x| x.to_string()));
                    state.launcher_state.cached_launch_options = (options, false);
                }

                ManagerEvent::Error(error) => {
                    toasts.add(Toast {
                        kind: ToastKind::Error,
                        text: format!("Manager error\n{}", error).into(),
                        options: ToastOptions::default()
                            .show_progress(true)
                            .duration_in_seconds(30.0),
                    });
                }

                ManagerEvent::ModMapChanged(map) => {
                    state.mod_list = map;
                }

                ManagerEvent::Notification(kind, message) => {
                    toasts.add(Toast {
                        kind,
                        text: message.into(),
                        options: ToastOptions::default()
                            .show_progress(true)
                            .duration_in_seconds(5.0)
                    });
                }

                ManagerEvent::LongNotification(kind, message) => {
                    toasts.add(Toast {
                        kind,
                        text: message.into(),
                        options: ToastOptions::default()
                            .show_progress(true)
                            .duration_in_seconds(30.0)
                    });
                }
            }
        }
        Err(err) => {
            match err {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => {
                    panic!("Manager is dead!")
                }
            }
        }
    }
}

#[derive(PartialEq)]
pub enum ManagerTabs {
    Launcher,
    Updates,
    ModLoader,
    InstalledMods,
    GetMods,
    Settings
}

pub fn manager_ui(state: &mut UIManagerState, config: &Arc<ArcSwap<Config>>, ctx: &Context, toasts: &mut Toasts, command: &Sender<ManagerCommand>, event: &mut Receiver<ManagerEvent>, alt_style: Arc<Style>) {
    handle_events(state, toasts, event);

    let normal = ctx.style();
    ctx.set_style(alt_style);

    SidePanel::new(Side::Left, "navbar")
        .exact_width(200.0)
        .resizable(false)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                let size = Vec2::new(200.0, 40.0);
                let text_size = 16.0;

                let names = [
                    (ManagerTabs::Launcher, "🚀 Launcher"),
                    (ManagerTabs::Updates, "↻ Updates"),
                    (ManagerTabs::ModLoader, "Ｎ Neos Mod Loader"),
                    (ManagerTabs::InstalledMods, "📦 Installed Mods"),
                    (ManagerTabs::GetMods, "⬇ Get More Mods"),
                    (ManagerTabs::Settings, "🛠 Settings")
                ];

                for (value, name) in names {
                    selectable_value_with_size(
                        ui,
                        size,
                        &mut state.current_tab,
                        value,
                        RichText::new(name).size(text_size)
                    );
                }
            })
        });

    ctx.set_style(normal);

    CentralPanel::default()
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    match state.current_tab {
                        ManagerTabs::Launcher => {
                            launcher_ui(state, config, ui, ctx, toasts, command);
                        }
                        ManagerTabs::Updates => {
                            ui.heading("Updates here");
                        }
                        ManagerTabs::ModLoader => {
                            ui.heading("modloader");
                        }
                        ManagerTabs::InstalledMods => {
                            mod_list_ui(state, config, ui, ctx, toasts, command);
                        }
                        ManagerTabs::GetMods => {}
                        ManagerTabs::Settings => {
                            CollapsingHeader::new("Tests")
                                .show(ui, |ui| {
                                    test_ui(state, ui, toasts, command, event);
                                });
                        }
                    }
                });

        });

    launcher_dialog(state, ctx, toasts, command);
}
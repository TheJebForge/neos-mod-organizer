#![windows_subsystem = "windows"]

mod manager;
mod config;
mod ui;
mod utils;
mod launch;
mod manifest;
mod version;
mod install;
mod resolver;

#[cfg(test)]
mod tests;


use std::path::{Component, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use arc_swap::ArcSwap;
use eframe::{App, CreationContext, Frame, NativeOptions, run_native};
use eframe::egui::{Align2, CentralPanel, Color32, Context, Direction, FontId, Style, TextStyle, Vec2, Window};
use eframe::egui::FontFamily;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use tokio::runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Instant;
use manager::{ManagerCommand, ManagerEvent};
use crate::config::{Config, ConfigError};
use crate::manager::{Manager, validate_path};
use crate::manifest::GlobalModList;
use crate::ui::first_time::{first_time_ui, FirstTimeState};
use crate::ui::manager::{manager_ui, ManagerTabs, UIManagerState};
use crate::ui::manager::mod_list::ModListState;
use crate::version::Version;


fn main() {
    let mut native_options = NativeOptions::default();

    native_options.min_window_size = Some(Vec2::new(900.0, 700.0));

    run_native(
        "Neos Mod Organizer",
        native_options,
        Box::new(|cc|
            Box::new(UIApp::new(cc))
        )
    ).unwrap();
}

pub struct UIApp {
    toast: Toasts,
    state: UIState,
    popup: Option<(String, Instant)>,
    manager_commander: Option<Sender<ManagerCommand>>,
    manager_events: Option<Receiver<ManagerEvent>>,
    config: Option<Arc<ArcSwap<Config>>>,

    reset_timer: Instant
}

pub enum UIState {
    FirstTime(FirstTimeState),
    Manager(UIManagerState),
    CompleteError(String)
}

impl UIApp {
    fn init_manager(&mut self, global_mods: GlobalModList) {
        let (command_s, command_r) = mpsc::channel::<ManagerCommand>(15);
        let (event_s, event_r) = mpsc::channel::<ManagerEvent>(15);

        let mut manager = Manager::new(command_r, event_s, self.config.clone().unwrap(), global_mods);

        thread::spawn(move || {
            runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(manager.run_event_loop())
        });

        self.manager_commander = Some(command_s);
        self.manager_events = Some(event_r);
    }

    fn new(cc: &CreationContext<'_>) -> Self {
        // Styles
        let mut style = (*cc.egui_ctx.style()).clone();

        style.text_styles = [
            (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(15.0, FontFamily::Monospace)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
        ].into();

        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgba_premultiplied(172, 172, 172, 255);
        style.visuals.widgets.inactive.fg_stroke.color = Color32::from_rgba_premultiplied(172, 172, 172, 255);

        style.visuals.window_shadow.extrusion = 10.0;
        style.visuals.window_shadow.color = Color32::from_rgba_premultiplied(0, 0, 0, 41);

        style.visuals.popup_shadow.extrusion = 10.0;
        style.visuals.popup_shadow.color = Color32::from_rgba_premultiplied(0, 0, 0, 41);

        cc.egui_ctx.set_style(style);

        let mut toast = Toasts::new()
            .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0))
            .direction(Direction::BottomUp);

        match Config::load_config_sync() {
            Ok(c) => {
                if validate_path(&c.neos_exe_location) {
                    let mods = GlobalModList::empty();

                    let mut instance = Self {
                        toast,
                        state: UIState::Manager(UIManagerState {
                            current_tab: ManagerTabs::Launcher,
                            launcher_state: Default::default(),
                            mod_list_state: ModListState::from_context(&cc.egui_ctx),
                            test_state: Default::default(),
                            manifest_mods: mods.clone(),
                            mod_list: Default::default(),
                        }),
                        popup: None,
                        manager_commander: None,
                        manager_events: None,
                        config: Some(Arc::new(ArcSwap::new(Arc::new(c)))),
                        reset_timer: Instant::now(),
                    };

                    instance.init_manager(mods);

                    instance
                } else {
                    toast.add(Toast {
                        kind: ToastKind::Error,
                        text: "Neos install location appears to be invalid, specify new location to Neos.exe".into(),
                        options: ToastOptions::default()
                            .duration_in_seconds(5.0)
                            .show_progress(true),
                    });

                    Self {
                        toast,
                        state: UIState::FirstTime(FirstTimeState {
                            neos_path_picker: None,
                            neos_path: "".to_string(),
                            picker_dialog: None,
                            config: Some(c),
                        }),
                        popup: None,
                        manager_commander: None,
                        manager_events: None,
                        config: None,
                        reset_timer: Instant::now(),
                    }
                }
            }
            Err(err) => {
                match err {
                    ConfigError::MissingConfig => {
                        Self {
                            toast,
                            state: UIState::FirstTime(FirstTimeState::default()),
                            popup: None,
                            manager_commander: None,
                            manager_events: None,
                            config: None,
                            reset_timer: Instant::now(),
                        }
                    }
                    _ => {
                        Self {
                            toast,
                            state: UIState::CompleteError(err.to_string()),
                            popup: None,
                            manager_commander: None,
                            manager_events: None,
                            config: None,
                            reset_timer: Instant::now(),
                        }
                    }
                }
            }
        }
    }
}

impl App for UIApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if let UIState::FirstTime(state) = &mut self.state {
            if let Some(config) = first_time_ui(state, ctx, &mut self.toast) {
                let config = Arc::new(ArcSwap::new(Arc::new(config)));

                self.config = Some(config.clone());

                let mods = GlobalModList::empty();

                match config.load().save_config_sync() {
                    Ok(_) => {
                        self.init_manager(mods.clone());
                        self.state = UIState::Manager(UIManagerState {
                            current_tab: ManagerTabs::Launcher,
                            launcher_state: Default::default(),
                            mod_list_state: ModListState::from_context(ctx),
                            test_state: Default::default(),
                            manifest_mods: mods,
                            mod_list: Default::default(),
                        });
                    }
                    Err(e) => {
                        self.toast.add(Toast {
                            kind: ToastKind::Error,
                            text: format!("Failed to save config.json\n{}", e).into(),
                            options: ToastOptions::default()
                                .duration_in_seconds(5.0)
                                .show_progress(true),
                        });
                    }
                }
            }
        } else {
            match &mut self.state {
                UIState::Manager(state) => {
                    if self.manager_events.is_some() && self.manager_commander.is_some() {
                        manager_ui(state, self.config.as_ref().unwrap(), ctx, &mut self.toast, self.manager_commander.as_ref().unwrap(), self.manager_events.as_mut().unwrap());
                    }
                }
                UIState::CompleteError(str) => {
                    CentralPanel::default()
                        .show(ctx, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.heading(format!("Unrecoverable error encountered:\n{}", str))
                            })
                        });
                }

                _ => {}
            }
        }

        self.toast.show(ctx);

        /*Window::new("Style")
            .show(ctx, |ui| {
                ctx.style_ui(ui);
            });*/
    }
}
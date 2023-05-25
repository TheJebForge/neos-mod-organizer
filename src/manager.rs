use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use eframe::egui::RichText;
use egui_toast::ToastKind;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time::{Instant, sleep};
use crate::config::Config;
use crate::install::{ActualInstall, ModFile, ModInstall, ModInstallOperations, ModMap};
use crate::launch::LaunchOptions;
use crate::manifest::{aggregate_manifests, Artifact, Category, Dependency, download_manifest, GlobalModList, Mod, ModVersion};
use crate::resolver::{find_latest_matching, resolve_install_mod, ResolveResult};
use crate::utils::{get_all_files_of_extension, sha256_file};
use crate::version::{Version, VersionReq};

pub fn validate_path(path: &PathBuf) -> bool {
    let Some(dir) = path.parent() else {
        return false;
    };

    let paths = &[
        path.clone(),
        {
            let mut lib_path = dir.to_path_buf();
            lib_path.push("Libraries");
            lib_path
        },
        {
            let mut froox_path = dir.to_path_buf();
            froox_path.push("Neos_Data");
            froox_path.push("Managed");
            froox_path.push("FrooxEngine.dll");
            froox_path
        }
    ];

    paths.into_iter().all(|path| path.exists())
}

pub struct Manager {
    command_receiver: Receiver<ManagerCommand>,
    event_sender: Sender<ManagerEvent>,
    config: Arc<ArcSwap<Config>>,
    global_mods: GlobalModList,
    install: ActualInstall
}

impl Manager {
    pub fn new(receiver: Receiver<ManagerCommand>, sender: Sender<ManagerEvent>, config: Arc<ArcSwap<Config>>, global_mods: GlobalModList) -> Self {
        let config_str = config.load_full();

        Self {
            command_receiver: receiver,
            event_sender: sender,
            config,
            global_mods: global_mods.clone(),
            install: ActualInstall::new_empty(&config_str.neos_exe_location.parent().unwrap(), global_mods),
        }
    }

    pub async fn run_event_loop(&mut self) {
        self.event_sender.send(ManagerEvent::LaunchOptionsState(self.config.load().launch_options.clone())).await.expect("Failed");

        // Get the manifest
        let time = Instant::now();
        let config = self.config.load();

        let (mods, errors) = aggregate_manifests(config.manifest_links.as_ref()).await;

        for (url, error) in errors {
            self.event_sender.send(ManagerEvent::LongNotification(
                ToastKind::Error,
                format!("Reading manifest \"{}\" failed, error:\n{}", url, error)
            )).await.ok();
        }

        let len = mods.len();
        self.global_mods.update_list(mods);

        self.event_sender.send(ManagerEvent::Notification(ToastKind::Success, format!("Downloaded info about {} mods in {}ms", len, time.elapsed().as_millis()))).await.ok();

        // Rescan mods
        let time = Instant::now();

        if let Some(_) = handle_error(self.install.rescan_mods(self.config.load_full()).await, &self.event_sender).await {
            self.event_sender.send(ManagerEvent::ModMapChanged(self.install.mod_map().clone())).await.ok();
            self.event_sender.send(ManagerEvent::Notification(ToastKind::Success, format!("Found {} mods in {}ms", self.install.mod_map().len(), time.elapsed().as_millis()))).await.ok();
        }

        loop {
            if let Some(command) = self.command_receiver.recv().await {
                match command {
                    ManagerCommand::Test => {println!("test")}
                    ManagerCommand::LaunchNeos => {
                        let mut command = self.config.load().launch_options.build_command(&self.config.load().neos_exe_location);

                        handle_error(command.spawn(), &self.event_sender).await;
                    }

                    ManagerCommand::CreateShortcut(path) => {
                        #[cfg(target_os="windows")]
                        handle_error(self.config.load().launch_options.make_shortcut(&self.config.load().neos_exe_location, path), &self.event_sender).await;
                        #[cfg(not(target_os="windows"))]
                        self.event_sender.send(ManagerEvent::Error(format!("Cannot create shortcut\nmslnk wasn't compiled due to compilation target"))).await.ok();
                    }

                    ManagerCommand::SaveConfig => {
                        handle_error(self.config.load().save_config().await, &self.event_sender).await;
                    }
                    ManagerCommand::RefreshModMap => {}
                    ManagerCommand::RefreshManifests => {}
                }
            }
        }
    }
}

#[inline]
async fn handle_error<T, E: Error>(result: Result<T, E>, sender: &Sender<ManagerEvent>) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            sender.send(ManagerEvent::Error(e.to_string())).await.ok();
            None
        }
    }
}

/// For communication from UI to Manager
#[derive(Debug)]
pub enum ManagerCommand {
    Test,
    SaveConfig,
    LaunchNeos,
    CreateShortcut(PathBuf),
    RefreshManifests,
    RefreshModMap
}

/// For communication from Manager to UI
#[derive(Debug)]
pub enum ManagerEvent {
    LaunchOptionsState(LaunchOptions),
    ModMapChanged(ModMap),
    Notification(ToastKind, String),
    LongNotification(ToastKind, String),
    Error(String)
}

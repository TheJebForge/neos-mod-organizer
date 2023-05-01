use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time::sleep;
use crate::config::Config;
use crate::launch::LaunchOptions;
use crate::manifest::download_manifest;

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
    config: Config
}

impl Manager {
    pub fn new(receiver: Receiver<ManagerCommand>, sender: Sender<ManagerEvent>, config: Config) -> Self {
        Self {
            command_receiver: receiver,
            event_sender: sender,
            config,
        }
    }

    pub async fn run_event_loop(&mut self) {
        self.event_sender.send(ManagerEvent::LaunchOptionsState(self.config.launch_options.clone())).await.expect("Failed");

        /*let manifest = download_manifest("https://raw.githubusercontent.com/neos-modding-group/neos-mod-manifest/master/manifest.json").await.unwrap();
        println!("{:#?}", manifest);*/

        loop {
            if let Some(command) = self.command_receiver.recv().await {
                match command {
                    ManagerCommand::Test => {println!("test")}
                    ManagerCommand::LaunchNeos => {
                        let mut command = self.config.launch_options.build_command(&self.config.neos_location);

                        handle_error(command.spawn(), &self.event_sender).await;
                    }
                    ManagerCommand::CreateShortcut(path) => {
                        handle_error(self.config.launch_options.make_shortcut(&self.config.neos_location, path), &self.event_sender).await;
                    }
                    ManagerCommand::SetLaunchOptions(options) => {
                        self.config.launch_options = options;
                        handle_error(self.config.save_config().await, &self.event_sender).await;
                    }
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
    SetLaunchOptions(LaunchOptions),
    LaunchNeos,
    CreateShortcut(PathBuf)
}

/// For communication from Manager to UI
#[derive(Debug)]
pub enum ManagerEvent {
    LaunchOptionsState(LaunchOptions),
    Error(String)
}

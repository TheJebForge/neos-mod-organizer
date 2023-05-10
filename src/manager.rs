use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time::sleep;
use crate::config::Config;
use crate::install::{ModFile, ModInstallOperations};
use crate::launch::LaunchOptions;
use crate::manifest::{Artifact, Category, Dependency, download_manifest, Mod, ModVersion};
use crate::resolver::{find_latest_matching, resolve_install_mod, ResolveResult};
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

        let manifest = download_manifest("https://raw.githubusercontent.com/neos-modding-group/neos-mod-manifest/master/manifest.json").await.unwrap();
        println!("{:#?}", manifest);

        let current = HashMap::from([
            (format!("dev.zkxs.neosmodloader"), vec![
                ModFile::new("dev.zkxs.neosmodloader", Mod {
                    name: format!("Test Mod 1"),
                    color: None,
                    description: format!("Testing things and how they work"),
                    authors: Default::default(),
                    source_location: None,
                    website: None,
                    tags: None,
                    category: Category::AssetImportingTweaks,
                    flags: None,
                    versions: HashMap::from([
                        (Version::from_patch(1, 12, 5), ModVersion {
                            changelog: None,
                            release_url: None,
                            neos_version_compatibility: None,
                            modloader_version_compatibility: None,
                            flags: None,
                            conflicts: None,
                            dependencies: Some(HashMap::from([
                                (format!("test.mod.dep"), Dependency {
                                    version: VersionReq::from_str("1").unwrap(),
                                })
                            ])),
                            artifacts: vec![
                                Artifact {
                                    url: "test.com/test.dll".to_string(),
                                    filename: None,
                                    sha256: "135153".to_string(),
                                    blake3: None,
                                    install_location: None,
                                }
                            ],
                        })
                    ]),
                }, Version::from_patch(1, 12, 5))
            ])
        ]);

        let op_result = resolve_install_mod(
            "dev.zkxs.neosmodloader",
            &VersionReq::from_str("*").unwrap(),
            &current,
            &manifest.mods
        );

        let ops = match op_result {
            ResolveResult::Ok(ops) => ops,
            ResolveResult::UnableToFind { mod_id, requirement } => {
                println!("Couldn't find {mod_id}@{requirement}");
                vec![]
            }
        };

        for op in ops {
            match op {
                ModInstallOperations::InstallMod { mod_id, info, version } => {
                    println!("Install {mod_id}@{version}");
                }
                ModInstallOperations::UninstallMod( file ) => {
                    println!("Uninstall {}@{}", file.mod_id, file.version.map_or_else(|| "?".to_string(), |x| x.to_string()))
                }
            }
        }

        loop {
            if let Some(command) = self.command_receiver.recv().await {
                match command {
                    ManagerCommand::Test => {println!("test")}
                    ManagerCommand::LaunchNeos => {
                        let mut command = self.config.launch_options.build_command(&self.config.neos_location);

                        handle_error(command.spawn(), &self.event_sender).await;
                    }

                    ManagerCommand::CreateShortcut(path) => {
                        #[cfg(target_os="windows")]
                        handle_error(self.config.launch_options.make_shortcut(&self.config.neos_location, path), &self.event_sender).await;
                        #[cfg(not(target_os="windows"))]
                        self.event_sender.send(ManagerEvent::Error(format!("Cannot create shortcut\nmslnk wasn't compiled due to compilation target"))).await.ok();
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

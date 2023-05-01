use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os="windows")]
use mslnk::{MSLinkError, ShellLink};

use serde::{Serialize, Deserialize};
use strum_macros::{Display, EnumIter};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LaunchOptions {
    pub device: Device,
    pub force_sr_anipal: bool,
    pub enable_owo: Option<String>,

    pub use_mods: bool,

    pub display_mode: WindowType,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,

    pub auto_join: JoinOptions,
    pub announce_home_on_lan: bool,
    pub bootstrap: Option<String>,

    pub force_lan: bool,
    pub force_relay: bool,
    pub use_local_cloud: bool,
    pub use_staging_cloud: bool,

    pub drone_camera: DroneCamera,
    pub use_neos_camera: bool,

    pub force_no_voice: bool,

    pub data_path: Option<PathBuf>,
    pub cache_path: Option<PathBuf>,

    pub delete_unsynced_cloud_records: bool,
    pub force_sync_conflicting_cloud_records: bool,
    pub repair_database: bool,

    pub ctaa: Option<CinematicTemporalAntiAliasing>,

    pub watchdog: Option<PathBuf>,
    pub load_assembly: Vec<String>,
    pub kiosk: bool,
    pub no_ui: bool,
    pub do_not_auto_load_cloud_home: bool,
    pub reset_dash: bool,
    pub skip_intro_tutorial: bool,
    pub force_intro_tutorial: bool,
    pub invisible: bool,
    pub config: Option<PathBuf>,
    pub force_reticle_above_horizon: bool,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            device: Default::default(),
            force_sr_anipal: false,
            enable_owo: None,
            use_mods: true,
            display_mode: WindowType::Auto,
            resolution_width: None,
            resolution_height: None,
            auto_join: Default::default(),
            announce_home_on_lan: false,
            bootstrap: None,
            force_lan: false,
            force_relay: false,
            use_local_cloud: false,
            use_staging_cloud: false,
            drone_camera: Default::default(),
            use_neos_camera: false,
            force_no_voice: false,
            data_path: None,
            cache_path: None,
            delete_unsynced_cloud_records: false,
            force_sync_conflicting_cloud_records: false,
            repair_database: false,
            ctaa: None,
            watchdog: None,
            load_assembly: vec![],
            kiosk: false,
            no_ui: false,
            do_not_auto_load_cloud_home: false,
            reset_dash: false,
            skip_intro_tutorial: false,
            force_intro_tutorial: false,
            invisible: false,
            config: None,
            force_reticle_above_horizon: false,
        }
    }
}

impl LaunchOptions {
    pub fn build_arguments(&self) -> Vec<(String, bool)> {
        let mut args = vec![];

        match &self.device {
            Device::AutoDetect => {}
            Device::SteamVR => args.push((format!("-SteamVR"), false)),
            Device::LegacySteamVR =>  args.push((format!("-LegacySteamVRInput"), false)),
            Device::Oculus =>  args.push((format!("-RiftTouch"), false)),
            Device::Desktop =>  args.push((format!("-Screen"), false)),
            Device::LegacyDesktop =>  args.push((format!("-LegacyScreen"), false)),
            Device::Screen360 =>  args.push((format!("-Screen360"), false)),
            Device::CameraMode =>  args.push((format!("-StaticCamera"), false)),
            Device::Camera360Mode =>  args.push((format!("-StaticCamera360"), false)),
            Device::MixedReality =>  args.push((format!("-MixedRealityCamera"), false)),
        }

        if self.force_sr_anipal {
            args.push((format!("-ForceSRAnipal"), false));
        }

        if let Some(address) = &self.enable_owo {
            args.push((format!("-EnableOWO"), false));
            args.push((address.to_string(), true));
        }

        if self.use_mods {
            args.push((format!("-LoadAssembly"), false));
            args.push((format!("Libraries\\NeosModLoader.dll"), true));
        }

        match &self.auto_join {
            JoinOptions::None => {}
            JoinOptions::JoinAuto => {
                args.push((format!("-Join"), false));
                args.push((format!("Auto"), false));
            },
            JoinOptions::Join(addr) => {
                args.push((format!("-Join"), false));
                args.push((addr.to_string(), true));
            },
            JoinOptions::Open(addr) => {
                args.push((format!("-Open"), false));
                args.push((addr.to_string(), true));
            },
        }

        if self.announce_home_on_lan {
            args.push((format!("-ForceSRAnipal"), false));
        }

        if let Some(bootstrap) = &self.bootstrap {
            args.push((format!("-Bootstrap"), false));
            args.push((bootstrap.to_string(), false));
        }

        if self.force_lan {
            args.push((format!("-ForceLANOnly"), false));
        }

        if self.force_relay {
            args.push((format!("-ForceRelay"), false));
        }

        if self.use_local_cloud {
            args.push((format!("-UseLocalCloud"), false));
        }

        if self.use_staging_cloud {
            args.push((format!("-UseStagingCloud"), false));
        }

        match &self.drone_camera {
            DroneCamera::None => {}
            DroneCamera::CameraBiggestGroup => args.push((format!("-CameraBiggestGroup"), false)),
            DroneCamera::CameraTimelapse => args.push((format!("-CameraTimelapse"), false)),
            DroneCamera::CameraStayBehind => args.push((format!("-CameraStayBehind"), false)),
            DroneCamera::CameraStayInFront => args.push((format!("-CameraStayInFront"), false)),
        }

        if self.use_neos_camera {
            args.push((format!("-UseNeosCamera"), false));
        }

        if self.force_no_voice {
            args.push((format!("-ForceNoVoice"), false));
        }

        if let Some(data_path) = &self.data_path {
            args.push((format!("-DataPath"), false));
            args.push((data_path.to_string_lossy().to_string(), true));
        }

        if let Some(cache_path) = &self.cache_path {
            args.push((format!("-CachePath"), false));
            args.push((cache_path.to_string_lossy().to_string(), true));
        }

        if self.delete_unsynced_cloud_records {
            args.push((format!("-DeleteUnsyncedCloudRecords"), false));
        }

        if self.force_sync_conflicting_cloud_records {
            args.push((format!("-ForceSyncConflictingCloudRecords"), false));
        }

        if self.repair_database {
            args.push((format!("-RepairDatabase"), false));
        }

        if let Some(ctaa) = &self.ctaa {
            args.push((format!("-ctaa"), false));

            if let Some(temporal_edge_power) = ctaa.temporal_edge_power {
                args.push((format!("-ctaaTemporalEdgePower"), false));
                args.push((format!("{}", temporal_edge_power), false));
            }

            if let Some(aptive_sharpness) = ctaa.aptive_sharpness {
                args.push((format!("-ctaaAptiveSharpness"), false));
                args.push((format!("{}", aptive_sharpness), false));
            }

            args.push((format!("-ctaaSharpnessEnabled"), false));
            args.push((format!("{}", ctaa.sharpness_enabled), false));
        }

        if let Some(watchdog) = &self.watchdog {
            args.push((format!("-Watchdog"), false));
            args.push((watchdog.to_string_lossy().to_string(), true));
        }

        for assembly in &self.load_assembly {
            args.push((format!("-LoadAssembly"), false));
            args.push((assembly.to_string(), true));
        }

        if self.kiosk {
            args.push((format!("-Kiosk"), false));
        }

        if self.no_ui {
            args.push((format!("-NoUI"), false));
        }

        if self.do_not_auto_load_cloud_home {
            args.push((format!("-DontAutoOpenCloudHome"), false));
        }

        if self.reset_dash {
            args.push((format!("-ResetDash"), false));
        }

        if self.skip_intro_tutorial {
            args.push((format!("-SkipIntroTutorial"), false));
        }

        if self.force_intro_tutorial {
            args.push((format!("-Forceintrotutorial"), false));
        }

        if self.invisible {
            args.push((format!("-Invisible"), false));
        }

        if let Some(config) = &self.config {
            args.push((format!("-Config"), false));
            args.push((config.to_string_lossy().to_string(), true));
        }

        if self.force_reticle_above_horizon {
            args.push((format!("-ForceReticleAboveHorizon"), false));
        }

        match &self.display_mode {
            WindowType::Auto => {}
            WindowType::Windowed => {
                args.push((format!("-screen-fullscreen"), false));
                args.push((format!("0"), false));
            }
            WindowType::FullScreen => {
                args.push((format!("-screen-fullscreen"), false));
                args.push((format!("1"), false));
            }
        }

        if let Some(width) = &self.resolution_width {
            args.push((format!("-screen-width"), false));
            args.push((format!("{}", width), false));
        }

        if let Some(height) = &self.resolution_height {
            args.push((format!("-screen-height"), false));
            args.push((format!("{}", height), false));
        }

        args
    }

    pub fn build_command(&self, neos_path: impl AsRef<Path>) -> Command {
        let args = self.build_arguments().into_iter()
            .map(|(arg, _)| arg)
            .collect::<Vec<String>>();

        let path = neos_path.as_ref();

        let mut command = Command::new(path.as_os_str());

        command.args(args.iter())
            .current_dir(path.parent().unwrap());

        command
    }
    
    #[cfg(target_os="windows")]
    pub fn make_shortcut(&self, neos_path: impl AsRef<Path>, shortcut_path: impl AsRef<Path>) -> Result<(), MSLinkError> {
        let neos_path = neos_path.as_ref();
        let shortcut_path = shortcut_path.as_ref();

        let args = self.build_arguments().into_iter()
            .map(|(arg, quotes)| {
                if quotes {
                    format!("\"{}\"", arg)
                } else {
                    arg
                }
            })
            .collect::<Vec<String>>();

        let arg_str = args.join(" ");

        let mut link = ShellLink::new(neos_path)?;

        link.set_working_dir(Some(neos_path.parent().unwrap().to_string_lossy().to_string()));
        link.set_name(Some(shortcut_path.file_stem().unwrap().to_string_lossy().to_string()));
        link.set_arguments(Some(arg_str));

        link.create_lnk(shortcut_path)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display, EnumIter)]
pub enum Device {
    AutoDetect,
    SteamVR,
    Oculus,
    MixedReality,
    Desktop,
    LegacySteamVR,
    LegacyDesktop,
    Screen360,
    CameraMode,
    Camera360Mode,
}

impl Default for Device {
    fn default() -> Self {
        Self::AutoDetect
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum JoinOptions {
    None,
    JoinAuto,
    Join(String),
    Open(String)
}

impl Default for JoinOptions {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display, EnumIter)]
pub enum DroneCamera {
    None,
    CameraBiggestGroup,
    CameraTimelapse,
    CameraStayBehind,
    CameraStayInFront
}

impl Default for DroneCamera {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct CinematicTemporalAntiAliasing {
    pub temporal_edge_power: Option<f32>,
    pub sharpness_enabled: bool,
    pub aptive_sharpness: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display, EnumIter)]
pub enum WindowType {
    Auto,
    Windowed,
    FullScreen
}

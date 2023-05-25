use std::error::Error;
use std::fmt::{Display, Formatter};
use std::{env, io};
use std::path::{PathBuf};
use dirs::config_dir;
use serde::{Serialize, Deserialize};
use tokio::task::{JoinError, spawn_blocking};
use crate::launch::LaunchOptions;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub neos_exe_location: PathBuf,
    #[serde(default)]
    pub launch_options: LaunchOptions,
    #[serde(default = "default_scan_locations")]
    pub scan_locations: Vec<PathBuf>,
    #[serde(default = "default_manifest_links")]
    pub manifest_links: Vec<String>
}

pub fn default_scan_locations() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Libraries"),
        PathBuf::from("/nml_libs"),
        PathBuf::from("/nml_mods")
    ]
}

pub fn default_manifest_links() -> Vec<String> {
    vec![
        format!("https://raw.githubusercontent.com/neos-modding-group/neos-mod-manifest/master/manifest.json")
    ]
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut dir = config_dir().map(|mut d| {
            d.push("neos-mod-organizer"); d
        }).unwrap_or_else(|| env::current_dir().expect("where tf am i?"));

        dir.push("config.json");

        dir
    }

    pub fn config_exists(path: &PathBuf) -> bool {
        path.try_exists().expect("Can't access config")
    }

    pub fn load_config_sync() -> Result<Config, ConfigError> {
        let path = Self::config_path();

        if !Self::config_exists(&path) {
            return Err(ConfigError::MissingConfig);
        }

        let str = std::fs::read_to_string(path)?;

        Ok(serde_json::from_str(&str)?)
    }

    pub async fn load_config() -> Result<Config, ConfigError> {
        let path = Self::config_path();

        if !Self::config_exists(&path) {
            return Err(ConfigError::MissingConfig);
        }

        let str = tokio::fs::read_to_string(path).await?;

        Ok(spawn_blocking(move || serde_json::from_str(&str)).await??)
    }

    pub fn save_config_sync(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();
        let config_folder = path.parent().unwrap().to_path_buf();

        std::fs::create_dir_all(&config_folder)?;

        Ok(std::fs::write(path, serde_json::to_string(self)?)?)
    }

    pub async fn save_config(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();
        let config_folder = path.parent().unwrap().to_path_buf();

        tokio::fs::create_dir_all(&config_folder).await?;

        Ok(tokio::fs::write(path, serde_json::to_string(self)?).await?)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingConfig,
    IOError(io::Error),
    JSONError(serde_json::Error),
    JoinError(JoinError)
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(value: serde_json::Error) -> Self {
        Self::JSONError(value)
    }
}

impl From<JoinError> for ConfigError {
    fn from(value: JoinError) -> Self {
        Self::JoinError(value)
    }
}
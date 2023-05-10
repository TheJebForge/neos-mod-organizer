use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::version::{Version, VersionReq};

pub async fn download_manifest(url: &str) -> Result<ModManifest, reqwest::Error> {
    Ok(reqwest::get(url)
        .await?
        .json()
        .await?)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModManifest {
    pub schema_version: Version,
    pub mods: HashMap<GUID, Mod>
}

pub type GUID = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub name: String,
    pub color: Option<String>,
    pub description: String,
    pub authors: HashMap<String, Author>,
    pub source_location: Option<String>,
    pub website: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Category,
    pub flags: Option<Vec<String>>,
    pub versions: HashMap<Version, ModVersion>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModVersion {
    pub changelog: Option<String>,
    #[serde(rename = "releaseURL")]
    pub release_url: Option<String>,
    pub neos_version_compatibility: Option<VersionReq>,
    pub modloader_version_compatibility: Option<VersionReq>,
    pub flags: Option<Vec<String>>,
    pub conflicts: Option<HashMap<GUID, Conflict>>,
    pub dependencies: Option<HashMap<GUID, Dependency>>,
    pub artifacts: Vec<Artifact>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub version: VersionReq
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub version: VersionReq
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub url: String,
    pub filename: Option<String>,
    pub sha256: String,
    pub blake3: Option<String>,
    pub install_location: Option<PathBuf>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub url: String,
    pub icon_url: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Category {
    #[serde(rename = "Asset Importing Tweaks")]
    AssetImportingTweaks,
    #[serde(rename = "Bug Workarounds")]
    BugWorkarounds,
    #[serde(rename = "Context Menu Tweaks")]
    ContextMenuTweaks,
    #[serde(rename = "Dash Tweaks")]
    DashTweaks,
    Developers,
    #[serde(rename = "General UI Tweaks")]
    GeneralUITweaks,
    #[serde(rename = "Hardware Integrations")]
    HardwareIntegrations,
    Inspectors,
    #[serde(rename = "Keybinds & Gestures")]
    KeybindsGestures,
    Libraries,
    LogiX,
    Memes,
    Misc,
    Optimization,
    Plugins,
    #[serde(rename = "Technical Tweaks")]
    TechnicalTweaks,
    #[serde(rename = "Visual Tweaks")]
    VisualTweaks,
    Wizards,
}
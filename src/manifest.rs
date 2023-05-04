use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::version::{Version, VersionReq};

pub async fn download_manifest(url: &str) -> Result<ModManifest, ManifestDownloadError> {
    Ok(reqwest::get(url)
        .await?
        .json()
        .await?)
}

#[derive(Debug)]
pub enum ManifestDownloadError {
    ReqwestError(reqwest::Error)
}

impl Display for ManifestDownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ManifestDownloadError {}

impl From<reqwest::Error> for ManifestDownloadError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModManifest {
    schema_version: Version,
    mods: HashMap<GUID, Mod>
}

pub type GUID = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    name: String,
    color: Option<String>,
    description: String,
    authors: HashMap<String, Author>,
    source_location: Option<String>,
    website: Option<String>,
    tags: Option<Vec<String>>,
    category: Category,
    flags: Option<Vec<String>>,
    versions: HashMap<Version, ModVersion>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModVersion {
    changelog: Option<String>,
    #[serde(rename = "releaseURL")]
    release_url: Option<String>,
    neos_version_compatibility: Option<VersionReq>,
    modloader_version_compatibility: Option<VersionReq>,
    flags: Option<Vec<String>>,
    conflicts: Option<HashMap<GUID, Conflict>>,
    dependencies: Option<HashMap<GUID, Dependency>>,
    artifacts: Vec<Artifact>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    version: VersionReq
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    version: VersionReq
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    url: String,
    filename: Option<String>,
    sha256: String,
    blake3: Option<String>,
    install_location: Option<PathBuf>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    url: String,
    icon_url: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
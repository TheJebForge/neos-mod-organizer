use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use arc_swap::ArcSwap;
use futures::future::join_all;
use serde::{Serialize, Deserialize};
use crate::version::{Version, VersionReq};

pub async fn download_manifest(url: &str) -> Result<ModManifest, reqwest::Error> {
    Ok(reqwest::get(url)
        .await?
        .json()
        .await?)
}

pub async fn aggregate_manifests(urls: &[String]) -> (ManifestMods, Vec<(String, reqwest::Error)>) {
    let mut errors = vec![];
    let mods = join_all(urls.iter().map(|x| async { (x.clone(), download_manifest(x).await) }))
        .await
        .into_iter()
        .filter_map(|(url, x)| x.map_err(|e| errors.push((url, e))).ok())
        .flat_map(|m| m.mods.into_iter())
        .collect();

    (mods, errors)
}


pub type ManifestMods = HashMap<GUID, Mod>;
/// Sha256 hash to mod_id and version
pub type ModHashTable = HashMap<String, (String, Version)>;
/// Mod_id and version to list of sha256 hashes
pub type ReverseHashTable = HashMap<(String, Version), Vec<String>>;

#[derive(Clone)]
pub struct GlobalModList {
    pub mod_list: Arc<ArcSwap<ManifestMods>>,
    pub mod_hash_table: Arc<ArcSwap<ModHashTable>>,
    pub reverse_hash_table: Arc<ArcSwap<ReverseHashTable>>,
}

impl GlobalModList {
    pub fn empty() -> Self {
        Self {
            mod_list: Arc::new(Default::default()),
            mod_hash_table: Arc::new(Default::default()),
            reverse_hash_table: Arc::new(Default::default()),
        }
    }

    pub fn from_list(manifest_mods: ManifestMods) -> Self {
        let hashtable = hashtable_from_mod_list(&manifest_mods);
        let reverse = reverse_hashtable_from_mod_list(&manifest_mods);

        Self {
            mod_list: Arc::new(ArcSwap::from(Arc::new(manifest_mods))),
            mod_hash_table: Arc::new(ArcSwap::from(Arc::new(hashtable))),
            reverse_hash_table: Arc::new(ArcSwap::from(Arc::new(reverse))),
        }
    }

    pub fn update_list(&self, manifest_mods: ManifestMods) {
        self.mod_list.swap(Arc::new(manifest_mods));
        self.recreate_tables();
    }

    pub fn recreate_tables(&self) {
        let manifest_mods = self.mod_list.load();

        let hashtable = hashtable_from_mod_list(&manifest_mods);
        let reverse = reverse_hashtable_from_mod_list(&manifest_mods);

        self.mod_hash_table.swap(Arc::new(hashtable));
        self.reverse_hash_table.swap(Arc::new(reverse));
    }
}

pub fn hashtable_from_mod_list(mod_list: &ManifestMods) -> ModHashTable {
    mod_list.iter()
        .flat_map(|(mod_id, info)| {
            info.versions.iter()
                .flat_map(|(version, version_info)| {
                    version_info.artifacts.iter()
                        .map(|a| {
                            (a.sha256.clone(), (mod_id.clone(), version.clone()))
                        })
                        .collect::<Vec<(String, (String, Version))>>()
                })
                .collect::<Vec<(String, (String, Version))>>()
        })
        .collect()
}

pub fn reverse_hashtable_from_mod_list(mod_list: &ManifestMods) -> ReverseHashTable {
    mod_list.iter()
        .flat_map(|(mod_id, info)| {
            info.versions.iter()
                .map(|(version, version_info)| {
                    let hashes = version_info.artifacts.iter()
                        .map(|a| {
                            a.sha256.clone()
                        })
                        .collect::<Vec<String>>();

                    ((mod_id.clone(), version.clone()), hashes)
                })
                .collect::<Vec<((String, Version), Vec<String>)>>()
        })
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModManifest {
    pub schema_version: Option<Version>,
    pub mods: ManifestMods
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

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
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
    #[serde(other)]
    Unknown
}
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::{io, path};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf, StripPrefixError};
use std::sync::Arc;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use crate::manifest::{GlobalModList, GUID, ManifestMods, Mod, ModVersion};
use crate::version::{Version, VersionReq};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use crate::config::Config;
use crate::utils::{append_relative_path, find_filename_from_url, get_all_files_of_extension, sha256_file};

pub type IDVersion = (String, Version);
pub type IDVersionReq = (String, VersionReq);

pub type ModMap = HashMap<GUID, HashMap<Version, ModFile>>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ModFile {
    pub files: Vec<ModFileArtifact>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ModFileArtifact {
    pub file_path: PathBuf,
    pub file_hash: String,
    pub disabled: bool,
}

impl ModFile {
    pub fn new(mod_id: &str, version: &Version, mods: &ManifestMods) -> Self {
        let files = if let Some(mod_info) = mods.get(mod_id) {
            let version_info = mod_info.versions.get(&version);

            version_info.map_or_else(|| vec![], |x| {
                x.artifacts.iter()
                    .filter_map(|x| {
                        let filename = x.filename.clone()
                            .or_else(|| find_filename_from_url(&x.url, ".dll"))?;

                        let mut location = x.install_location.clone()
                            .unwrap_or_else(|| PathBuf::from("/nml_mods"));

                        location.push(filename);

                        Some(ModFileArtifact {
                            file_path: location,
                            file_hash: x.sha256.clone(),
                            disabled: false,
                        })
                    })
                    .collect()
            })
        } else {
            vec![]
        };

        ModFile {
            files,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ModConflict {
    /// Multiple versions of a single mod are found
    VersionConflict(GUID),

    /// Mods have a direct conflict with each other
    DirectConflict {
        this: IDVersion,
        conflict_with: IDVersion,
    },

    /// If dependency for a mod is missing
    DependencyMissing {
        this: IDVersion,
        needs: IDVersionReq,
    },

    /// If a mod wasn't satisfied with found dependency
    DependencyMismatch {
        this: IDVersion,
        needs: IDVersionReq,
        found_versions: Vec<Version>
    },

    /// Not all of the mod's artifacts are installed
    IncompleteInstall {
        this: IDVersion,
        missing_file: String,
    },

    /// There's multiples of the same file
    FileConflict {
        this: IDVersion,
        already_exists: PathBuf
    }
}

#[derive(Clone, Debug)]
pub enum ModInstallOperations {
    InstallMod(IDVersion),
    UninstallMod(IDVersion)
}

#[async_trait::async_trait]
pub trait ModInstall {
    fn mod_map(&self) -> &ModMap;
    async fn perform_operations(&mut self, operations: &[ModInstallOperations]) -> Result<(), InstallError>;

    fn check_for_conflicts(&self, mods: &ManifestMods) -> Vec<ModConflict> {
        let mut conflicts = vec![];

        let map = self.mod_map();
        let mut install_files: HashSet<PathBuf> = HashSet::new();

        for (file_guid, mod_files) in map {
            if mod_files.len() > 1 { // If there's more than one version of a single mod installed, then version conflict
                conflicts.push(ModConflict::VersionConflict(file_guid.clone()));
            }

            for (file_version, file) in mod_files { // For each mod file
                if let Some(mod_info) = mods.get(file_guid) {
                    if let Some(version) = mod_info.versions.get(file_version) { // If version info is found
                        for artifact in &version.artifacts {
                            let filename = artifact.filename.clone()
                                .or_else(|| find_filename_from_url(&artifact.url, ".dll"))
                                .unwrap_or_else(|| "unknown.dll".to_string());

                            let mut filepath = artifact.install_location.clone().unwrap_or_else(|| PathBuf::from("/nml_mods"));
                            filepath.push(&filename);

                            if install_files.contains(&filepath) { // If there's already a file at the path, file conflict
                                conflicts.push(ModConflict::FileConflict {
                                    this: (file_guid.clone(), file_version.clone()),
                                    already_exists: filepath
                                })
                            } else { // If there's not, add the file path to hash set
                                install_files.insert(filepath);
                            }

                            if !file.files.iter().any(|x| x.file_hash == artifact.sha256) {
                                conflicts.push(ModConflict::IncompleteInstall {
                                    this: (file_guid.clone(), file_version.clone()),
                                    missing_file: filename,
                                })
                            }
                        }

                        if let Some(mod_dependencies) = &version.dependencies { // If there's defined dependencies for this version
                            for (dependency_guid, dependency_info) in mod_dependencies { // For each found dependency
                                if let Some(found_files) = map.get(dependency_guid) { // If dependency is installed
                                    if !found_files.iter().any(|(v, _)| { // If all versions don't match the requirement
                                        return dependency_info.version.matches(v);
                                    }) { // Report it as depedency mismatch
                                        let versions = found_files.iter()
                                            .map(|(v, _)| v.clone())
                                            .collect::<Vec<Version>>();

                                        conflicts.push(ModConflict::DependencyMismatch {
                                            this: (file_guid.clone(), file_version.clone()),
                                            needs: (dependency_guid.clone(), dependency_info.version.clone()),
                                            found_versions: versions,
                                        });
                                    }
                                } else { // If dependency wasn't installed, report it as dependency mismatch
                                    conflicts.push(ModConflict::DependencyMissing {
                                        this: (file_guid.clone(), file_version.clone()),
                                        needs: (dependency_guid.clone(), dependency_info.version.clone()),
                                    });
                                }
                            }
                        }

                        if let Some(mod_conflicts) = &version.conflicts { // If there's defined conflicts for this version
                            for (conflict_guid, conflict_info) in mod_conflicts { // For each found conflict
                                if let Some(mod_conflict) = map.get(conflict_guid) { // Check if mod is installed
                                    if let Some((conflicting_version, conflicting_file)) = mod_conflict.iter() // Check if any of the mod versions match the conflict
                                        .find(|(v, _)| {
                                            conflict_info.version.matches(v) // Check if the installed version matches the conflict conditions
                                        }) { // If true, add it as direct conflict
                                        conflicts.push(ModConflict::DirectConflict {
                                            this: (file_guid.clone(), file_version.clone()),
                                            conflict_with: (conflict_guid.clone(), conflicting_version.clone()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        conflicts
    }
}

pub struct ActualInstall {
    location: PathBuf,
    installed_mods: ModMap,
    manifest_mods: GlobalModList,
}

impl ActualInstall {
    pub fn new_empty(location: impl AsRef<Path>, global_mods: GlobalModList) -> ActualInstall {
        Self {
            location: location.as_ref().to_path_buf(),
            installed_mods: Default::default(),
            manifest_mods: global_mods,
        }
    }

    pub async fn rescan_mods(&mut self, config: Arc<Config>) -> Result<(), InstallError> {
        let install_location = self.location.clone();
        let mod_hashtable = self.manifest_mods.mod_hash_table.load();

        let mut installed = HashMap::new();

        for scan_location in &config.scan_locations {
            let mut location = install_location.clone();
            append_relative_path(&mut location, scan_location)?;

            if location.exists() {
                let files = get_all_files_of_extension(location, &["dll", "disabled"]).await?;

                for file in files {
                    let disabled = file.ends_with(".disabled");
                    let hash = sha256_file(&file).await?;

                    println!("file {} - hash: {}", file.to_string_lossy(), hash);

                    let (mod_id, version) = if let Some((mod_id, version)) = mod_hashtable.get(&hash) {
                        println!("recognized hash as {}", mod_id);
                        (mod_id.clone(), version.clone())
                    } else {
                        println!("unrecognized");
                        (
                            file.file_name().map_or_else(|| "unknown.dll".to_string(), |x| x.to_string_lossy().to_string()),
                            Version::zero()
                        )
                    };

                    installed.entry(mod_id)
                        .or_insert(HashMap::new())
                        .entry(version)
                        .or_insert(ModFile::default())
                        .files.push(
                        ModFileArtifact {
                            file_path: file,
                            file_hash: hash,
                            disabled,
                        }
                    );
                }
            }
        }

        self.installed_mods = installed;

        Ok(())
    }

    pub fn virtualize(&self) -> VirtualInstall {
        VirtualInstall {
            installed_mods: self.installed_mods.clone(),
            manifest_mods: self.manifest_mods.mod_list.load_full(),
        }
    }
}

#[async_trait::async_trait]
impl ModInstall for ActualInstall {
    fn mod_map(&self) -> &ModMap {
        &self.installed_mods
    }

    async fn perform_operations(&mut self, operations: &[ModInstallOperations]) -> Result<(), InstallError> {
        for op in operations {
            match op {
                ModInstallOperations::InstallMod((id, version)) => {
                    println!("Pretend am actually installing {}@{}", id, version)
                }
                ModInstallOperations::UninstallMod((id, version)) => {
                    println!("Pretend am actually uninstalling {}@{}", id, version)
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct VirtualInstall {
    installed_mods: ModMap,
    manifest_mods: Arc<ManifestMods>
}

impl VirtualInstall {
    pub fn new(mod_map: ModMap, manifest_mods: Arc<ManifestMods>) -> VirtualInstall {
        Self {
            installed_mods: mod_map,
            manifest_mods,
        }
    }
}

#[async_trait::async_trait]
impl ModInstall for VirtualInstall {
    fn mod_map(&self) -> &ModMap {
        &self.installed_mods
    }

    async fn perform_operations(&mut self, operations: &[ModInstallOperations]) -> Result<(), InstallError> {
        for op in operations {
            match op {
                ModInstallOperations::InstallMod ((mod_id, version))  => {
                    let file = ModFile::new(mod_id, version, &self.manifest_mods);

                    let files = self.installed_mods.entry(mod_id.clone()).or_default();

                    files.insert(version.clone(), file);
                }

                ModInstallOperations::UninstallMod((mod_id, version))  => {
                    let Some(files) = self.installed_mods.get_mut(mod_id) else {
                        return Err(InstallError::FileNotFound)
                    };

                    files.remove(version);

                    if files.len() <= 0 {
                        self.installed_mods.remove(mod_id);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum InstallError {
    /// Happens on ActualInstall if install was attempted for a file that already exists
    FileAlreadyExists,
    /// Happens when trying to uninstall a mod that already doesn't exist
    FileNotFound,
    FileError(io::Error),
    StripError(path::StripPrefixError)
}

impl Display for InstallError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for InstallError {}

impl From<io::Error> for InstallError {
    fn from(value: io::Error) -> Self {
        Self::FileError(value)
    }
}

impl From<path::StripPrefixError> for InstallError {
    fn from(value: StripPrefixError) -> Self {
        Self::StripError(value)
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use async_trait::async_trait;
use crate::manifest::{GUID, Mod, ModVersion};
use crate::version::{Version, VersionReq};
use serde::{Serialize, Deserialize};
use crate::utils::find_filename_from_url;

pub type ModMap = HashMap<GUID, Vec<ModFile>>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ModFile {
    pub files: Vec<ModFileArtifact>,
    pub mod_id: GUID,
    pub mod_info: Option<Mod>,
    pub version: Option<Version>,
    pub version_info: Option<ModVersion>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ModFileArtifact {
    pub file_path: PathBuf,
    pub file_hash: String,
}

impl ModFile {
    pub fn new(mod_id: &str, mod_info: Mod, version: Version) -> Self {
        let version_info = mod_info.versions.get(&version).cloned();
        let files = version_info.clone().map_or_else(|| vec![], |x| {
            x.artifacts.into_iter()
                .filter_map(|x| {
                    let filename = x.filename.clone()
                        .or_else(|| find_filename_from_url(&x.url, ".dll"))?;

                    let mut location = x.install_location.clone()
                        .unwrap_or_else(|| PathBuf::from("/nml_mods"));

                    location.push(filename);

                    Some(ModFileArtifact {
                        file_path: location,
                        file_hash: x.sha256,
                    })
                })
                .collect()
        });

        ModFile {
            files,
            mod_id: mod_id.to_string(),
            mod_info: Some(mod_info),
            version: Some(version),
            version_info,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ModConflict {
    /// Multiple versions of a single mod are found
    VersionConflict(GUID),

    /// Mods have a direct conflict with each other
    DirectConflict {
        this: ModFile,
        conflict_with: ModFile,
    },

    /// If dependency for a mod is missing
    DependencyMissing {
        this: ModFile,
        needs: GUID,
        needed_version: VersionReq,
    },

    /// If a mod wasn't satisfied with found dependency
    DependencyMismatch {
        this: ModFile,
        needs: GUID,
        needed_version: VersionReq,
        found_versions: Vec<Version>
    }
}

#[derive(Clone, Debug)]
pub enum ModInstallOperations {
    InstallMod {
        /// Artifact name
        mod_id: String,
        /// Info of the mod that will be installed
        info: Mod,
        /// Version of the mod to install
        version: Version,
    },
    UninstallMod(ModFile)
}

#[async_trait::async_trait]
pub trait ModInstall {
    fn mod_map(&self) -> &ModMap;
    async fn perform_operations(&mut self, operations: &[ModInstallOperations]) -> Result<(), InstallError>;

    fn check_for_conflicts(&self) -> Vec<ModConflict> {
        let mut conflicts = vec![];

        let map = self.mod_map();

        for (guid, mod_files) in map {
            if mod_files.len() > 1 { // If there's more than one version of a single mod installed, then version conflict
                conflicts.push(ModConflict::VersionConflict(guid.clone()));
            }

            for file in mod_files { // For each mod file
                if let Some(version) = &file.version_info { // If version info is found
                    if let Some(mod_dependencies) = &version.dependencies { // If there's defined dependencies for this version
                        for (dependency_guid, dependency_info) in mod_dependencies { // For each found dependency
                            if let Some(found_files) = map.get(dependency_guid) { // If dependency is installed
                                if !found_files.iter().any(|x| { // If all versions don't match the requirement
                                    if let Some(ver) = &x.version {
                                        return dependency_info.version.matches(ver);
                                    }

                                    false
                                }) { // Report it as depedency mismatch
                                    let versions = found_files.iter()
                                        .filter_map(|x| x.version.clone())
                                        .collect::<Vec<Version>>();

                                    conflicts.push(ModConflict::DependencyMismatch {
                                        this: file.clone(),
                                        needs: dependency_guid.clone(),
                                        needed_version: dependency_info.version.clone(),
                                        found_versions: versions,
                                    });
                                }
                            } else { // If dependency wasn't installed, report it as dependency mismatch
                                conflicts.push(ModConflict::DependencyMissing {
                                    this: file.clone(),
                                    needs: dependency_guid.to_string(),
                                    needed_version: dependency_info.version.clone(),
                                });
                            }
                        }
                    }

                    if let Some(mod_conflicts) = &version.conflicts { // If there's defined conflicts for this version
                        for (conflict_guid, conflict_info) in mod_conflicts { // For each found conflict
                            if let Some(mod_conflict) = map.get(conflict_guid) { // Check if mod is installed
                                if let Some(conflicting) = mod_conflict.iter() // Check if any of the mod versions match the conflict
                                    .find(|i| {
                                        if let Some(mod_version) = &i.version {
                                            conflict_info.version.matches(mod_version) // Check if the installed version matches the conflict conditions
                                        } else {
                                            true // If no version in the mod, define it as a conflict anyways
                                        }
                                    }) { // If true, add it as direct conflict
                                    conflicts.push(ModConflict::DirectConflict {
                                        this: file.clone(),
                                        conflict_with: conflicting.clone(),
                                    });
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
    installed_mods: ModMap
}

impl ActualInstall {
    pub fn virtualize(&self) -> VirtualInstall {
        VirtualInstall {
            installed_mods: self.installed_mods.clone()
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
                ModInstallOperations::InstallMod { mod_id, info, version } => {
                    todo!()
                }
                ModInstallOperations::UninstallMod(file) => {
                    todo!()
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct VirtualInstall {
    installed_mods: ModMap
}

impl VirtualInstall {
    pub fn from_mod_map(mod_map: ModMap) -> VirtualInstall {
        Self {
            installed_mods: mod_map
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
                ModInstallOperations::InstallMod {
                    mod_id, info, version
                } => {
                    let file = ModFile::new(mod_id, info.clone(), version.clone());

                    let files = self.installed_mods.entry(mod_id.clone()).or_default();

                    files.push(file);
                }

                ModInstallOperations::UninstallMod(modfile) => {
                    let Some(files) = self.installed_mods.get_mut(&modfile.mod_id) else {
                        return Err(InstallError::FileNotFound)
                    };

                    if files.contains(modfile) {
                        files.retain(|x| x != modfile);
                    }

                    if files.len() <= 0 {
                        self.installed_mods.remove(&modfile.mod_id);
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
    FileError(io::Error)
}
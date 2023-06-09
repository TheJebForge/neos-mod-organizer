use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use crate::install::{ModFile, ModInstall, ModMap, VirtualInstall};
use crate::manifest::{Artifact, Category, Conflict, Dependency, ManifestMods, Mod, ModVersion};
use crate::version::{Version, Comparator};

#[test]
fn mod_install_missing_dependency() {
    let manifest_mods: Arc<ManifestMods> = Arc::new(HashMap::from([
        (format!("test.mod.1"), Mod {
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
                (Version::from_major(1), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: Some(HashMap::from([
                        (format!("test.mod.dep"), Dependency {
                            version: Comparator::from_str("1").unwrap(),
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
        })
    ]));

    let mod_map: ModMap = HashMap::from([
        (format!("test.mod.1"), vec![
            ModFile::new("test.mod.1", Version::from_major(1), &manifest_mods)
        ])
    ]);

    let virt = VirtualInstall::new(mod_map, manifest_mods.clone());

    assert_eq!(virt.check_for_conflicts(&manifest_mods).len(), 1)
}

#[test]
fn mod_install_valid_dependency() {
    let manifest_mods: Arc<ManifestMods> = Arc::new(HashMap::from([
        (format!("test.mod.1"), Mod {
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
                (Version::from_major(1), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: Some(HashMap::from([
                        (format!("test.mod.dep"), Dependency {
                            version: Comparator::from_str("1").unwrap(),
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
        }),
        (format!("test.mod.dep"), Mod {
            name: "".to_string(),
            color: None,
            description: "".to_string(),
            authors: Default::default(),
            source_location: None,
            website: None,
            tags: None,
            category: Category::Libraries,
            flags: None,
            versions: HashMap::from([
                (Version::from_major(1), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: None,
                    artifacts: vec![
                        Artifact {
                            url: "test.mod/testdep.dll".to_string(),
                            filename: None,
                            sha256: "356357".to_string(),
                            blake3: None,
                            install_location: None,
                        }
                    ],
                })
            ]),
        })
    ]));

    let mod_map: ModMap = HashMap::from([
        (format!("test.mod.1"), vec![
            ModFile::new("test.mod.1", Version::from_major(1), &manifest_mods)
        ]),
        (format!("test.mod.dep"), vec![
            ModFile::new("test.mod.dep", Version::from_major(1), &manifest_mods)
        ])
    ]);

    let virt = VirtualInstall::new(mod_map, manifest_mods.clone());

    assert_eq!(virt.check_for_conflicts(&manifest_mods).len(), 0)
}

#[test]
fn mod_install_invalid_dependency() {
    let manifest_mods: Arc<ManifestMods> = Arc::new(HashMap::from([
        (format!("test.mod.1"), Mod {
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
                (Version::from_major(1), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: Some(HashMap::from([
                        (format!("test.mod.dep"), Dependency {
                            version: Comparator::from_str("1").unwrap(),
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
        }),
        (format!("test.mod.dep"), Mod {
            name: "".to_string(),
            color: None,
            description: "".to_string(),
            authors: Default::default(),
            source_location: None,
            website: None,
            tags: None,
            category: Category::Libraries,
            flags: None,
            versions: HashMap::from([
                (Version::from_major(2), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: None,
                    artifacts: vec![
                        Artifact {
                            url: "test.mod/testdep.dll".to_string(),
                            filename: None,
                            sha256: "356357".to_string(),
                            blake3: None,
                            install_location: None,
                        }
                    ],
                })
            ]),
        })
    ]));

    let mod_map: ModMap = HashMap::from([
        (format!("test.mod.1"), vec![
            ModFile::new("test.mod.1", Version::from_major(1), &manifest_mods)
        ]),
        (format!("test.mod.dep"), vec![
            ModFile::new("test.mod.dep", Version::from_major(2), &manifest_mods)
        ])
    ]);

    let virt = VirtualInstall::new(mod_map, manifest_mods.clone());

    assert_eq!(virt.check_for_conflicts(&manifest_mods).len(), 1)
}

#[test]
fn mod_install_multiple_versions() {
    let manifest_mods: Arc<ManifestMods> = Arc::new(HashMap::from([
        (format!("test.mod.1"), Mod {
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
                (Version::from_major(1), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: Some(HashMap::from([
                        (format!("test.mod.dep"), Dependency {
                            version: Comparator::from_str("1").unwrap(),
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
        }),
        (format!("test.mod.dep"), Mod {
            name: "".to_string(),
            color: None,
            description: "".to_string(),
            authors: Default::default(),
            source_location: None,
            website: None,
            tags: None,
            category: Category::Libraries,
            flags: None,
            versions: HashMap::from([
                (Version::from_major(2), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: None,
                    artifacts: vec![
                        Artifact {
                            url: "test.mod/testdep.dll".to_string(),
                            filename: None,
                            sha256: "356357".to_string(),
                            blake3: None,
                            install_location: None,
                        }
                    ],
                }),
                (Version::from_major(3), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: None,
                    artifacts: vec![
                        Artifact {
                            url: "test.mod/testdep.dll".to_string(),
                            filename: None,
                            sha256: "356357".to_string(),
                            blake3: None,
                            install_location: None,
                        }
                    ],
                })
            ]),
        })
    ]));

    let mod_map: ModMap = HashMap::from([
        (format!("test.mod.1"), vec![
            ModFile::new("test.mod.1", Version::from_major(1), &manifest_mods)
        ]),
        (format!("test.mod.dep"), vec![
            ModFile::new("test.mod.dep", Version::from_major(2), &manifest_mods),
            ModFile::new("test.mod.dep", Version::from_major(3), &manifest_mods)
        ])
    ]);

    let virt = VirtualInstall::new(mod_map, manifest_mods.clone());

    assert_eq!(virt.check_for_conflicts(&manifest_mods).len(), 2)
}

#[test]
fn mod_install_direct_conflict() {
    let manifest_mods: Arc<ManifestMods> = Arc::new(HashMap::from([
        (format!("test.mod.1"), Mod {
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
                (Version::from_major(1), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: Some(HashMap::from([
                        (format!("test.mod.dep"), Conflict {
                            version: Comparator::from_str("*").unwrap(),
                        })
                    ])),
                    dependencies: None,
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
        }),
        (format!("test.mod.dep"), Mod {
            name: "".to_string(),
            color: None,
            description: "".to_string(),
            authors: Default::default(),
            source_location: None,
            website: None,
            tags: None,
            category: Category::Libraries,
            flags: None,
            versions: HashMap::from([
                (Version::from_major(2), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: None,
                    artifacts: vec![
                        Artifact {
                            url: "test.mod/testdep.dll".to_string(),
                            filename: None,
                            sha256: "356357".to_string(),
                            blake3: None,
                            install_location: None,
                        }
                    ],
                })
            ]),
        })
    ]));

    let mod_map: ModMap = HashMap::from([
        (format!("test.mod.1"), vec![
            ModFile::new("test.mod.1", Version::from_major(1), &manifest_mods)
        ]),
        (format!("test.mod.dep"), vec![
            ModFile::new("test.mod.dep", Version::from_major(1), &manifest_mods)
        ])
    ]);

    let virt = VirtualInstall::new(mod_map, manifest_mods.clone());

    assert_eq!(virt.check_for_conflicts(&manifest_mods).len(), 1)
}

#[test]
fn mod_install_direct_conflict_unaffected() {
    let manifest_mods: Arc<ManifestMods> = Arc::new(HashMap::from([
        (format!("test.mod.1"), Mod {
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
                (Version::from_major(1), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: Some(HashMap::from([
                        (format!("test.mod.dep"), Conflict {
                            version: Comparator::from_str("^0.1").unwrap(),
                        })
                    ])),
                    dependencies: None,
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
        }),
        (format!("test.mod.dep"), Mod {
            name: "".to_string(),
            color: None,
            description: "".to_string(),
            authors: Default::default(),
            source_location: None,
            website: None,
            tags: None,
            category: Category::Libraries,
            flags: None,
            versions: HashMap::from([
                (Version::from_major(2), ModVersion {
                    changelog: None,
                    release_url: None,
                    neos_version_compatibility: None,
                    modloader_version_compatibility: None,
                    flags: None,
                    conflicts: None,
                    dependencies: None,
                    artifacts: vec![
                        Artifact {
                            url: "test.mod/testdep.dll".to_string(),
                            filename: None,
                            sha256: "356357".to_string(),
                            blake3: None,
                            install_location: None,
                        }
                    ],
                })
            ]),
        })
    ]));

    let mod_map: ModMap = HashMap::from([
        (format!("test.mod.1"), vec![
            ModFile::new("test.mod.1", Version::from_major(1), &manifest_mods)
        ]),
        (format!("test.mod.dep"), vec![
            ModFile::new("test.mod.dep", Version::from_major(1), &manifest_mods)
        ])
    ]);

    let virt = VirtualInstall::new(mod_map, manifest_mods.clone());

    assert_eq!(virt.check_for_conflicts(&manifest_mods).len(), 0)
}
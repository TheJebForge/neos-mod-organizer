use std::collections::{HashMap, VecDeque};
use crate::install::{ModInstallOperations, ModMap};
use crate::manifest::{GUID, Mod, ModVersion};
use crate::version::{Version, VersionReq};

#[inline]
pub fn find_latest_matching<'a>(mod_id: &str, requirement: &VersionReq, mod_list: &'a HashMap<GUID, Mod>) -> Option<(&'a Mod, &'a Version, &'a ModVersion)> {
    let Some(mod_info) = mod_list.get(mod_id) else {
        return None;
    };

    let mut fitting_versions = mod_info.versions.iter().filter(|(version, _)| {
        requirement.matches(version)
    }).collect::<Vec<(&Version, &ModVersion)>>();

    if fitting_versions.len() <= 0 {
        return None;
    }

    fitting_versions.sort_by(|(a, _), (b, _)| {
        b.cmp(a)
    });

    let (latest_version, latest_info) = fitting_versions.remove(0);

    Some((mod_info, latest_version, latest_info))
}

pub fn resolve_install_mod(mod_id: &str, requirement: &VersionReq, current_install: &ModMap, mod_list: &HashMap<GUID, Mod>) -> ResolveResult {
    let mut ops = Vec::new();
    let mut queue = VecDeque::from([(mod_id, requirement)]);

    while let Some((mod_id, requirement)) = queue.pop_back() {
        let mut piece = vec![];

        let Some((mod_info, version, version_info)) = find_latest_matching(mod_id, requirement, mod_list) else {
            return ResolveResult::UnableToFind {
                mod_id: mod_id.to_string(),
                requirement: requirement.clone()
            }
        };

        if let Some(installed_versions) = current_install.get(mod_id) {
            if installed_versions.iter().any(|x| x.version.is_some() && requirement.matches(x.version.as_ref().unwrap()) && x.version.as_ref().unwrap() >= version) {
                continue;
            } else {
                for version in installed_versions {
                    piece.push(ModInstallOperations::UninstallMod(version.clone()));
                }
            }
        }



        piece.push(ModInstallOperations::InstallMod {
            mod_id: mod_id.to_string(),
            info: mod_info.clone(),
            version: version.clone(),
        });

        ops.push(piece);

        if let Some(dependencies) = &version_info.dependencies {
            for (depedency_id, dependency_info) in dependencies {
                queue.push_back((depedency_id.as_str(), &dependency_info.version));
            }
        }
    }

    ops.reverse();

    ResolveResult::Ok(ops.into_iter().flatten().collect())
}

pub enum ResolveResult {
    /// When everything went ok
    Ok(Vec<ModInstallOperations>),

    /// When a mod couldn't be found
    UnableToFind {
        mod_id: GUID,
        requirement: VersionReq
    }
}
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use crate::pkg_config::PkgConfigFile;

fn get_multiarch_lib_path_iter() -> &'static [PathBuf] {
    static MULTIARCH_PATH: OnceLock<Vec<PathBuf>> = OnceLock::new();
    MULTIARCH_PATH.get_or_init(|| {
        Command::new("gcc")
            .arg("-dumpmachine")
            .output()
            .map(|o| String::from_utf8(o.stdout).unwrap_or_default())
            .map_or(vec![], |arch| {
                vec![PathBuf::from(format!("/usr/lib/{}", arch.trim()))]
            })
    })
}

pub fn find_library(library: &str, extension: &str, search_paths: &[PathBuf]) -> Result<String> {
    let filepaths: Vec<_> = search_paths
        .iter()
        .chain(get_multiarch_lib_path_iter())
        .map(|base| base.join(format!("lib{}.{}", library, extension)))
        .collect();

    let error = anyhow!(
        "Could not find required library `{}` at paths: `{:?}`",
        library,
        &filepaths
    );
    Ok(filepaths
        .into_iter()
        .find(|path| path.exists())
        .ok_or(error)?
        .into_os_string()
        .into_string()
        .unwrap())
}

pub fn find_dylib(library: &str, search_paths: &[PathBuf]) -> Result<String> {
    find_library(library, "so", search_paths)
}

pub fn find_archive(library: &str, search_paths: &[PathBuf]) -> Result<String> {
    find_library(library, "a", search_paths)
}

pub fn is_dylib(path: &str) -> bool {
    path.ends_with("so")
}

pub fn is_archive(path: &str) -> bool {
    path.ends_with('a')
}

#[derive(Debug, Default)]
pub struct FullLibraryPaths {
    pub default_component_name: String,
    pub dylib_location: Option<String>,
    pub archive_location: Option<String>,
    pub link_libraries: HashMap<String, String>,
}

impl FullLibraryPaths {
    pub fn find(pkg_config: &PkgConfigFile) -> Result<Self> {
        let search_paths = pkg_config
            .link_locations
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        let dylib_location = pkg_config
            .link_libraries
            .first()
            .map(|library| find_dylib(library, &search_paths))
            .transpose()?;

        let archive_location = pkg_config
            .link_libraries
            .first()
            .map(|library| find_archive(library, &search_paths))
            .transpose()?;

        let link_libraries: HashMap<String, String> = pkg_config
            .link_libraries
            .iter()
            .skip(1)
            .map(|name| -> Result<(String, String)> {
                let dylib_path = find_dylib(name, &search_paths);
                let archive_path = find_archive(name, &search_paths);

                // prefer dylib if dylib_location exists
                if dylib_location.is_some() {
                    if let Ok(dylib_path) = dylib_path {
                        return Ok((name.clone(), dylib_path));
                    } else if let Ok(archive_path) = archive_path {
                        return Ok((name.clone(), archive_path));
                    }
                }

                // otherwise, prefer static lib
                if let Ok(archive_path) = archive_path {
                    return Ok((name.clone(), archive_path));
                } else if let Ok(dylib_path) = dylib_path {
                    return Ok((name.clone(), dylib_path));
                }

                // if we found neither, error
                Err(anyhow!(
                    "Error finding paths for `{}`:\ndylib: {}\narchive: {}",
                    &pkg_config.name,
                    dylib_path.err().unwrap(),
                    archive_path.err().unwrap(),
                ))
            })
            .collect::<Result<_>>()?;

        let default_component_name = pkg_config
            .link_libraries
            .first()
            .unwrap_or(&pkg_config.name)
            .clone();

        Ok(Self {
            default_component_name,
            dylib_location,
            archive_location,
            link_libraries,
        })
    }
}

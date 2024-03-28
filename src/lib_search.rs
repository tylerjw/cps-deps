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

#[derive(Debug)]
pub enum LibraryLocation {
    Archive(String),
    Dylib(String),
    Both { archive: String, dylib: String },
}

impl LibraryLocation {
    pub fn find(library: &str, search_paths: &[PathBuf]) -> Result<Self> {
        let dylib = find_library(library, "so", search_paths);
        let archive = find_library(library, "a", search_paths);

        match (dylib, archive) {
            (Ok(dylib), Err(_)) => Ok(Self::Dylib(dylib)),
            (Err(_), Ok(archive)) => Ok(Self::Archive(archive)),
            (Ok(dylib), Ok(archive)) => Ok(Self::Both { archive, dylib }),
            (Err(dylib_error), Err(archive_error)) => {
                Err(anyhow!("{}\n{}", dylib_error, archive_error))
            }
        }
    }
}

pub fn find_locations(pkg_config: &PkgConfigFile) -> Result<HashMap<String, LibraryLocation>> {
    let search_paths = pkg_config
        .link_locations
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    Ok(pkg_config
        .link_libraries
        .iter()
        .map(|name| -> Result<(String, LibraryLocation)> {
            let location = LibraryLocation::find(name, &search_paths)?;
            Ok((name.clone(), location))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .collect())
}

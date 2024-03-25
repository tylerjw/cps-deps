use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

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

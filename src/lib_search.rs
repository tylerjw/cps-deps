use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn get_multiarch_lib_path() -> Option<PathBuf> {
    let output = Command::new("gcc").arg("-dumpmachine").output().ok()?;
    let arch = String::from_utf8(output.stdout).ok()?.trim().to_string();
    Some(PathBuf::from(format!("/usr/lib/{}", arch)))
}

pub fn find_library(library: &str, extension: &str, search_paths: &[PathBuf]) -> Result<String> {
    let filepaths: Vec<_> = search_paths
        .iter()
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

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use walkdir::WalkDir;

fn find_pc_files() -> Vec<PathBuf> {
    [
        "/usr/lib",
        "/usr/share",
        "/usr/local/lib",
        "/usr/local/share",
    ]
    .iter()
    .map(PathBuf::from)
    .flat_map(|dir| WalkDir::new(dir).into_iter().filter_map(Result::ok))
    .filter(|dir_entry| dir_entry.file_type().is_file())
    .filter(|dir_entry| dir_entry.path().extension().is_some_and(|ex| ex == "pc"))
    .map(|dir_entry| PathBuf::from(dir_entry.path()))
    .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let pc_files = find_pc_files();

    dbg!(&pc_files);

    let package_names: Vec<_> = pc_files
        .iter()
        .map(|path| {
            path.with_extension("")
                .to_owned()
                .file_name()
                .unwrap()
                .to_os_string()
        })
        .collect();

    let libraries: HashMap<String, pkg_config::Library> = package_names
        .iter()
        .flat_map(|name| name.to_str())
        .flat_map(|name| {
            pkg_config::Config::new()
                .cargo_metadata(false)
                .env_metadata(false)
                .print_system_cflags(false)
                .print_system_libs(false)
                .probe(name)
                .map(|lib| (name.to_string(), lib))
        })
        .collect();

    dbg!(libraries.get("svrcore"));

    Ok(())
}

use anyhow::Context;
use clap::Parser;
use cps_deps::cps;
use cps_deps::pkg_config;
use std::collections::HashMap;
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

#[derive(Parser)]
struct Args {
    outputdir: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let pc_files = find_pc_files();

    for path in pc_files {
        dbg!(&path);
        let pc_filename = path
            .file_name()
            .context("error getting filename of pc file")?
            .to_str()
            .context("error converting OsStr to str")?
            .to_string();
        let data = std::fs::read_to_string(path)?;
        let pkg = pkg_config::PkgConfigFile::parse(&data)?;
        let library = pkg_config::Library::new(&data, &pc_filename);
        let library = match library {
            Ok(library) => library,
            Err(error) => {
                println!("{}", error);
                continue;
            }
        };

        let cps = match (library.archive_location, library.dylib_location) {
            (None, None) => {
                // Interface
                cps::Package {
                    name: pkg.name.clone(),
                    cps_version: "0.10.0".to_string(),
                    version: Some(pkg.version),
                    description: Some(pkg.description),
                    default_components: Some(vec![library.default_component_name.clone()]),
                    components: HashMap::from([(
                        library.default_component_name,
                        cps::Component::Interface(cps::LocationOptionalComponent {
                            requires: Some(pkg.requires.iter().map(|d| d.name.clone()).collect()),
                            compile_flags: (!pkg.compile_flags.is_empty()).then(|| {
                                cps::LanguageStringList::any_language_map(pkg.compile_flags)
                            }),
                            definitions: (!pkg.definitions.is_empty()).then(|| {
                                cps::LanguageStringList::any_language_map(pkg.definitions)
                            }),
                            includes: (!pkg.includes.is_empty())
                                .then(|| cps::LanguageStringList::any_language_map(pkg.includes)),
                            link_flags: (!pkg.link_flags.is_empty()).then_some(pkg.link_flags),
                            ..cps::LocationOptionalComponent::default()
                        }),
                    )]),
                    ..cps::Package::default()
                }
            }
            (Some(archive_location), None) => {
                // Archive
                let mut components = HashMap::<String, cps::Component>::new();
                let local_requires: Option<Vec<String>> =
                    (library.link_libraries.keys().next().is_some()).then(|| {
                        library
                            .link_libraries
                            .keys()
                            .map(|name| format!(":{}", name))
                            .collect()
                    });
                let remote_requres = Some(
                    pkg.requires
                        .iter()
                        .map(|d| d.name.clone())
                        .collect::<Vec<_>>(),
                );
                let requires = match (local_requires, remote_requres) {
                    (Some(local), Some(remote)) => {
                        Some(local.into_iter().chain(remote.into_iter()).collect())
                    }
                    (local, remote) => local.or(remote),
                };

                components.insert(
                    library.default_component_name.clone(),
                    cps::Component::Archive(cps::LocationRequiredComponent {
                        location: archive_location,
                        requires,
                        compile_flags: (!pkg.compile_flags.is_empty())
                            .then(|| cps::LanguageStringList::any_language_map(pkg.compile_flags)),
                        definitions: (!pkg.definitions.is_empty())
                            .then(|| cps::LanguageStringList::any_language_map(pkg.definitions)),
                        includes: (!pkg.includes.is_empty())
                            .then(|| cps::LanguageStringList::any_language_map(pkg.includes)),
                        link_flags: (!pkg.link_flags.is_empty()).then_some(pkg.link_flags),
                        ..cps::LocationRequiredComponent::default()
                    }),
                );

                for (name, location) in library.link_libraries {
                    if location.ends_with("so") {
                        components.insert(
                            name,
                            cps::Component::Dylib(cps::LocationRequiredComponent {
                                location,
                                ..cps::LocationRequiredComponent::default()
                            }),
                        );
                    } else {
                        components.insert(
                            name,
                            cps::Component::Archive(cps::LocationRequiredComponent {
                                location,
                                ..cps::LocationRequiredComponent::default()
                            }),
                        );
                    }
                }

                cps::Package {
                    name: pkg.name.clone(),
                    cps_version: "0.10.0".to_string(),
                    version: Some(pkg.version),
                    description: Some(pkg.description),
                    default_components: Some(vec![library.default_component_name]),
                    components,
                    ..cps::Package::default()
                }
            }
            (_, Some(dylib_location)) => {
                // Dylib
                let mut components = HashMap::<String, cps::Component>::new();
                let local_requires: Option<Vec<String>> =
                    (library.link_libraries.keys().next().is_some()).then(|| {
                        library
                            .link_libraries
                            .keys()
                            .map(|name| format!(":{}", name))
                            .collect()
                    });
                let remote_requres = Some(
                    pkg.requires
                        .iter()
                        .map(|d| d.name.clone())
                        .collect::<Vec<_>>(),
                );
                let requires = match (local_requires, remote_requres) {
                    (Some(local), Some(remote)) => {
                        Some(local.into_iter().chain(remote.into_iter()).collect())
                    }
                    (local, remote) => local.or(remote),
                };

                components.insert(
                    library.default_component_name.clone(),
                    cps::Component::Dylib(cps::LocationRequiredComponent {
                        location: dylib_location,
                        requires,
                        compile_flags: (!pkg.compile_flags.is_empty())
                            .then(|| cps::LanguageStringList::any_language_map(pkg.compile_flags)),
                        definitions: (!pkg.definitions.is_empty())
                            .then(|| cps::LanguageStringList::any_language_map(pkg.definitions)),
                        includes: (!pkg.includes.is_empty())
                            .then(|| cps::LanguageStringList::any_language_map(pkg.includes)),
                        link_flags: (!pkg.link_flags.is_empty()).then_some(pkg.link_flags),
                        ..cps::LocationRequiredComponent::default()
                    }),
                );

                for (name, location) in library.link_libraries {
                    if location.ends_with("so") {
                        components.insert(
                            name,
                            cps::Component::Dylib(cps::LocationRequiredComponent {
                                location,
                                ..cps::LocationRequiredComponent::default()
                            }),
                        );
                    } else {
                        components.insert(
                            name,
                            cps::Component::Archive(cps::LocationRequiredComponent {
                                location,
                                ..cps::LocationRequiredComponent::default()
                            }),
                        );
                    }
                }

                cps::Package {
                    name: pkg.name.clone(),
                    cps_version: "0.10.0".to_string(),
                    version: Some(pkg.version),
                    description: Some(pkg.description),
                    default_components: Some(vec![library.default_component_name]),
                    components,
                    ..cps::Package::default()
                }
            }
        };

        let json = serde_json::to_string_pretty(&cps)?;
        let cps_filename = pc_filename.replace(".pc", ".cps");
        std::fs::write(args.outputdir.join(cps_filename), json)?;
    }

    Ok(())
}

use crate::{cps, lib_search, pkg_config};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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

impl TryFrom<pkg_config::PkgConfigFile> for cps::Package {
    type Error = anyhow::Error;

    fn try_from(pkg_config: pkg_config::PkgConfigFile) -> Result<cps::Package> {
        let library = lib_search::FullLibraryPaths::find(&pkg_config)?;

        let package_requires_map: HashMap<_, _> = pkg_config
            .requires
            .iter()
            .filter(|req| req.version.is_some())
            .map(|req| {
                (
                    req.name.clone(),
                    cps::Requirement {
                        version: req.version.clone(),
                        ..cps::Requirement::default()
                    },
                )
            })
            .collect();
        let package_requires_map =
            (!package_requires_map.is_empty()).then_some(package_requires_map);

        let local_requires: Option<Vec<String>> = (library.link_libraries.keys().next().is_some())
            .then(|| {
                library
                    .link_libraries
                    .keys()
                    .map(|name| format!(":{}", name))
                    .collect()
            });
        let remote_requres = Some(
            pkg_config
                .requires
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<_>>(),
        );
        let main_component_requires = match (local_requires, remote_requres) {
            (Some(local), Some(remote)) => {
                Some(local.into_iter().chain(remote.into_iter()).collect())
            }
            (Some(local), None) => Some(local),
            (None, Some(remote)) => Some(remote),
            (None, None) => None,
        };

        let cps = match (library.archive_location, library.dylib_location) {
            (None, None) => {
                // Interface
                cps::Package {
                    name: pkg_config.name.clone(),
                    version: Some(pkg_config.version),
                    description: Some(pkg_config.description),
                    requires: package_requires_map,
                    default_components: Some(vec![library.default_component_name.clone()]),
                    components: HashMap::from([(
                        library.default_component_name,
                        cps::MaybeComponent::Component(cps::Component::Interface(
                            cps::ComponentFields {
                                requires: main_component_requires,
                                compile_flags: (!pkg_config.compile_flags.is_empty()).then(|| {
                                    cps::LanguageStringList::any_language_map(
                                        pkg_config.compile_flags,
                                    )
                                }),
                                definitions: (!pkg_config.definitions.is_empty()).then(|| {
                                    cps::LanguageStringList::any_language_map(
                                        pkg_config.definitions,
                                    )
                                }),
                                includes: (!pkg_config.includes.is_empty()).then(|| {
                                    cps::LanguageStringList::any_language_map(pkg_config.includes)
                                }),
                                link_flags: (!pkg_config.link_flags.is_empty())
                                    .then_some(pkg_config.link_flags),
                                ..cps::ComponentFields::default()
                            },
                        )),
                    )]),
                    ..cps::Package::default()
                }
            }
            (Some(archive_location), None) => {
                // Archive
                let mut components = HashMap::<String, cps::MaybeComponent>::new();
                components.insert(
                    library.default_component_name.clone(),
                    cps::MaybeComponent::Component(cps::Component::Archive(cps::ComponentFields {
                        location: Some(archive_location),
                        requires: main_component_requires,
                        compile_flags: (!pkg_config.compile_flags.is_empty()).then(|| {
                            cps::LanguageStringList::any_language_map(pkg_config.compile_flags)
                        }),
                        definitions: (!pkg_config.definitions.is_empty()).then(|| {
                            cps::LanguageStringList::any_language_map(pkg_config.definitions)
                        }),
                        includes: (!pkg_config.includes.is_empty()).then(|| {
                            cps::LanguageStringList::any_language_map(pkg_config.includes)
                        }),
                        link_flags: (!pkg_config.link_flags.is_empty())
                            .then_some(pkg_config.link_flags),
                        ..cps::ComponentFields::default()
                    })),
                );

                for (name, location) in library.link_libraries {
                    if location.ends_with("so") {
                        components.insert(
                            name,
                            cps::MaybeComponent::Component(cps::Component::Dylib(
                                cps::ComponentFields {
                                    location: Some(location),
                                    ..cps::ComponentFields::default()
                                },
                            )),
                        );
                    } else {
                        components.insert(
                            name,
                            cps::MaybeComponent::Component(cps::Component::Archive(
                                cps::ComponentFields {
                                    location: Some(location),
                                    ..cps::ComponentFields::default()
                                },
                            )),
                        );
                    }
                }

                cps::Package {
                    name: pkg_config.name.clone(),
                    version: Some(pkg_config.version),
                    description: Some(pkg_config.description),
                    default_components: Some(vec![library.default_component_name]),
                    requires: package_requires_map,
                    components,
                    ..cps::Package::default()
                }
            }
            (_, Some(dylib_location)) => {
                // Dylib
                let mut components = HashMap::<String, cps::MaybeComponent>::new();
                components.insert(
                    library.default_component_name.clone(),
                    cps::MaybeComponent::Component(cps::Component::Dylib(cps::ComponentFields {
                        location: Some(dylib_location),
                        requires: main_component_requires,
                        compile_flags: (!pkg_config.compile_flags.is_empty()).then(|| {
                            cps::LanguageStringList::any_language_map(pkg_config.compile_flags)
                        }),
                        definitions: (!pkg_config.definitions.is_empty()).then(|| {
                            cps::LanguageStringList::any_language_map(pkg_config.definitions)
                        }),
                        includes: (!pkg_config.includes.is_empty()).then(|| {
                            cps::LanguageStringList::any_language_map(pkg_config.includes)
                        }),
                        link_flags: (!pkg_config.link_flags.is_empty())
                            .then_some(pkg_config.link_flags),
                        ..cps::ComponentFields::default()
                    })),
                );

                for (name, location) in library.link_libraries {
                    if location.ends_with("so") {
                        components.insert(
                            name,
                            cps::MaybeComponent::Component(cps::Component::Dylib(
                                cps::ComponentFields {
                                    location: Some(location),
                                    ..cps::ComponentFields::default()
                                },
                            )),
                        );
                    } else {
                        components.insert(
                            name,
                            cps::MaybeComponent::Component(cps::Component::Archive(
                                cps::ComponentFields {
                                    location: Some(location),
                                    ..cps::ComponentFields::default()
                                },
                            )),
                        );
                    }
                }

                cps::Package {
                    name: pkg_config.name.clone(),
                    version: Some(pkg_config.version),
                    description: Some(pkg_config.description),
                    requires: package_requires_map,
                    default_components: Some(vec![library.default_component_name]),
                    components,
                    ..cps::Package::default()
                }
            }
        };
        Ok(cps)
    }
}

pub fn generate_from_pkg_config(outdir: &Path) -> Result<()> {
    let pc_files = find_pc_files();

    fs::create_dir_all(outdir)?;

    for path in pc_files {
        dbg!(&path);
        let pc_filename = path
            .file_name()
            .context("error getting filename of pc file")?
            .to_str()
            .context("error converting OsStr to str")?
            .to_string();
        let data = std::fs::read_to_string(path)?;
        let pkg_config = pkg_config::PkgConfigFile::parse(&data)?;
        let cps_package: cps::Package = match pkg_config.try_into() {
            Ok(cps) => cps,
            Err(error) => {
                eprintln!("Error: {}", error);
                continue;
            }
        };
        let json = serde_json::to_string_pretty(&cps_package)?;
        let cps_filename = pc_filename.replace(".pc", ".cps");
        std::fs::write(outdir.join(cps_filename), json)?;
    }

    Ok(())
}

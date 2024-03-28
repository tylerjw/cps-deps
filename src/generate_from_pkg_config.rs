use crate::lib_search::LibraryLocation;
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
        let library_locations = lib_search::find_locations(&pkg_config)?;

        let location_library_name = pkg_config.link_libraries.first();
        let default_component_name = location_library_name.unwrap_or(&pkg_config.name);

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

        let local_requires: Vec<String> = library_locations
            .keys()
            .filter(|&name| {
                location_library_name.is_some() && name != location_library_name.unwrap()
            })
            .map(|name| format!(":{}", name))
            .collect();
        let local_requires = (!local_requires.is_empty()).then_some(local_requires);
        let remote_requres = (!pkg_config.requires.is_empty()).then(|| {
            pkg_config
                .requires
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<_>>()
        });
        let default_component_requires = match (local_requires, remote_requres) {
            (Some(local), Some(remote)) => Some(local.into_iter().chain(remote).collect()),
            (Some(local), None) => Some(local),
            (None, Some(remote)) => Some(remote),
            (None, None) => None,
        };

        let mut package_configurations: Option<Vec<String>> = None;
        let mut components = HashMap::<String, cps::MaybeComponent>::new();
        for (name, location) in library_locations {
            match location {
                LibraryLocation::Dylib(location) => {
                    components.insert(
                        name.clone(),
                        cps::MaybeComponent::from_dylib_location(&location),
                    );
                }
                LibraryLocation::Archive(location) => {
                    components.insert(
                        name.clone(),
                        cps::MaybeComponent::from_archive_location(&location),
                    );
                }
                LibraryLocation::Both { archive, dylib } => {
                    package_configurations = Some(vec!["shared".to_string(), "static".to_string()]);
                    components.insert(
                        name.clone(),
                        cps::MaybeComponent::Component(cps::Component::Interface(
                            cps::ComponentFields {
                                configurations: Some(
                                    [
                                        (
                                            "shared".to_string(),
                                            cps::Configuration {
                                                requires: Some(vec![format!(":{}-shared", name)]),
                                                ..cps::Configuration::default()
                                            },
                                        ),
                                        (
                                            "static".to_string(),
                                            cps::Configuration {
                                                requires: Some(vec![format!(":{}-static", name)]),
                                                ..cps::Configuration::default()
                                            },
                                        ),
                                    ]
                                    .into_iter()
                                    .collect(),
                                ),
                                ..cps::ComponentFields::default()
                            },
                        )),
                    );
                    components.insert(
                        format!("{}-shared", name),
                        cps::MaybeComponent::from_dylib_location(&archive),
                    );
                    components.insert(
                        format!("{}-static", name),
                        cps::MaybeComponent::from_archive_location(&dylib),
                    );
                }
            };
        }

        let default_component = components.entry(default_component_name.clone()).or_insert(
            cps::MaybeComponent::Component(cps::Component::Interface(
                cps::ComponentFields::default(),
            )),
        );
        let default_component = match default_component {
            cps::MaybeComponent::Component(cps::Component::Interface(fields)) => fields,
            cps::MaybeComponent::Component(cps::Component::Dylib(fields)) => fields,
            cps::MaybeComponent::Component(cps::Component::Archive(fields)) => fields,
            component => {
                anyhow::bail!("Unknwon default component type found: {:?}", component)
            }
        };

        // Requires could be per-configuration or on the component
        if default_component_requires.is_some() {
            if let Some(configurations) = &mut default_component.configurations {
                for configuration in configurations.values_mut() {
                    configuration.requires = Some(
                        [
                            &configuration.requires.clone().unwrap_or_default()[..],
                            &default_component_requires.clone().unwrap_or_default()[..],
                        ]
                        .concat(),
                    );
                }
            } else {
                default_component.requires = default_component_requires;
            }
        }

        default_component.compile_flags = (!pkg_config.compile_flags.is_empty())
            .then(|| cps::LanguageStringList::any_language_map(pkg_config.compile_flags));
        default_component.definitions = (!pkg_config.definitions.is_empty())
            .then(|| cps::LanguageStringList::any_language_map(pkg_config.definitions));
        default_component.includes = (!pkg_config.includes.is_empty())
            .then(|| cps::LanguageStringList::any_language_map(pkg_config.includes));
        default_component.link_flags =
            (!pkg_config.link_flags.is_empty()).then_some(pkg_config.link_flags);

        let cps = cps::Package {
            name: pkg_config.name.clone(),
            version: Some(pkg_config.version),
            description: Some(pkg_config.description),
            default_components: Some(vec![default_component_name.clone()]),
            requires: package_requires_map,
            components,
            configurations: package_configurations,
            ..cps::Package::default()
        };
        Ok(cps)
    }
}

pub fn generate_all_from_pkg_config(outdir: &Path) -> Result<()> {
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

pub fn generate_from_pkg_config(pc_filepath: &Path, cps_filepath: &Path) -> Result<()> {
    let data = std::fs::read_to_string(pc_filepath)?;
    let pkg_config = pkg_config::PkgConfigFile::parse(&data)?;
    let cps_package: cps::Package = pkg_config.try_into()?;
    let json = serde_json::to_string_pretty(&cps_package)?;
    std::fs::write(cps_filepath, json)?;
    Ok(())
}

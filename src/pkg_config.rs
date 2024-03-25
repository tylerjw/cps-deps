use std::process::Command;
use std::{collections::HashMap, path::PathBuf};

use regex::Regex;

#[derive(Debug)]
pub struct Library {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub requires: Option<Vec<String>>,
    pub includes: Vec<PathBuf>,
    pub definitions: Vec<String>,
    pub compile_flags: Vec<String>,
    pub default_component_name: String,
    pub dylib_location: Option<PathBuf>,
    pub archive_location: Option<PathBuf>,
    pub link_libraries: HashMap<String, PathBuf>,
    pub link_flags: Vec<String>,
}

fn get_multiarch_lib_path() -> Option<PathBuf> {
    let output = Command::new("gcc").arg("-dumpmachine").output().ok()?;
    let arch = String::from_utf8(output.stdout).ok()?.trim().to_string();
    Some(PathBuf::from(format!("/usr/lib/{}", arch)))
}

fn strip_comments(data: &str) -> String {
    data.lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n")
}

fn parse_variables(data: &str) -> HashMap<String, String> {
    let re = Regex::new(r"([a-zA-Z0-9\-_]+)[ ]*=[ ]*([:a-zA-Z0-9\-_/=\.+ ]*)?$").unwrap();

    data.lines()
        .flat_map(|line| re.captures_iter(line))
        .flat_map(|c| {
            let name = c.get(1).map(|m| m.as_str().to_string())?;
            let value = c.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            Some((name, value))
        })
        .collect()
}

fn expand_variables(data: &str, index: i32) -> Result<String, String> {
    let variables = parse_variables(data);

    if index > 100 {
        return Err(format!(
            "Max recursion hit expanding variables\n\n{}\n\n{:?}",
            data, variables
        ));
    }

    let mut data = data.to_string();
    for (key, value) in variables {
        let from = format!("${{{}}}", key);
        data = data.replace(&from, &value);
    }

    if data.contains("${") {
        expand_variables(&data, index + 1)
    } else {
        Ok(data)
    }
}

fn find_library(
    library: &str,
    extension: &str,
    search_paths: &[PathBuf],
) -> Result<PathBuf, String> {
    let filepaths: Vec<_> = search_paths
        .iter()
        .map(|base| base.join(format!("lib{}.{}", library, extension)))
        .collect();

    let error_string = format!(
        "Could not find required library `{}` at paths: `{:?}`",
        library, &filepaths
    );
    filepaths
        .into_iter()
        .find(|path| path.exists())
        .ok_or(error_string)
}

fn filter_starts_with(data: &[String], predicate: &str) -> Vec<String> {
    data.iter()
        .filter(|&s| s.starts_with(predicate))
        .map(|l| String::from(&l[predicate.len()..]))
        .collect::<Vec<_>>()
}
fn filter_excluding_starts_with(data: &[String], predicates: &[&str]) -> Vec<String> {
    data.iter()
        .filter(|&s| !predicates.iter().any(|p| s.starts_with(p)))
        .map(String::from)
        .collect::<Vec<_>>()
}

impl Library {
    pub fn new(data: &str, pc_filename: &str) -> Result<Self, String> {
        let data = strip_comments(data);
        let data = expand_variables(&data, 0)?;

        let name = Regex::new(r"Name:[ ]+(.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .unwrap_or_default();
        let description = Regex::new(r"Description:[ ]+(.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string());
        let version = Regex::new(r"Version:[ ]+(.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string());
        let requires = Regex::new(r"Requires:[ ]+(.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .map(|req| {
                req.split(", ")
                    .map(String::from)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>()
            });
        let libs = Regex::new(r"Libs:[ ]+(.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .map(|req| {
                req.split(' ')
                    .map(String::from)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        let cflags = Regex::new(r"Cflags:[ ]+(.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .map(|req| {
                req.split(' ')
                    .map(String::from)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let includes = filter_starts_with(&cflags, "-I")
            .iter()
            .map(PathBuf::from)
            .collect();
        let definitions = filter_starts_with(&cflags, "-D");
        let compile_flags = filter_excluding_starts_with(&cflags, &["-I", "-D"]);

        let mut search_paths = filter_starts_with(&libs, "-L")
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        if let Some(multiarch_search_path) = get_multiarch_lib_path() {
            search_paths.push(multiarch_search_path);
        }

        let link_flags = filter_excluding_starts_with(&libs, &["-L", "-l"]);

        let library_names = filter_starts_with(&libs, "-l");

        let dylib_location = if !library_names.is_empty() {
            find_library(library_names.first().unwrap(), "so", &search_paths).ok()
        } else {
            None
        };

        let archive_location = if !library_names.is_empty() {
            find_library(library_names.first().unwrap(), "a", &search_paths).ok()
        } else {
            None
        };

        let link_libraries: HashMap<String, PathBuf> = library_names
            .iter()
            .skip(1)
            .map(|name| -> Result<(String, PathBuf), String> {
                let dylib_path = find_library(name, "so", &search_paths);
                let archive_path = find_library(name, "a", &search_paths);

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
                Err(format!(
                    "Error finding paths for `{}`:\ndylib: {}\narchive: {}",
                    &pc_filename,
                    dylib_path.err().unwrap(),
                    archive_path.err().unwrap(),
                ))
            })
            .collect::<Result<_, _>>()?;

        let default_component_name = library_names.first().unwrap_or(&name).clone();

        Ok(Self {
            name,
            description,
            version,
            requires,
            includes,
            definitions,
            compile_flags,
            default_component_name,
            dylib_location,
            archive_location,
            link_libraries,
            link_flags,
        })
    }
}

#[test]
fn test_parse() {
    let srvcore_pc = r#"
prefix=/usr
exec_prefix=${prefix}
libdir=${exec_prefix}/lib/x86_64-linux-gnu
includedir=${prefix}/include/nss

Name: NSS
Description: Mozilla Network Security Services
Version: 3.68.2
Requires: nspr
Libs: -L${libdir} -lnss3 -lnssutil3 -lsmime3 -lssl3
Cflags: -I${includedir}
    "#;

    let _library = Library::new(srvcore_pc, "nss.pc").unwrap();
    dbg!(_library);
}

#[test]
fn test_parse_fcl() {
    let fcl = r#"
prefix=/usr
exec_prefix=${prefix}
libdir=/usr/lib/x86_64-linux-gnu
includedir=/usr/include

Name: fcl
Description: Flexible Collision Library
Version: 0.7.0
Requires: ccd eigen3 octomap
Libs: -L${libdir} -lfcl
Cflags: -std=c++11 -I${includedir}
    "#;

    let _library = Library::new(fcl, "nss.pc").unwrap();
    dbg!(_library);
}

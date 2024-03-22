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
    pub location: Option<PathBuf>,
    pub link_libraries: HashMap<String, PathBuf>,
    pub link_flags: Vec<String>,
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

fn find_library(library: &str, search_paths: &[PathBuf]) -> Result<PathBuf, String> {
    let filepaths: Vec<_> = search_paths
        .iter()
        .flat_map(|base| {
            [
                base.join(format!("lib{}.so", library)),
                base.join(format!("lib{}.a", library)),
            ]
            .into_iter()
        })
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
        .map(|l| String::from(&l[predicates[0].len()..]))
        .collect::<Vec<_>>()
}

impl Library {
    pub fn new(data: &str) -> Result<Self, String> {
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

        let search_paths = filter_starts_with(&libs, "-L")
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let link_flags = filter_excluding_starts_with(&libs, &["-L", "-l"]);

        let library_names = filter_starts_with(&libs, "-l");
        let default_component_name = library_names.first().unwrap_or(&name).clone();

        let location = if !library_names.is_empty() {
            Some(find_library(&default_component_name, &search_paths)?)
        } else {
            None
        };

        let link_libraries: HashMap<String, PathBuf> = library_names
            .iter()
            .skip(1)
            .map(|name| -> Result<(String, PathBuf), String> {
                let path = find_library(name, &search_paths)
                    .map_err(|e| format!("Error reading pkg-config `{}.pc`: {}", &name, e))?;
                Ok((name.clone(), path))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            name,
            description,
            version,
            requires,
            includes,
            definitions,
            compile_flags,
            default_component_name,
            location,
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

    let _library = Library::new(srvcore_pc);
}

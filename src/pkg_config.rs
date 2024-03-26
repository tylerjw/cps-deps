use anyhow::{anyhow, Result};
use std::collections::HashMap;

use regex::Regex;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub op: Option<String>,
    pub version: Option<String>,
}

impl Dependency {
    fn parse_list(data: &str) -> Vec<Self> {
        let re = Regex::new(r"([^ ,]+)[ ]*(([<=>!]+)[ ]*([^ ,]+)?)?").unwrap();
        re.captures_iter(data)
            .flat_map(|c| -> Result<Self> {
                Ok(Self {
                    name: c
                        .get(1)
                        .ok_or(anyhow!("captured dependency without name: {:?}", c))?
                        .as_str()
                        .to_string(),
                    op: c.get(3).map(|m| m.as_str().to_string()),
                    version: c.get(4).map(|m| m.as_str().to_string()),
                })
            })
            .collect()
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct PkgConfigFile {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: Option<String>,
    pub includes: Vec<String>,
    pub definitions: Vec<String>,
    pub compile_flags: Vec<String>,
    pub cflags_private: Option<String>,
    pub copyright: Option<String>,
    pub link_locations: Vec<String>,
    pub link_libraries: Vec<String>,
    pub link_flags: Vec<String>,
    pub libs_private: Option<String>,
    pub license: Option<String>,
    pub maintainer: Option<String>,
    pub requires: Vec<Dependency>,
    pub requires_private: Vec<Dependency>,
    pub conflicts: Vec<Dependency>,
    pub provides: Vec<Dependency>,
}

impl PkgConfigFile {
    pub fn parse(data: &str) -> Result<Self> {
        let data = strip_comments(data);
        let data = expand_variables(&data, 0)?;

        let name =
            capture_property("Name", &data)?.ok_or(anyhow!("missing required property `Name`"))?;
        let version = capture_property("Version", &data)?
            .ok_or(anyhow!("missing required property `Version`"))?;
        let description = capture_property("Description", &data)?
            .ok_or(anyhow!("missing required property `Description`"))?;
        let url = capture_property("URL", &data)?;
        let cflags = capture_property("Cflags", &data)?;
        let cflags_private = capture_property("Cflags.private", &data)?;
        let copyright = capture_property("Copyright", &data)?;
        let libs = capture_property("Libs", &data)?;
        let libs_private = capture_property("Libs.private", &data)?;
        let license = capture_property("License", &data)?;
        let maintainer = capture_property("Maintainer", &data)?;
        let requires = capture_property("Requires", &data)?.unwrap_or_default();
        let requires_private = capture_property("Requires.private", &data)?.unwrap_or_default();
        let conflicts = capture_property("Conflicts", &data)?.unwrap_or_default();
        let provides = capture_property("Provides", &data)?.unwrap_or_default();

        // process cflags
        let cflags: Vec<_> = cflags
            .unwrap_or_default()
            .split_whitespace()
            .map(String::from)
            .collect();
        let includes = filter_flag(&cflags, "-I");
        let definitions = filter_flag(&cflags, "-D");
        let compile_flags = filter_excluding_flags(&cflags, &["-I", "-D"]);

        // process libs
        let libs: Vec<_> = libs
            .unwrap_or_default()
            .split_whitespace()
            .map(String::from)
            .collect();
        let link_locations = filter_flag(&libs, "-L");
        let link_libraries = filter_flag(&libs, "-l");
        let link_flags = filter_excluding_flags(&libs, &["-L", "-l"]);

        // process requires
        let requires = Dependency::parse_list(&requires);
        let requires_private = Dependency::parse_list(&requires_private);
        let conflicts = Dependency::parse_list(&conflicts);
        let provides = Dependency::parse_list(&provides);

        Ok(Self {
            name,
            version,
            description,
            url,
            includes,
            definitions,
            compile_flags,
            cflags_private,
            copyright,
            link_locations,
            link_libraries,
            link_flags,
            libs_private,
            license,
            maintainer,
            requires,
            requires_private,
            conflicts,
            provides,
        })
    }
}

fn capture_property(name: &str, data: &str) -> Result<Option<String>> {
    Ok(Regex::new(&format!(r"{}:[ ]+(.+)", name))?
        .captures(data)
        .map(|cap| cap[1].trim().to_string()))
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

fn expand_variables(data: &str, index: i32) -> Result<String> {
    let variables = parse_variables(data);

    if index > 100 {
        return Err(anyhow!(
            "Max recursion hit expanding variables\n\n{}\n\n{:?}",
            data,
            variables
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

fn filter_flag(data: &[String], flag: &str) -> Vec<String> {
    data.iter()
        .filter(|&s| s.starts_with(flag))
        .map(|l| String::from(&l[flag.len()..]))
        .collect::<Vec<_>>()
}

fn filter_excluding_flags(data: &[String], flags: &[&str]) -> Vec<String> {
    data.iter()
        .filter(|&s| !flags.iter().any(|f| s.starts_with(f)))
        .map(String::from)
        .collect::<Vec<_>>()
}

#[test]
fn test_parse_pc_files() -> Result<()> {
    let fcl_pc = r#"
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

    assert_eq!(
        PkgConfigFile::parse(fcl_pc)?,
        PkgConfigFile {
            name: "fcl".to_string(),
            description: "Flexible Collision Library".to_string(),
            version: "0.7.0".to_string(),
            requires: vec![
                Dependency {
                    name: "ccd".to_string(),
                    ..Dependency::default()
                },
                Dependency {
                    name: "eigen3".to_string(),
                    ..Dependency::default()
                },
                Dependency {
                    name: "octomap".to_string(),
                    ..Dependency::default()
                },
            ],
            link_locations: vec!["/usr/lib/x86_64-linux-gnu".to_string()],
            link_libraries: vec!["fcl".to_string()],
            includes: vec!["/usr/include".to_string()],
            compile_flags: vec!["-std=c++11".to_string()],
            ..PkgConfigFile::default()
        },
        "input: {}",
        fcl_pc
    );

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

    assert_eq!(
        PkgConfigFile::parse(srvcore_pc)?,
        PkgConfigFile {
            name: "NSS".to_string(),
            description: "Mozilla Network Security Services".to_string(),
            version: "3.68.2".to_string(),
            requires: vec![Dependency {
                name: "nspr".to_string(),
                ..Dependency::default()
            },],
            link_locations: vec!["/usr/lib/x86_64-linux-gnu".to_string()],
            link_libraries: vec![
                "nss3".to_string(),
                "nssutil3".to_string(),
                "smime3".to_string(),
                "ssl3".to_string()
            ],
            includes: vec!["/usr/include/nss".to_string()],
            ..PkgConfigFile::default()
        },
        "input: {}",
        srvcore_pc
    );
    Ok(())
}

#[test]
fn test_capture_property() -> Result<()> {
    let data = r#"
Name: Fontconfig
Description: Font configuration and customization library
Version: 2.13.1
Requires:  freetype2 >= 21.0.15
Requires.private:  uuid expat
Libs: -L${libdir} -lfontconfig
Libs.private:
Cflags: -I${includedir}
    "#;

    assert_eq!(
        capture_property("Name", data)?.expect("`Name` property not captured"),
        "Fontconfig"
    );
    assert_eq!(
        capture_property("Version", data)?.expect("`Version` property not captured"),
        "2.13.1"
    );
    assert_eq!(
        capture_property("Description", data)?.expect("`Description` property not captured"),
        "Font configuration and customization library"
    );
    assert_eq!(
        capture_property("Cflags", data)?.expect("`Cflags` property not captured"),
        "-I${includedir}"
    );
    assert_eq!(
        capture_property("Libs", data)?.expect("`Libs` property not captured"),
        "-L${libdir} -lfontconfig"
    );
    assert_eq!(capture_property("Libs.private", data)?, None);
    assert_eq!(
        capture_property("Requires", data)?.expect("`Requires` property not captured"),
        "freetype2 >= 21.0.15"
    );
    assert_eq!(
        capture_property("Requires.private", data)?
            .expect("`Requires.private` property not captured"),
        "uuid expat"
    );

    Ok(())
}

#[test]
fn test_parse_dependency_list() -> Result<()> {
    let dependency_lists = [
        "ACE_ETCL",
        "freetype2 >= 21.0.15",
        "gio-2.0 >= 2.50 gee-0.8 >= 0.20",
        "gcalc-2 >= 3.34 gtk+-3.0 > 3.19.3",
        "glib-2.0, gobject-2.0",
        "libudev >=  199",
        "nspr, nss",
        "xproto x11",
        "",
    ];
    let expected = [
        vec![Dependency {
            name: "ACE_ETCL".to_string(),
            op: None,
            version: None,
        }],
        vec![Dependency {
            name: "freetype2".to_string(),
            op: Some(">=".to_string()),
            version: Some("21.0.15".to_string()),
        }],
        vec![
            Dependency {
                name: "gio-2.0".to_string(),
                op: Some(">=".to_string()),
                version: Some("2.50".to_string()),
            },
            Dependency {
                name: "gee-0.8".to_string(),
                op: Some(">=".to_string()),
                version: Some("0.20".to_string()),
            },
        ],
        vec![
            Dependency {
                name: "gcalc-2".to_string(),
                op: Some(">=".to_string()),
                version: Some("3.34".to_string()),
            },
            Dependency {
                name: "gtk+-3.0".to_string(),
                op: Some(">".to_string()),
                version: Some("3.19.3".to_string()),
            },
        ],
        vec![
            Dependency {
                name: "glib-2.0".to_string(),
                op: None,
                version: None,
            },
            Dependency {
                name: "gobject-2.0".to_string(),
                op: None,
                version: None,
            },
        ],
        vec![Dependency {
            name: "libudev".to_string(),
            op: Some(">=".to_string()),
            version: Some("199".to_string()),
        }],
        vec![
            Dependency {
                name: "nspr".to_string(),
                op: None,
                version: None,
            },
            Dependency {
                name: "nss".to_string(),
                op: None,
                version: None,
            },
        ],
        vec![
            Dependency {
                name: "xproto".to_string(),
                op: None,
                version: None,
            },
            Dependency {
                name: "x11".to_string(),
                op: None,
                version: None,
            },
        ],
        vec![],
    ];

    for (dependency_list, expected) in dependency_lists.iter().zip(expected.iter()) {
        let output = Dependency::parse_list(dependency_list);
        assert_eq!(output, *expected, "dependency_list: `{}`", dependency_list);
    }

    Ok(())
}

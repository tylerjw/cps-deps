use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path, str::FromStr};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Platform {
    pub c_runtime_vendor: Option<String>,
    pub c_runtime_version: Option<String>,
    pub clr_vendor: Option<String>,
    pub clr_version: Option<String>,
    pub cpp_runtime_vendor: Option<String>,
    pub cpp_runtime_version: Option<String>,
    pub isa: Option<String>,
    pub jvm_vendor: Option<String>,
    pub jvm_version: Option<String>,
    pub kernel: Option<String>,
    pub kernel_version: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Requirement {
    pub components: Option<Vec<String>>,
    pub hints: Option<Vec<String>>,
    pub version: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ComponentFields {
    pub location: Option<String>,
    pub requires: Option<Vec<String>>,
    pub configurations: Option<HashMap<String, Configuration>>,
    pub compile_features: Option<Vec<String>>,
    pub compile_flags: Option<LanguageStringList>,
    pub definitions: Option<LanguageStringList>,
    pub includes: Option<LanguageStringList>,
    pub link_features: Option<Vec<String>>,
    pub link_flags: Option<Vec<String>>,
    pub link_languages: Option<Vec<String>>,
    pub link_libraries: Option<Vec<String>>,
    pub link_location: Option<String>,
    pub link_requires: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum MaybeComponent {
    Component(Component),
    Other(serde_json::Value),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Component {
    Archive(ComponentFields),
    Dylib(ComponentFields),
    Module(ComponentFields),
    Jar(ComponentFields),
    Interface(ComponentFields),
    Symbolic(ComponentFields),
    #[default]
    Unknwon,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum LanguageStringList {
    LanguageMap(HashMap<String, Vec<String>>),
    List(Vec<String>),
}

impl LanguageStringList {
    pub fn any_language_map(list: Vec<String>) -> Self {
        Self::LanguageMap(HashMap::from([("*".to_string(), list)]))
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Configuration {
    pub location: Option<String>,
    pub requires: Option<Vec<String>>,
    pub compile_features: Option<Vec<String>>,
    pub compile_flags: Option<LanguageStringList>,
    pub definitions: Option<LanguageStringList>,
    pub includes: Option<LanguageStringList>,
    pub link_features: Option<Vec<String>>,
    pub link_flags: Option<Vec<String>>,
    pub link_languages: Option<Vec<String>>,
    pub link_libraries: Option<Vec<String>>,
    pub link_location: Option<String>,
    pub link_requires: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Package {
    pub name: String,
    pub cps_version: String,
    pub components: HashMap<String, MaybeComponent>,

    pub platform: Option<Platform>,
    pub configuration: Option<String>, // required in configuration-specific cps and ignored otherwise
    pub configurations: Option<Vec<String>>,
    pub cps_path: Option<String>,
    pub version: Option<String>,
    pub version_schema: Option<String>,
    pub description: Option<String>,
    pub default_components: Option<Vec<String>>,
    pub requires: Option<HashMap<String, Requirement>>,
    pub compat_version: Option<String>,
}

pub fn parse_and_print_cps(filepath: &Path) -> Result<()> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let package = Package::from_reader(reader)?;

    dbg!(package);
    Ok(())
}

impl FromStr for Package {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Self> {
        let package: Package = serde_json::from_str(data)?;
        Ok(package)
    }
}

impl Package {
    pub fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read,
    {
        let package: Package = serde_json::from_reader(reader)?;
        Ok(package)
    }
}

#[test]
fn test_parse_cps() -> Result<()> {
    // cps_version was manually added: https://github.com/cps-org/cps/issues/57
    let sample_cps = r#"{
    "name": "sample",
    "description": "Sample CPS",
    "license": "BSD",
    "version": "1.2.0",
    "compat_version": "0.8.0",
    "cps_version": "0.11.0",
    "platform": {
        "isa": "x86_64",
        "kernel": "linux",
        "c_runtime_vendor": "gnu",
        "c_runtime_version": "2.20",
        "jvm_version": "1.6"
    },
    "configurations": [ "optimized", "debug" ],
    "default_components": [ "sample" ],
    "components": {
        "sample-core": {
        "type": "interface",
        "definitions": [ "SAMPLE" ],
        "includes": [ "@prefix@/include" ]
        },
        "sample": {
        "type": "interface",
        "configurations": {
            "shared": {
            "requires": [ ":sample-shared" ]
            },
            "static": {
            "requires": [ ":sample-static" ]
            }
        }
        },
        "sample-shared": {
        "type": "dylib",
        "requires": [ ":sample-core" ],
        "configurations": {
            "optimized": {
            "location": "@prefix@/lib64/libsample.so.1.2.0"
            },
            "debug": {
            "location": "@prefix@/lib64/libsample_d.so.1.2.0"
            }
        }
        },
        "sample-static": {
        "type": "archive",
        "definitions": [ "SAMPLE_STATIC" ],
        "requires": [ ":sample-core" ],
        "configurations": {
            "optimized": {
            "location": "@prefix@/lib64/libsample.a"
            },
            "debug": {
            "location": "@prefix@/lib64/libsample_d.a"
            }
        }
        },
        "sample-tool": {
        "type": "exe",
        "location": "@prefix@/bin/sample-tool"
        },
        "sample-java": {
        "type": "jar",
        "location": "@prefix@/share/java/sample.jar"
        }
    }
}"#;

    Package::from_str(sample_cps)?;
    Ok(())
}

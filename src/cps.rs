use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Platform {
    pub c_runtime_vendor: Option<String>,
    pub c_runtime_version: Option<String>,
    pub clr_vendor: Option<String>,
    pub clr_version: Option<String>,
    pub compat_version: Option<String>,
    pub cpp_runtime_vendor: Option<String>,
    pub cpp_runtime_version: Option<String>,
    pub isa: Option<String>,
    pub jvm_vendor: Option<String>,
    pub jvm_version: Option<String>,
    pub kernel: Option<String>,
    pub kernel_version: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Requirement {
    pub components: Option<Vec<String>>,
    pub hints: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Component {
    Archive {
        location: String,
        requires: Option<Vec<String>>,
        configurations: Option<HashMap<String, Configuration>>,
        compile_feature: Option<Vec<String>>,
        compile_flags: Option<LanguageStringList>,
        definitions: Option<LanguageStringList>,
        includes: Option<LanguageStringList>,
        link_features: Option<Vec<String>>,
        link_flags: Option<Vec<String>>,
        link_languages: Option<Vec<String>>,
        link_libraries: Option<Vec<String>>,
        link_location: Option<String>,
        link_requires: Option<String>,
    },
    Dylib {
        location: String,
        requires: Option<Vec<String>>,
        configurations: Option<HashMap<String, Configuration>>,
        compile_feature: Option<Vec<String>>,
        compile_flags: Option<LanguageStringList>,
        definitions: Option<LanguageStringList>,
        includes: Option<LanguageStringList>,
        link_features: Option<Vec<String>>,
        link_flags: Option<Vec<String>>,
        link_languages: Option<Vec<String>>,
        link_libraries: Option<Vec<String>>,
        link_location: Option<String>,
        link_requires: Option<String>,
    },
    Module {
        location: String,
        requires: Option<Vec<String>>,
        configurations: Option<HashMap<String, Configuration>>,
        compile_feature: Option<Vec<String>>,
        compile_flags: Option<LanguageStringList>,
        definitions: Option<LanguageStringList>,
        includes: Option<LanguageStringList>,
        link_features: Option<Vec<String>>,
        link_flags: Option<Vec<String>>,
        link_languages: Option<Vec<String>>,
        link_libraries: Option<Vec<String>>,
        link_location: Option<String>,
        link_requires: Option<String>,
    },
    Jar {
        location: String,
        requires: Option<Vec<String>>,
        configurations: Option<HashMap<String, Configuration>>,
        compile_feature: Option<Vec<String>>,
        compile_flags: Option<LanguageStringList>,
        definitions: Option<LanguageStringList>,
        includes: Option<LanguageStringList>,
        link_features: Option<Vec<String>>,
        link_flags: Option<Vec<String>>,
        link_languages: Option<Vec<String>>,
        link_libraries: Option<Vec<String>>,
        link_location: Option<String>,
    },
    Interface {
        location: Option<String>,
        requires: Option<Vec<String>>,
        configurations: Option<HashMap<String, Configuration>>,
        compile_feature: Option<Vec<String>>,
        compile_flags: Option<LanguageStringList>,
        definitions: Option<LanguageStringList>,
        includes: Option<LanguageStringList>,
        link_features: Option<Vec<String>>,
        link_flags: Option<Vec<String>>,
        link_languages: Option<Vec<String>>,
        link_libraries: Option<Vec<String>>,
        link_location: Option<String>,
        link_requires: Option<String>,
    },
    Symbolic {
        location: Option<String>,
        requires: Option<Vec<String>>,
        configurations: Option<HashMap<String, Configuration>>,
        compile_feature: Option<Vec<String>>,
        compile_flags: Option<LanguageStringList>,
        definitions: Option<LanguageStringList>,
        includes: Option<LanguageStringList>,
        link_features: Option<Vec<String>>,
        link_flags: Option<Vec<String>>,
        link_languages: Option<Vec<String>>,
        link_libraries: Option<Vec<String>>,
        link_location: Option<String>,
        link_requires: Option<String>,
    },
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub requires: Option<Vec<String>>,
    pub compile_feature: Option<Vec<String>>,
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub cps_version: String,
    pub components: HashMap<String, Component>,

    pub platform: Option<Platform>,
    pub configuration: Option<String>,
    pub configurations: Option<Vec<String>>,
    pub cps_path: Option<String>,
    pub version: Option<String>,
    pub version_schema: Option<String>,
    pub description: Option<String>,
    pub default_components: Option<Vec<String>>,
    pub requires: Option<HashMap<String, Requirement>>,
}

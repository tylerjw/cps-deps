use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Platform {
    c_runtime_vendor: Option<String>,
    c_runtime_version: Option<String>,
    clr_vendor: Option<String>,
    clr_version: Option<String>,
    compat_version: Option<String>,
    cpp_runtime_vendor: Option<String>,
    cpp_runtime_version: Option<String>,
    isa: Option<String>,
    jvm_vendor: Option<String>,
    jvm_version: Option<String>,
    kernel: Option<String>,
    kernel_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Requirement {
    components: Option<Vec<String>>,
    hints: Option<Vec<String>>,
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    requires: Option<Vec<String>>,
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    name: String,
    cps_version: String,
    components: HashMap<String, Component>,

    platform: Option<Platform>,
    configuration: Option<String>,
    configurations: Option<Vec<String>>,
    cps_path: Option<String>,
    version: Option<String>,
    version_schema: Option<String>,
    description: Option<String>,
    default_components: Option<Vec<String>>,
    requires: Option<HashMap<String, Requirement>>,
}

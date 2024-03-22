use std::collections::HashMap;

use regex::Regex;

#[derive(Debug)]
pub struct Library {
    pub name: String,
    pub description: String,
    pub version: String,
    pub requires: Vec<String>,
    pub libs: Vec<String>,
    pub cflags: Vec<String>,
}

fn parse_variables(data: &str) -> HashMap<String, String> {
    Regex::new(r"(\w*)=([\w/]+)")
        .unwrap()
        .captures_iter(data)
        .map(|c| c.extract())
        .map(|(_, [name, value])| (name.to_string(), value.to_string()))
        .collect()
}

fn expand_variables(data: String) -> String {
    let variables = parse_variables(&data);

    let mut data = data;
    for (key, value) in variables {
        let from = format!("${{{}}}", key);
        data = data.replace(&from, &value);
    }

    if data.contains("${") {
        expand_variables(data)
    } else {
        data
    }
}

impl Library {
    pub fn parse(data: &str) -> Self {
        let data = expand_variables(data.to_string());

        let name = Regex::new(r"Name: (.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .unwrap_or_default();
        let description = Regex::new(r"Description: (.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .unwrap_or_default();
        let version = Regex::new(r"Version: (.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .unwrap_or_default();
        let requires = Regex::new(r"Requires: (.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .map(|req| req.split(", ").map(String::from).collect::<Vec<String>>())
            .unwrap_or_default();
        let libs = Regex::new(r"Libs: (.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .map(|req| req.split(' ').map(String::from).collect::<Vec<String>>())
            .unwrap_or_default();
        let cflags = Regex::new(r"Cflags: (.+)")
            .unwrap()
            .captures(&data)
            .map(|cap| cap[1].to_string())
            .map(|req| req.split(' ').map(String::from).collect::<Vec<String>>())
            .unwrap_or_default();

        Self {
            name,
            description,
            version,
            requires,
            libs,
            cflags,
        }
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

    let library = Library::parse(srvcore_pc);
    dbg!(library);
}

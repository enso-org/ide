extern crate toml;

use std::env;
use std::fs;
use std::process::Command;

fn get_workspace_members(cargo_toml_root : toml::Value) -> Result<Vec<String>, &'static str> {
    if let toml::Value::Array(list) = &cargo_toml_root["workspace"]["members"] {
        list.iter().map(|val| {
            if let toml::Value::String(s) = val {
                Ok(s.clone())
            } else {
                Err("Workspace member is not a string")
            }
        }).collect()
    } else {
        return Err("Invalid workspace element")
    }
}

fn main() {
    let value = fs::read_to_string("Cargo.toml").unwrap().parse::<toml::Value>().unwrap();
    let arguments : Vec<String> = env::args().skip(1).collect();

    for member in get_workspace_members(value).unwrap() {
        let status = Command::new("wasm-pack")
            .arg("test")
            .arg(&member)
            .args(arguments.iter())
            .status()
            .unwrap();
        println!("Process for {} returned status {}", member, status);
    }
}

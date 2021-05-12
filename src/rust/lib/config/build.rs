use std::fs;
use serde_yaml::Value;
use inflector::*;

const CONFIG_PATH : &str = "../../../config.yaml";

fn main() {
    println!("cargo:rerun-if-changed={}",CONFIG_PATH);
    println!("cargo:rerun-if-changed=build.rs");

    let f = std::fs::File::open(CONFIG_PATH).unwrap();
    let value: Value = serde_yaml::from_reader(f).unwrap();

    let indent   = " ".repeat(4);
    let mut def  = "".to_string();
    let mut inst = "".to_string();
    let mut vars = "".to_string();
    match value {
        Value::Mapping(mapping) => {
            for (key,value) in mapping {
                let key = key.as_str().unwrap().to_snake_case();
                let value = value.as_str().unwrap();
                def.push_str(&format!("{}pub {} : &'static str,\n",indent,key));
                inst.push_str(&format!("{}{} : \"{}\",\n",indent,key,value));
                vars.push_str(&format!("pub const {} : &str = \"{}\";\n",key,value));
            }
        }
        _ => panic!("Unexpected config format.")
    }

    let file = format!(r#"// THIS IS AN AUTOGENERATED FILE BASED ON THE '{config_path}' CONFIG FILE. DO NOT MODIFY IT.

#![allow(non_upper_case_globals)]
pub struct Config {{
    {def}
}}

pub const CONFIG : Config = Config {{
    {inst}
}};

{vars}"#,
    config_path=CONFIG_PATH,def=def,inst=inst,vars=vars);
    fs::write("src/autogenerated.rs",file).ok();
}

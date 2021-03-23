extern crate xmlrpc;
extern crate json;

use std::io;
use std::io::BufRead;
use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use json::JsonValue;
use xmlrpc::{Request, Value};
use std::collections::HashMap;

pub fn call_server(xmlrpc_method: &str, key: Option<String>) -> Value {
    let request = match key {
        None => Request::new(xmlrpc_method)
            .arg(dotenv!("UYUNI_USER"))
            .arg(dotenv!("UYUNI_PASS")),
        Some(val) => Request::new(xmlrpc_method).arg(val),
    };
    let result = request.call_url(dotenv!("UYUNI_URL"));
    return result.unwrap();
}

pub fn read_env(env_variable: &str) -> String {
    env::var(env_variable).unwrap().to_string()
}

pub fn import_json_data(json_file: &str) -> HashMap<String, String> {
    let format_pathfile = format!("assets/{}/{}.json", read_env("UYUNI_PROFILE"), json_file);
    let pathfile = Path::new(&format_pathfile);
    println!("{:?}", pathfile.display());
    let mut file = match File::open(pathfile) {
        Err(why) => panic!("Cannot open file!"),
        Ok(file) => file,
    };
    let mut json_data = String::new();
    file.read_to_string(&mut json_data);

    let parsed = json::parse(&json_data).unwrap();    
    let mut parsed_data = HashMap::new();
    for (json_key, json_value) in parsed.entries() {
        parsed_data.insert(json_key.to_string(), json_value.to_string());
    }
    return parsed_data;
}

pub fn get_system_id(system_name: &str) -> i32 {
    let req = Request::new("system.getId")
        .arg(read_env("UYUNI_KEY"))
        .arg(system_name)
        .call_url(dotenv!("UYUNI_URL"));
    return req.unwrap()[0]["id"].as_i32().unwrap();
}

pub fn create_kiwi_profile() -> i32 {
    let config = import_json_data("config");
    println!("{:?}", config);
    let req = Request::new("image.profile.create")
        .arg(read_env("UYUNI_KEY"))
        .arg(config["kiwi_profile"].clone())
        .arg("kiwi")
        .arg(config["image_store"].clone())
        .arg(config["profile_path"].clone())    
        .arg(config["activation_key"].clone())
        .call_url(dotenv!("UYUNI_URL").clone());
    return req.unwrap().as_i32().unwrap();
}


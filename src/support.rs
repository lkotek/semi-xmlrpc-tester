extern crate xmlrpc;
extern crate json;

use std::io;
use std::io::BufRead;
use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use xmlrpc::{Request, Value};

pub fn read_env(env_variable: &str) -> String {
    env::var(env_variable).unwrap().to_string()
}

pub fn call_server(xmlrpc_method: &str, key: Option<&str>) -> Value {
    let request = match key {
        None => Request::new(xmlrpc_method)
            .arg(dotenv!("MANAGER_USER"))
            .arg(dotenv!("MANAGER_PASS")),
        Some(val) => Request::new(xmlrpc_method).arg(val),
    };
    let result = request.call_url(dotenv!("MANAGER_URL"));
    return result.unwrap();
}

pub fn import_json_data() {
    let format_pathfile = format!("assets/{}/config.json", read_env("UPROFILE"));
    let pathfile = Path::new(&format_pathfile);
    println!("{:?}", pathfile.display());
    let mut file = match File::open(pathfile) {
        Err(why) => panic!("Cannot open file!"),
        Ok(file) => file,
    };
    let mut s = String::new();
    println!("{:?}", file.read_to_string(&mut s));

    let parse = json::parse(&s).unwrap();
    println!("{}", parse["activation_key"]);
}

pub fn get_system_id() {}

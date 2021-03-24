extern crate json;
extern crate xmlrpc;
extern crate chrono;
extern crate iso8601;

use json::JsonValue;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufRead;
use std::path::Path;
use xmlrpc::{Request, Value};
use std::time::SystemTime;
use chrono::DateTime;

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

pub fn read_text_file(text_file: &str) -> String {
    let format_pathfile = format!("assets/{}/{}", read_env("UYUNI_PROFILE"), text_file);
    let pathfile = Path::new(&format_pathfile);
    println!("{:?}", pathfile.display());
    let mut file = match File::open(pathfile) {
        Err(why) => panic!("Cannot open file!"),
        Ok(file) => file,
    };    
    let mut text_data = String::new();
    file.read_to_string(&mut text_data);
    return text_data;    
}

pub fn import_json_data(json_file: &str) -> HashMap<String, String> {
    let json_data = read_text_file(json_file);
    let parsed = json::parse(&json_data).unwrap();
    let mut parsed_data = HashMap::new();
    for (json_key, json_value) in parsed.entries() {
        parsed_data.insert(json_key.to_string(), json_value.to_string());
        env::set_var(
            format!("UYUNI_{}", json_key.to_string().to_uppercase()),
            json_value.to_string(),
        );
    }
    return parsed_data;
}

pub fn input() -> String {
   let mut input_string = String::new();   
   std::io::stdin().read_line(&mut input_string).unwrap();
   return input_string;
}

pub fn get_system_id(system_name: String) -> i32 {
    let req = Request::new("system.getId")
        .arg(read_env("UYUNI_KEY"))
        .arg(system_name)
        .call_url(dotenv!("UYUNI_URL"));
    return req.unwrap()[0]["id"].as_i32().unwrap();
}

pub fn exists_kiwi_profile() -> bool {
    let profiles = call_server("image.profile.listImageProfiles", Some(read_env("UYUNI_KEY")));
    for profile in profiles.as_array().unwrap(){
        if profile["label"].as_str().unwrap().contains(&read_env("UYUNI_KIWI_PROFILE")) {
            println!("Kiwi profile with name {} already exists.", read_env("UYUNI_KIWI_PROFILE"));
            return true;
        }
    }
    return false;
}

pub fn create_kiwi_profile() -> bool {
    let req = Request::new("image.profile.create")
        .arg(read_env("UYUNI_KEY"))
        .arg(read_env("UYUNI_KIWI_PROFILE"))
        .arg("kiwi")
        .arg(read_env("UYUNI_IMAGE_STORE"))
        .arg(read_env("UYUNI_PROFILE_PATH"))
        .arg(read_env("UYUNI_ACTIVATION_KEY"))
        .call_url(dotenv!("UYUNI_URL"));
    if req.unwrap().as_i32().unwrap() == 1 {
        println!("Kiwi profile with name {} created.", read_env("UYUNI_KIWI_PROFILE"));
        return true;
    }
    return false;    
}

pub fn delete_kiwi_profile() -> bool {
    let req = Request::new("image.profile.delete")
        .arg(read_env("UYUNI_KEY"))
        .arg(read_env("UYUNI_KIWI_PROFILE"))
        .call_url(dotenv!("UYUNI_URL"));
    if req.unwrap().as_i32().unwrap() == 1 {
        println!("Kiwi profile with name {} deleted.", read_env("UYUNI_KIWI_PROFILE"));
        return true;
    }
    return false;
}

pub fn exists_kiwi_image() -> HashMap<bool, i32> {
    let images = call_server("image.listImages", Some(read_env("UYUNI_KEY")));
    let mut image_status = HashMap::new();
    for image in images.as_array().unwrap(){
        if image["name"].as_str().unwrap().contains(&read_env("UYUNI_KIWI_PROFILE")) {
            println!("Building of image with name {} was already started.", read_env("UYUNI_KIWI_PROFILE"));
            image_status.insert(true, image["id"].as_i32().unwrap());
            return image_status;
        }
    }
    image_status.insert(false, -1);
    return image_status;
}

pub fn status_kiwi_image(image_id: i32) -> String {
    let req = Request::new("image.getDetails")
        .arg(read_env("UYUNI_KEY"))
        .arg(image_id)
        .call_url(dotenv!("UYUNI_URL"));
    return req.unwrap()["buildStatus"].as_str().unwrap().to_string();
}

pub fn schedule_kiwi_image() -> i32 {
    let now = iso8601::datetime(&chrono::offset::Utc::now().to_rfc3339()).unwrap();
    let req = Request::new("image.scheduleImageBuild")
        .arg(read_env("UYUNI_KEY"))
        .arg(read_env("UYUNI_KIWI_PROFILE"))
        .arg("")
        .arg(get_system_id(read_env("UYUNI_BUILD_HOST")))
        .arg(Value::from(now))
        .call_url(dotenv!("UYUNI_URL"));
    println!("Building of image with name {} started.", read_env("UYUNI_KIWI_PROFILE"));        
    return req.unwrap().as_i32().unwrap()
}

pub fn delete_kiwi_image(image_id: i32) -> bool {
    let req = Request::new("image.delete")
        .arg(read_env("UYUNI_KEY"))
        .arg(image_id)
        .call_url(dotenv!("UYUNI_URL"));
    if req.unwrap().as_i32().unwrap() == 1 {
        println!("Kiwi image with id {} deleted.", image_id);
        return true;
    }
    return false;
}

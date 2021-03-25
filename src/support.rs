extern crate chrono;
extern crate iso8601;
extern crate json;
extern crate xmlrpc;

use chrono::DateTime;
use json::JsonValue;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufRead;
use std::path::Path;
use std::time::SystemTime;
use xmlrpc::{Request, Value};

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



pub fn json_to_btree(parsed: &JsonValue) -> BTreeMap<String, Value> {
    //let parsed = &json::parse(&json_data).unwrap()[root]; // go thru child items next
                                                                    // !!! https://docs.rs/json/0.12.4/json/enum.JsonValue.html
    let mut map: BTreeMap<String, Value> = BTreeMap::new();
    for (json_key, json_value) in parsed.entries() {
        /*println!("{:?}", json_key);
        println!("{:?}, {:?}", json_value.len(), json_value);
        if json_value.is_string() {println!("SHORT")}
        if json_value.is_number() {println!("NUMBER")}*/
        map.insert(
            json_key.to_string(),
            if json_value.is_string() {
                Value::String(json_value.to_string())
            } else if json_value.is_number() {
                Value::Int(json_value.as_i32().unwrap())
            } else if json_value.is_array() {
                Value::Array(vec![Value::from(json_value.to_string())])                
            } else {
                Value::Struct(json_to_btree(json_value))
            }          
        );                    
    }
    //println!("{:?}", map);
    return map;
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
    let profiles = call_server(
        "image.profile.listImageProfiles",
        Some(read_env("UYUNI_KEY")),
    );
    for profile in profiles.as_array().unwrap() {
        if profile["label"]
            .as_str()
            .unwrap()
            .contains(&read_env("UYUNI_KIWI_PROFILE"))
        {
            println!(
                "Kiwi profile with name {} already exists.",
                read_env("UYUNI_KIWI_PROFILE")
            );
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
        println!(
            "Kiwi profile with name {} created.",
            read_env("UYUNI_KIWI_PROFILE")
        );
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
        println!(
            "Kiwi profile with name {} deleted.",
            read_env("UYUNI_KIWI_PROFILE")
        );
        return true;
    }
    return false;
}

pub fn exists_kiwi_image() -> HashMap<bool, i32> {
    let images = call_server("image.listImages", Some(read_env("UYUNI_KEY")));
    let mut image_status = HashMap::new();
    for image in images.as_array().unwrap() {
        if image["name"]
            .as_str()
            .unwrap()
            .contains(&read_env("UYUNI_KIWI_PROFILE"))
        {
            println!("Image with name {} exists.", read_env("UYUNI_KIWI_PROFILE"));
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
    println!(
        "Building of image with name {} started.",
        read_env("UYUNI_KIWI_PROFILE")
    );
    return req.unwrap().as_i32().unwrap();
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

pub fn has_buildhost_entitlement() -> bool {
    let entitlements = Request::new("system.getEntitlements")
        .arg(read_env("UYUNI_KEY"))
        .arg(get_system_id(read_env("UYUNI_BUILD_HOST")))
        .call_url(dotenv!("UYUNI_URL"));
    for entitlement in entitlements.unwrap().as_array().unwrap() {
        if entitlement.as_str().unwrap().contains("osimage_build_host") {
            return true;
        }
    }
    return false;
}

pub fn add_buildhost_entitlement() -> i32 {
    let req = Request::new("system.addEntitlements")
        .arg(read_env("UYUNI_KEY"))
        .arg(get_system_id(read_env("UYUNI_BUILD_HOST")))
        .arg(Value::Array(vec![Value::from("osimage_build_host")]))
        .call_url(dotenv!("UYUNI_URL"));
    println!(
        "Buildhost entitlement set for {}.",
        read_env("UYUNI_BUILD_HOST")
    );
    return req.unwrap().as_i32().unwrap();
}

pub fn schedule_highstate(system_name: String) -> i32 {
    let now = iso8601::datetime(&chrono::offset::Utc::now().to_rfc3339()).unwrap();
    let req = Request::new("system.scheduleApplyHighstate")
        .arg(read_env("UYUNI_KEY"))
        .arg(get_system_id(system_name.clone()))
        .arg(Value::from(now))
        .arg(false)
        .call_url(dotenv!("UYUNI_URL"));
    println!("Highstate for system {} scheduled.", system_name);
    return req.unwrap().as_i32().unwrap();
}

pub fn status_highstate(system_name: String, id: i32) -> i32 {
    let events = Request::new("system.listSystemEvents")
        .arg(read_env("UYUNI_KEY"))
        .arg(get_system_id(system_name.clone()))
        .call_url(dotenv!("UYUNI_URL"));
    for event in events.unwrap().as_array().unwrap() {
        if event["id"].as_i32().unwrap() == id {
            let failed = event["failed_count"].as_i32().unwrap();
            let success = event["successful_count"].as_i32().unwrap();
            if failed == 0 && success > 0 {
                return 1; // Success
            } else if failed == 1 {
                return -1; // Failure
            } else {
                return 0; // We don't know yet
            }
        }
    }
    return 0;
}

pub fn create_system_group(group_name: &str) -> i32 {
    let req = Request::new("systemgroup.create")
        .arg(read_env("UYUNI_KEY"))
        .arg(group_name)
        .arg(group_name)
        .call_url(dotenv!("UYUNI_URL"));
    println!("System group {} created.", group_name);
    return req.unwrap()["id"].as_i32().unwrap();
}

pub fn delete_system_group(group_name: &str) -> bool {
    let req = Request::new("systemgroup.delete")
        .arg(read_env("UYUNI_KEY"))
        .arg(group_name)
        .call_url(dotenv!("UYUNI_URL"));
    if req.unwrap().as_i32().unwrap() == 1 {
        println!("System group with name {} deleted.", group_name);
        return true;
    }
    return false;
}

pub fn exists_system_group(group_name: &str) -> bool {
    let system_groups = call_server("systemgroup.listAllGroups", Some(read_env("UYUNI_KEY")));
    for system_group in system_groups.as_array().unwrap() {
        if system_group["name"]
            .as_str()
            .unwrap()
            .contains(&read_env("UYUNI_HWTYPE_GROUP"))
        {
            println!(
                "System_group with name {} exists.",
                read_env("UYUNI_HWTYPE_GROUP")
            );
            return true;
        }
    }
    return false;
}

pub fn set_saltboot_formula(group_id: i32) -> i32 {
    let formula = Request::new("formula.setFormulasOfGroup")
        .arg(read_env("UYUNI_KEY"))
        .arg(group_id)
        .arg(Value::Array(vec![Value::from("saltboot")]))
        .call_url(dotenv!("UYUNI_URL"));

    let json_data = read_text_file("saltboot.json");
    let parsed = &json::parse(&json_data).unwrap()["partitioning"];
    println!("{:?}", json_to_btree(parsed));     
    let mut map = BTreeMap::new();
    map.insert("partitioning".to_string(), Value::Struct(json_to_btree(parsed)));
           
    let data = Request::new("formula.setGroupFormulaData")
        .arg(read_env("UYUNI_KEY"))
        .arg(group_id)
        .arg("saltboot")
        .arg(Value::Struct(map))
        .call_url(dotenv!("UYUNI_URL"));
    println!("Saltboot formula cofigured.");
    return data.unwrap().as_i32().unwrap();
}

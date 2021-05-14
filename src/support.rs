extern crate chrono;
extern crate iso8601;
extern crate json;
extern crate xmlrpc;

use json::JsonValue;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::{thread, time};
use xmlrpc::{Request, Value};

pub fn call_server(xmlrpc_method: &str, key: Option<String>) -> Value {
    let request = match key {
        None => Request::new(xmlrpc_method)
            .arg(read_env("UYUNI_USER"))
            .arg(read_env("UYUNI_PASS")),
        Some(val) => Request::new(xmlrpc_method).arg(val),
    };
    debug(format!("Uyuni server {:?} called.", read_env("UYUNI_URL")));
    let result = request.call_url(read_env("UYUNI_URL"));
    return result.unwrap();
}

pub fn read_env(env_variable: &str) -> String {
    env::var(env_variable).unwrap().to_string()
}

pub fn log(info: String, level: &str) {
    let now = chrono::Local::now();
    let info_levels = vec!["INFO", "ERROR", "WARNING"];
    if !read_env("UYUNI_LOG_LEVEL").contains("NO")
        && (read_env("UYUNI_LOG_LEVEL").contains(level) || info_levels.contains(&level))
    {
        println!("{} {}: {}", now.format("%F %T"), &level, &info);
    }
}

pub fn info(info: String) {
    log(info, "INFO");
}

pub fn warning(info: String) {
    log(info, "WARNING");
}

pub fn error(info: String) {
    log(info, "ERROR");
}

pub fn debug(info: String) {
    log(info, "DEBUG");
}

pub fn read_text_file(text_file: &str) -> String {
    let format_pathfile = format!("assets/{}/{}", read_env("UYUNI_PROFILE"), text_file);
    let pathfile = Path::new(&format_pathfile);
    debug(format!("File {:?} opened.", pathfile.display()));
    let mut file = match File::open(pathfile) {
        Err(reason) => panic!("Cannot open file, because {:?}", reason),
        Ok(file) => file,
    };
    let mut text_data = String::new();
    match file.read_to_string(&mut text_data) {
        Err(reason) => {
            panic!("Cannot read file, because {:?}", reason);
        }
        Ok(_) => return text_data,
    };
}

pub fn import_json_data(json_file: &str) -> HashMap<String, String> {
    let json_data = read_text_file(json_file);
    debug(format!("File {:?} opened.", &json_file));
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

pub fn json_array_to_xmlrpc(json_array: JsonValue) -> Vec<Value> {
    let mut map: Vec<Value> = Vec::new();
    for json_value in json_array.members() {
        map.push(if json_value.is_string() {
            Value::String(json_value.to_string())
        } else if json_value.is_number() {
            Value::Int(json_value.as_i32().unwrap())
        } else {
            Value::Bool(false)
        });
    }
    return map;
    // Approach to be considered: json_array.members().map(|item| json_to_btree(item));
}

pub fn json_to_btree(parsed: &JsonValue) -> BTreeMap<String, Value> {
    let mut map: BTreeMap<String, Value> = BTreeMap::new();
    for (json_key, json_value) in parsed.entries() {
        map.insert(
            json_key.to_string(),
            if json_value.is_string() {
                Value::String(json_value.to_string())
            } else if json_value.is_number() {
                Value::Int(json_value.as_i32().unwrap())
            } else if json_value.is_boolean() {
                Value::Bool(json_value.as_bool().unwrap())
            } else if json_value.is_array() {
                Value::Array(json_array_to_xmlrpc(json_value.clone()))
            } else {
                Value::Struct(json_to_btree(json_value))
            },
        );
    }
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
        .call_url(read_env("UYUNI_URL"));
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
            info(format!(
                "Profile with name {} exists.",
                read_env("UYUNI_KIWI_PROFILE")
            ));
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
        .call_url(read_env("UYUNI_URL"));
    if req.unwrap().as_i32().unwrap() == 1 {
        info(format!(
            "Kiwi profile with name {} created.",
            read_env("UYUNI_KIWI_PROFILE")
        ));
        return true;
    }
    return false;
}

pub fn delete_kiwi_profile() -> bool {
    let req = Request::new("image.profile.delete")
        .arg(read_env("UYUNI_KEY"))
        .arg(read_env("UYUNI_KIWI_PROFILE"))
        .call_url(read_env("UYUNI_URL"));
    if req.unwrap().as_i32().unwrap() == 1 {
        info(format!(
            "Kiwi profile with name {} deleted.",
            read_env("UYUNI_KIWI_PROFILE")
        ));
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
            warning(format!(
                "Image with name {} exists with unknown status.",
                read_env("UYUNI_KIWI_PROFILE")
            ));
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
        .call_url(read_env("UYUNI_URL"));
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
        .call_url(read_env("UYUNI_URL"));
    info(format!(
        "Building of image with name {} started.",
        read_env("UYUNI_KIWI_PROFILE")
    ));
    return req.unwrap().as_i32().unwrap();
}

pub fn delete_kiwi_image(image_id: i32) -> bool {
    let req = Request::new("image.delete")
        .arg(read_env("UYUNI_KEY"))
        .arg(image_id)
        .call_url(read_env("UYUNI_URL"));
    if req.unwrap().as_i32().unwrap() == 1 {
        info(format!("Kiwi image with id {} deleted.", image_id));
        return true;
    }
    return false;
}

pub fn has_buildhost_entitlement() -> bool {
    let entitlements = Request::new("system.getEntitlements")
        .arg(read_env("UYUNI_KEY"))
        .arg(get_system_id(read_env("UYUNI_BUILD_HOST")))
        .call_url(read_env("UYUNI_URL"));
    for entitlement in entitlements.unwrap().as_array().unwrap() {
        if entitlement.as_str().unwrap().contains("osimage_build_host") {
            warning(format!(
                "Buildhost entitlement was set already for {}.",
                read_env("UYUNI_BUILD_HOST")
            ));
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
        .call_url(read_env("UYUNI_URL"));
    info(format!(
        "Buildhost entitlement set for {}.",
        read_env("UYUNI_BUILD_HOST")
    ));
    return req.unwrap().as_i32().unwrap();
}

pub fn schedule_highstate(system_name: String) -> i32 {
    let now = iso8601::datetime(&chrono::offset::Utc::now().to_rfc3339()).unwrap();
    let req = Request::new("system.scheduleApplyHighstate")
        .arg(read_env("UYUNI_KEY"))
        .arg(get_system_id(system_name.clone()))
        .arg(Value::from(now))
        .arg(false)
        .call_url(read_env("UYUNI_URL"));
    info(format!(
        "Highstate for system {} scheduled (patience please).",
        system_name
    ));
    return req.unwrap().as_i32().unwrap();
}

pub fn status_highstate(system_name: String, id: i32) -> i32 {
    let events = Request::new("system.listSystemEvents")
        .arg(read_env("UYUNI_KEY"))
        .arg(get_system_id(system_name.clone()))
        .call_url(read_env("UYUNI_URL"));
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

pub fn wait_for_highstate(system_name: &str, event_id: i32, limit: u64, step_time: u64) {
    let step = time::Duration::from_secs(step_time);
    for i in 1..limit {
        thread::sleep(step);
        let status = status_highstate(system_name.to_string(), event_id);
        match status {
            0 => info(format!(
                "Highstate is still running after {} seconds.",
                i * step_time
            )),
            1 => {
                info(format!(
                    "Highstate was successfull after {} seconds.",
                    i * step_time
                ));
                break;
            }
            -1 => {
                error(format!("Highstate failed after {} seconds.", i * step_time));
                process::exit(1);
            }
            _ => warning(format!(
                "Better not to imagine what happened with highstate."
            )),
        }
    }
}

pub fn create_system_group(group_name: &str) -> i32 {
    let req = Request::new("systemgroup.create")
        .arg(read_env("UYUNI_KEY"))
        .arg(group_name)
        .arg(group_name)
        .call_url(read_env("UYUNI_URL"));
    info(format!("System group {} created.", group_name));
    return req.unwrap()["id"].as_i32().unwrap();
}

pub fn delete_system_group(group_name: &str) -> bool {
    let req = Request::new("systemgroup.delete")
        .arg(read_env("UYUNI_KEY"))
        .arg(group_name)
        .call_url(read_env("UYUNI_URL"));
    if req.unwrap().as_i32().unwrap() == 1 {
        info(format!("System group with name {} deleted.", group_name));
        return true;
    }
    return false;
}

pub fn exists_system_group(group_name: &str) -> bool {
    let system_groups = call_server("systemgroup.listAllGroups", Some(read_env("UYUNI_KEY")));
    for system_group in system_groups.as_array().unwrap() {
        if system_group["name"].as_str().unwrap().contains(&group_name) {
            warning(format!(
                "System_group with name {:?} exists.",
                system_group["name"].as_str().unwrap()
            ));
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
        .call_url(read_env("UYUNI_URL"));
    if formula.unwrap().as_i32().unwrap() == 1 {
        /* Parse data from json file and map it to XMLRPC data types */
        let json_data = read_text_file("saltboot.json");
        let parsed = &json::parse(&json_data).unwrap();
        debug(format!("{:?}", json_to_btree(parsed)));
        let data = Request::new("formula.setGroupFormulaData")
            .arg(read_env("UYUNI_KEY"))
            .arg(group_id)
            .arg("saltboot")
            .arg(Value::Struct(json_to_btree(parsed)))
            .call_url(read_env("UYUNI_URL"));
        info("Saltboot formula cofigured.".to_string());
        return data.unwrap().as_i32().unwrap();
    } else {
        return -1;
    }
}

pub fn set_system_formulas(system_id: i32, formulas: Vec<&str>) -> i32 {
    let mut formula_names: Vec<Value> = Vec::new();
    for formula in formulas {
        formula_names.push(Value::from(formula));
    }
    let req = Request::new("formula.setFormulasOfServer")
        .arg(read_env("UYUNI_KEY"))
        .arg(system_id)
        .arg(Value::Array(formula_names))
        .call_url(read_env("UYUNI_URL"));
    info("All formulas enabled for system, but not configured yet.".to_string());
    return req.unwrap().as_i32().unwrap();
}

pub fn set_system_formula_data(system_id: i32, formula_name: &str) -> i32 {
    let json_data = read_text_file(format!("{}.json", formula_name).as_str());
    let parsed = &json::parse(&json_data).unwrap();
    debug(format!("{:?}", json_to_btree(parsed)));
    let data = Request::new("formula.setSystemFormulaData")
        .arg(read_env("UYUNI_KEY"))
        .arg(system_id)
        .arg(formula_name)
        .arg(Value::Struct(json_to_btree(parsed)))
        .call_url(read_env("UYUNI_URL"));
    info(format!("*{:?}* formula cofigured.", formula_name));
    return data.unwrap().as_i32().unwrap();
}

pub fn clone_activation_key(key_name: &str) -> String {
    let req = Request::new("activationkey.clone")
        .arg(read_env("UYUNI_KEY"))
        .arg(read_env("UYUNI_CLONNED_KEY"))
        .arg(key_name)
        .call_url(read_env("UYUNI_URL"));
    info(format!("Activation key with name *{:?}* clonned.", key_name));
    return req.unwrap().as_str().unwrap().to_string();
}

use crate::support;

use std::env;
use std::process;
use std::{thread, time};

pub fn prepare_buildhost() {
    println!("INFO: Preparation of buildhost.");
    /* Prepare buildhost server */
    if !support::has_buildhost_entitlement() {
        support::add_buildhost_entitlement();
    }
    let event_id = support::schedule_highstate(support::read_env("UYUNI_BUILD_HOST"));
    support::wait_for_highstate(&support::read_env("UYUNI_BUILD_HOST"), event_id, 40, 30);
}

pub fn prepare_kiwi_profile() {
    println!("INFO: Preparation of kiwi profile.");
    /* Prepare Kiwi image profile and rewrite old one if necessary */
    if support::exists_kiwi_profile() {
        support::delete_kiwi_profile();
    }
    support::create_kiwi_profile();
}

pub fn build_kiwi_image() {
    println!("INFO: Building of kiwi image.");
    /* Building kiwi images */
    let image = support::exists_kiwi_image();
    if image.contains_key(&true) {
        // Image already exists tree
        let image_id = image[&true];
        let status = support::status_kiwi_image(image_id);
        match status.as_str() {
            "queued" | "picked up" | "completed" => {
                println!("Kiwi image status: {}. Do you wish to delete existing image (or building) and build again? [y, n]", status.as_str());
                if support::input().contains("y") {
                    support::delete_kiwi_image(image_id);
                    support::schedule_kiwi_image();
                }
            }
            "failed" => {
                support::delete_kiwi_image(image_id);
                support::schedule_kiwi_image();
            }
            _ => println!("Better not to imagine that."),
        }
    } else {
        support::schedule_kiwi_image();
    }
    // Image is being built tree
    let image = support::exists_kiwi_image();
    if image.contains_key(&true) {
        // Check status of image
        let step_time = 60;
        let step = time::Duration::from_secs(step_time);
        let image_id = image[&true];
        for i in 1..40 {
            thread::sleep(step);
            let status = support::status_kiwi_image(image_id);
            match status.as_str() {
                "queued" | "picked up" => println!(
                    "Kiwi image building is *{}* {} after seconds.",
                    status.as_str(),
                    i * step_time
                ),
                "completed" => {
                    println!(
                        "Kiwi image building is *{}* {} after seconds.",
                        status.as_str(),
                        i * step_time
                    );
                    break;
                }
                "failed" => {
                    println!(
                        "Kiwi image building failed after {} seconds.",
                        i * step_time
                    );
                    process::exit(1);
                }
                _ => println!("Better not to imagine that."),
            }
        }
    }
}

pub fn configure_saltboot() {
    println!("INFO: Configuration of salboot formula.");
    let hwgroup_name = support::read_env("UYUNI_HWTYPE_GROUP");
    if support::exists_system_group(&hwgroup_name) {
        support::delete_system_group(&hwgroup_name);
    }
    let hwgroup_id = support::create_system_group(&hwgroup_name);
    println!("{}", hwgroup_id);
    support::set_saltboot_formula(hwgroup_id);
}

pub fn configure_retail_formulas() {
    println!("INFO: Configuration of retail formulas.");
    let rbs_id = support::get_system_id(support::read_env("UYUNI_BRANCH_SERVER"));
    let formulas = vec![
        "branch-network",
        "dhcpd",
        "pxe",
        "tftpd",
        "vsftpd",
        "image-synchronize",
        "bind",
    ];
    support::set_system_formulas(rbs_id, formulas.clone());
    for formula in formulas {
        support::set_system_formula_data(rbs_id, formula);
    }
}

pub fn prepare_for_deployment() {
    println!("INFO: Preparing of groups and  higstate to branch server.");
    let system_groups = vec!["SERVERS", "TERMINALS", "id"];
    for group in system_groups {
        if support::exists_system_group(&group) {
            support::delete_system_group(&group);
        }
        let group_id = support::create_system_group(&group);
        println!("Group {:?} with id {:?} created.", &group, group_id);
    }
    let event_id = support::schedule_highstate(support::read_env("UYUNI_BRANCH_SERVER"));
    support::wait_for_highstate(&support::read_env("UYUNI_BRANCH_SERVER"), event_id, 20, 30);
}

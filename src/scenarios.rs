use crate::support;

use std::process;
use std::{thread, time};

pub fn prepare_buildhost() {
    support::info("Preparation of buildhost.".to_string());
    /* Prepare buildhost server */
    if !support::has_buildhost_entitlement() {
        support::add_buildhost_entitlement();
    }
    let event_id = support::schedule_highstate(support::read_env("UYUNI_BUILD_HOST"));
    support::wait_for_highstate(&support::read_env("UYUNI_BUILD_HOST"), event_id, 80, 15);
}

pub fn prepare_kiwi_profile() {
    support::info("STAGE Preparation of kiwi profile.".to_string());
    /* Prepare Kiwi image profile and rewrite old one if necessary */
    if support::exists_kiwi_profile() {
        support::delete_kiwi_profile();
    }
    support::create_kiwi_profile();
}

pub fn build_kiwi_image() {
    support::info("STAGE Building of kiwi image.".to_string());
    /* Building kiwi images */
    let image = support::exists_kiwi_image();
    if image.contains_key(&true) {
        // Image already exists tree
        let image_id = image[&true];
        let status = support::status_kiwi_image(image_id);
        match status.as_str() {
            "queued" | "picked up" | "completed" => {
                support::info(format!("Kiwi image status: *{}*.", status.as_str()));
                support::info("Do you wish to delete (or cancel process of) existing image and build again? [y, n]".to_string());
                if support::read_env("UYUNI_YES") == "yes" || support::input().contains("y") {
                    support::delete_kiwi_image(image_id);
                    support::schedule_kiwi_image();
                } else {
                    process::exit(0);
                }
            }
            "failed" => {
                support::delete_kiwi_image(image_id);
                support::schedule_kiwi_image();
            }
            _ => {
                support::error(format!(
                    "Better not to imagine what happened to poor kiwi image."
                ));
                process::exit(1);
            }
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
                "queued" | "picked up" => support::info(format!(
                    "Kiwi image building is *{}* {} after seconds.",
                    status.as_str(),
                    i * step_time
                )),
                "completed" => {
                    support::info(format!(
                        "Kiwi image building is *{}* {} after seconds.",
                        status.as_str(),
                        i * step_time
                    ));
                    break;
                }
                "failed" => {
                    support::error(format!(
                        "Kiwi image building failed after {} seconds.",
                        i * step_time
                    ));
                    process::exit(1);
                }
                _ => {
                    support::error(format!("Better not to imagine that."));
                    process::exit(1);
                }
            }
        }
    }
}

pub fn configure_saltboot() {
    support::info("STAGE Configuration of salboot formula.".to_string());
    let hwgroup_name = support::read_env("UYUNI_HWTYPE_GROUP");
    if support::exists_system_group(&hwgroup_name) {
        support::delete_system_group(&hwgroup_name);
    }
    let hwgroup_id = support::create_system_group(&hwgroup_name);
    support::set_saltboot_formula(hwgroup_id);
}

pub fn configure_retail_formulas() {
    support::info("STAGE Configuration of retail formulas.".to_string());
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
    support::info("INFO: Preparing of groups and  higstate to branch server.".to_string());
    let system_groups = vec!["SERVERS", "TERMINALS", "id"];
    for group in system_groups {
        if support::exists_system_group(&group) {
            support::delete_system_group(&group);
        }
        let group_id = support::create_system_group(&group);
        support::info(format!(
            "Group {:?} with id {:?} created.",
            &group, group_id
        ));
    }
    let event_id = support::schedule_highstate(support::read_env("UYUNI_BRANCH_SERVER"));
    support::wait_for_highstate(&support::read_env("UYUNI_BRANCH_SERVER"), event_id, 20, 30);
}

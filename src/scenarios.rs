use crate::support;

use std::env;
use std::process;
use std::{thread, time};

pub fn build_kiwi_image() {
    /* Prepare buildhost server */
    if !support::has_buildhost_entitlement() {
        support::add_buildhost_entitlement();
    }
    let event_id = support::schedule_highstate(support::read_env("UYUNI_BUILD_HOST"));
    // Check status of highstate
    let step_time = 30;
    let step = time::Duration::from_secs(step_time);
    for i in 1..10 {
        thread::sleep(step);
        let status = support::status_highstate(support::read_env("UYUNI_BUILD_HOST"), event_id);
        match status {
            0 => println!(
                "Highstate is still running {} after seconds.",
                i * step_time
            ),
            1 => {
                println!("Highstate was successfull after {} seconds.", i * step_time);
                break;
            }
            -1 => {
                println!("Highstate failed after {} seconds.", i * step_time);
                process::exit(1);
            }
            _ => println!("Better not to imagine that."),
        }
    }

    /* Prepare Kiwi image profile and rewrite old one if necessary */
    if support::exists_kiwi_profile() {
        support::delete_kiwi_profile();
    }
    support::create_kiwi_profile();

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

pub fn configure_salboot() {}

pub fn configure_retail_formulas() {}

pub fn prepare_terminal_deployment() {}

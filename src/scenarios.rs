use crate::support;

use std::env;
use std::process;
use std::{thread, time};

pub fn build_kiwi_image() {
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
            },
            "failed" => {
                support::delete_kiwi_image(image_id);
                support::schedule_kiwi_image();
            },
            _ => println!("Better not to imagine that."),
        }
    } else {
        support::schedule_kiwi_image();
    }
    // Image is being built tree
    let image = support::exists_kiwi_image();
    if image.contains_key(&true) {
        // Check status of image
        let step = time::Duration::from_secs(30);
        let image_id = image[&true];
        for i in 1..20{            
            let status = support::status_kiwi_image(image_id);
            match status.as_str() {
                "queued" | "picked up"  => println!("Kiwi image building is still in progress after {} seconds.", i*30),
                "completed"             => println!("Kiwi image was successfully built after {} seconds.", i*30),
                "failed"                => {
                    println!("Kiwi image building failed after {} seconds.", i*30);
                    process::exit(1);
                }
                _ => println!("Better not to imagine that."),
            }
            thread::sleep(step);
        } 
    } 
}

pub fn configure_salboot() {}

pub fn configure_retail_formulas() {}

pub fn prepare_terminal_deployment() {}

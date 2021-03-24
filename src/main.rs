extern crate xmlrpc;
#[macro_use]
extern crate dotenv_codegen;

mod scenarios;
mod support;

use std::env;
use std::process;
use xmlrpc::{Request, Value};

fn full_retail_deploy() {
    //scenarios::build_kiwi_image();
    //support::build_kiwi_image()
}

fn basic_tests() {
    println!(
        "SYSTEM_ID: {:?}",
        support::get_system_id(support::read_env("UYUNI_BUILD_HOST"))
    );
    let users_list = support::call_server("user.list_users", Some(support::read_env("UYUNI_KEY")));
    for user in users_list.as_array().unwrap() {
        println!("{:?}", user);
    }
    support::has_buildhost_entitlement();
    support::add_buildhost_entitlement();
    let id = support::schedule_highstate(support::read_env("UYUNI_BUILD_HOST"));
    println!("{:?}", id);
    support::status_highstate(support::read_env("UYUNI_BUILD_HOST"), id);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    env::set_var("UYUNI_PROFILE", &args[1]);
    support::import_json_data("config.json");

    let key = support::call_server("auth.login", None);
    println!("{:?}", key);
    env::set_var("UYUNI_KEY", key.as_str().unwrap());

    if args.len() != 3 {
        println!("Incorrect number of arguments passed.");
        process::exit(1);
    }
    match args[2].as_str() {
        "full_retail" => full_retail_deploy(),
        "basic_tests" => basic_tests(),
        "image" => scenarios::build_kiwi_image(),
        _ => {
            println!("Incorrect argument string passed.");
            process::exit(1);
        }
    }
}

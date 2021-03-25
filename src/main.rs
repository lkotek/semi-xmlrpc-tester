extern crate xmlrpc;
#[macro_use]
extern crate dotenv_codegen;

mod scenarios;
mod support;

use std::env;
use std::process;

fn full_retail_deploy() {
    scenarios::configure_retail_formulas();
    scenarios::prepare_buildhost();
    scenarios::prepare_kiwi_profile();
    scenarios::build_kiwi_image();
    scenarios::configure_saltboot();
    scenarios::prepare_for_deployment();
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
}

fn main() {
    let args: Vec<String> = env::args().collect();
    env::set_var("UYUNI_PROFILE", &args[1]);
    support::import_json_data("config.json");

    let key = support::call_server("auth.login", None);
    println!("Logged in with key {:?}", key.as_str().unwrap());
    env::set_var("UYUNI_KEY", key.as_str().unwrap());

    if args.len() != 3 {
        println!("Incorrect number of arguments passed.");
        process::exit(1);
    }
    match args[2].as_str() {
        "basic_tests" => basic_tests(),
        "formulas" => scenarios::configure_retail_formulas(),
        "full" => full_retail_deploy(),
        "buildhost" => scenarios::prepare_buildhost(),
        "profiles" => scenarios::prepare_kiwi_profile(),
        "image" => scenarios::build_kiwi_image(),
        "saltboot" => scenarios::configure_saltboot(),
        "prepare" => scenarios::prepare_for_deployment(),
        _ => {
            println!("Incorrect argument string passed.");
            process::exit(1);
        }
    }
    support::call_server("auth.logout", Some(support::read_env("UYUNI_KEY")));
}

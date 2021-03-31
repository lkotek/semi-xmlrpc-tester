extern crate xmlrpc;

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
    let users_list = support::call_server("user.list_users", Some(support::read_env("UYUNI_KEY")));
    for user in users_list.as_array().unwrap() {
        support::debug(format!("{:?}", user));
    }
    support::info(format!(
        "Logged in with key {:?}",
        support::read_env("UYUNI_KEY")
    ));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    env::set_var("UYUNI_PROFILE", &args[1]);

    if args.len() < 3 {
        println!("Incorrect number of arguments passed. Closing.");
        process::exit(1);
    }
    if args.contains(&"-y".to_string()) {
        env::set_var("UYUNI_YES", "yes"); // Answer all script question as yes
    } else {
        env::set_var("UYUNI_YES", "no");
    }
    if args.contains(&"--debug".to_string()) {
        env::set_var("UYUNI_LOG_LEVEL", "DEBUG");
    } else if args.contains(&"--silent".to_string()) {
        env::set_var("UYUNI_LOG_LEVEL", "NO");
    } else {
        env::set_var("UYUNI_LOG_LEVEL", "INFO");
    }
    support::info(format!(
        "Log level set to {}.",
        support::read_env("UYUNI_LOG_LEVEL")
    ));
    support::import_json_data("config.json");
    
    let key = support::call_server("auth.login", None);
    env::set_var("UYUNI_KEY", key.as_str().unwrap());
        
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
            support::error("Incorrect argument string passed.".to_string());
            process::exit(1);
        }
    }
    support::call_server("auth.logout", Some(support::read_env("UYUNI_KEY")));
}

extern crate xmlrpc;
#[macro_use]
extern crate dotenv_codegen;

mod scenarios;
mod support;

use std::env;
use std::process;
use xmlrpc::{Request, Value};

fn full_deploy() {
    println!("Sorry to say that, but there is nothing I can do right now.");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    env::set_var("UYUNI_PROFILE", &args[1]);    

    if args.len() != 3 {
        println!("Incorrect number of arguments passed.");
        process::exit(1);
    }
    match args[2].as_str() {
        "full"  => full_deploy(),
        _       => {
            println!("Incorrect argument string passed.");
            process::exit(1);
        }
    }

    let key = support::call_server("auth.login", None);
    println!("{:?}", key);
    env::set_var("UYUNI_KEY", key.as_str().unwrap());

    println!("SYSTEM_ID: {:?}", support::get_system_id("suma-bv-41-build-sles15sp2.mgr.prv.suse.net"));

    /*
    let users_list = support::call_server("user.list_users", Some(support::read_env("UYUNI_KEY")));
    println!("{:?}", users_list);

    for user in users_list.as_array().unwrap() {
        println!("{:?}", user);
    }*/

    //println!("{:?}", support::import_json_data("config"));
    println!("{:?}", support::create_kiwi_profile());
}

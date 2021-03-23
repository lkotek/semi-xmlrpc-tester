extern crate xmlrpc;
#[macro_use]
extern crate dotenv_codegen;

mod scenarios;
mod support;

use std::env;
use xmlrpc::{Request, Value};

fn main() {
    let args: Vec<String> = env::args().collect();
    env::set_var("UPROFILE", &args[1]);    
    let key = support::call_server("auth.login", None);
    println!("{:?}", key);

    let users_list = support::call_server("user.list_users", key.as_str());
    println!("{:?}", users_list);

    for user in users_list.as_array().unwrap() {
        println!("{:?}", user);
    }

    println!("{:?}", support::import_json_data());
}

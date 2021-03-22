extern crate xmlrpc;

#[macro_use]
extern crate dotenv_codegen;

use xmlrpc::{Request, Value};

fn call_server(xmlrpc_method: &str, key: Option<&str>) -> Value {
    let request = match key {
        None => Request::new(xmlrpc_method)
            .arg(dotenv!("MANAGER_USER"))
            .arg(dotenv!("MANAGER_PASS")),
        Some(val) => Request::new(xmlrpc_method).arg(val),
    };
    let result = request.call_url(dotenv!("MANAGER_URL"));
    return result.unwrap();
}

fn main() {
    let key = call_server("auth.login", None);
    println!("{:?}", key);

    let users_list = call_server("user.list_users", key.as_str());
    println!("{:?}", users_list);

    for user in users_list.as_array().unwrap() {
        println!("{:?}", user);
    }
}

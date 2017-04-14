extern crate dotenv;
extern crate iron;

use iron::prelude::*;
use iron::status;

use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    let port = match env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3000".to_string(),
    };
    let client_id = env::var("INSTAGRAM_CLIENT_ID").expect("lack of instagram client id.");

    Iron::new(|_: &mut Request| Ok(Response::with((status::Ok, "Hello world!"))))
        .http(format!("localhost:{}", port))
        .expect("Server start process is failed.");

    println!("{}-{}", port, client_id);
}


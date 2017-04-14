extern crate dotenv;
extern crate hyper;

use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    let port = match env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3000".to_string(),
    };
    let client_id = env::var("INSTAGRAM_CLIENT_ID").expect("lack of instagram client id.");

    println!("{}-{}", port, client_id);
}


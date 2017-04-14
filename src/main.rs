extern crate dotenv;
extern crate iron;

use iron::prelude::*;
use iron::status;

use dotenv::dotenv;
use std::env;

static INSTAGRAM_OAUTH_URI: &str = "https://api.instagram.com/oauth/authorize/";
static REDIRECT_URI: &str = "https://bento-photo.herokuapp.com/";

fn main() {
    dotenv().ok();

    let port = match env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3000".to_string(),
    };

    let client_id = env::var("INSTAGRAM_CLIENT_ID").expect("lack of instagram client id.");
    println!("{}?client_id={}&redirect_uri={}&response_type=code",
             INSTAGRAM_OAUTH_URI,
             client_id,
             REDIRECT_URI);

    println!("Server start on {}", port);

    Iron::new(|_: &mut Request| Ok(Response::with((status::Ok, "Hello world!"))))
        .http(format!("localhost:{}", port))
        .expect("Server start process is failed.");
}


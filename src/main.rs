extern crate dotenv;
extern crate iron;
extern crate handlebars;
extern crate handlebars_iron as hbs;
#[macro_use]
extern crate router;
#[cfg(not(feature = "serde_type"))]
extern crate rustc_serialize;
extern crate mount;
extern crate staticfile;
extern crate reqwest;

use iron::prelude::*;
use iron::modifiers::Redirect;
use iron::{Url, status};
use hbs::{Template, HandlebarsEngine, DirectorySource};
use rustc_serialize::json::{Json};
use staticfile::Static;
use mount::Mount;

use dotenv::dotenv;
use std::env;
use std::collections::BTreeMap;
use std::path::Path;
use std::io::Read;

static INSTAGRAM_OAUTH_URI: &'static str = "https://api.instagram.com/oauth/authorize/";
static REDIRECT_URI: &'static str = "https://bento-photo.herokuapp.com/";
static GRANT_TYPE: &'static str = "authorization_code";

fn main() {
    dotenv().ok();

    let port = match env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3000".to_string(),
    };

    let client_id = env::var("INSTAGRAM_CLIENT_ID").expect("lack of instagram client id.");
    let client_secret = env::var("INSTAGRAM_CLIENT_SECRET").expect("lack of instagram client secret.");
    let authorization_uri = format!("{}?client_id={}&redirect_uri={}&response_type=code",
                                    INSTAGRAM_OAUTH_URI,
                                    client_id,
                                    REDIRECT_URI);

    let router = router!(
        index: get "/" => move |req: &mut Request| {
            match req.url.query() {
                Some(query) => {
                    let code = query.split("=").last().expect("query parsing is failed").to_string();
                    let mut buffer = String::new();
                    
                    let params = [
                        ("client_id", client_id.clone()),
                        ("client_secret", client_secret.clone()),
                        ("grant_type", GRANT_TYPE.clone().to_string()),
                        ("redirect_uri", REDIRECT_URI.clone().to_string()),
                        ("code", code.to_string())
                    ];

                    println!("{:?}", params);

                    let client = reqwest::Client::new().expect("Create HTTP client is failed");
                    client.post("https://api.instagram.com/oauth/access_token")
                        .form(&params)
                        .send()
                        .expect("send Request failed")
                        .read_to_string(&mut buffer)
                        .expect("read Response failed");
                    
                    println!("{}", buffer);

                    let mut resp = Response::new();
                    let data = BTreeMap::<String, Json>::new();
                    resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
                    Ok(resp)
                },
                None => {
                    let mut resp = Response::new();
                    let data = BTreeMap::<String, Json>::new();
                    resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
                    Ok(resp)
                },
            }
        },
        oauth: get "/:oauth" => move |_: &mut Request| {
            Ok(Response::with((status::Found, Redirect(
                Url::parse(authorization_uri.as_str()).expect(format!("authorization_uri is invalid => {}", authorization_uri).as_str())
            ))))
        },
    );

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));
    hbse.reload().expect("template can't reload collectory.");

    let mut mount = Mount::new();
    mount
        .mount("/css", Static::new(Path::new("assets/css")))
        .mount("/js", Static::new(Path::new("assets/js")))
        .mount("/", router);

    let mut chain = Chain::new(mount);
    chain.link_after(hbse);

    println!("Server start on {}", port);
    Iron::new(chain).http(format!("0.0.0.0:{}", port)).expect("Server start process is failed.");
}


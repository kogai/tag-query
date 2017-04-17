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
extern crate serde_json;
extern crate iron_sessionstorage;
extern crate urlencoded;

use iron::prelude::*;
use iron::headers::ContentType;
use iron::modifiers::Redirect;
use iron::{Url, status};
use hbs::{Template, HandlebarsEngine, DirectorySource};
use rustc_serialize::json::{Json};
use staticfile::Static;
use mount::Mount;
use serde_json::Value;
use iron_sessionstorage::traits::*;
use iron_sessionstorage::SessionStorage;
use iron_sessionstorage::backends::SignedCookieBackend;
use urlencoded::UrlEncodedQuery;

use dotenv::dotenv;
use std::env;
use std::io::Read;
use std::collections::BTreeMap;
use std::path::Path;
use std::collections::HashMap;

static INSTAGRAM_OAUTH_URI: &'static str = "https://api.instagram.com/oauth/authorize/";
static GRANT_TYPE: &'static str = "authorization_code";

fn value_to_json(x: Value) -> Json {
    match x {
        Value::Number(ref x) if x.is_i64() => Json::I64(x.as_i64().unwrap()),
        Value::Number(ref x) if x.is_u64() => Json::U64(x.as_u64().unwrap()),
        Value::Number(ref x) if x.is_f64() => Json::F64(x.as_f64().unwrap()),
        Value::String(x) => Json::String(x),
        Value::Array(x) => Json::Array(x
            .into_iter()
            .map(|x| value_to_json(x))
            .collect::<Vec<Json>>()
        ),
        Value::Object(x) => {
            let mut buf = BTreeMap::<String, Json>::new();
            for (key, value) in x.into_iter() {
                buf.insert(key, value_to_json(value));
            }
            Json::Object(buf)
        },
        Value::Bool(x) => Json::Boolean(x),
        _ => Json::Null,
    }
}

#[derive(Debug)]
struct AccessToken(String);

impl iron_sessionstorage::Value for AccessToken {
    fn get_key() -> &'static str { "access_token" }
    fn into_raw(self) -> String { self.0 }
    fn from_raw(value: String) -> Option<Self> {
        Some(AccessToken(value))
    }
}

fn main() {
    dotenv().ok();

    let port = match env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3000".to_string(),
    };

    let redirect_url = env::var("REDIRECT_URL").expect("lack of redirect url.");
    let client_id = env::var("INSTAGRAM_CLIENT_ID").expect("lack of instagram client id.");
    let client_secret = env::var("INSTAGRAM_CLIENT_SECRET").expect("lack of instagram client secret.");
    let authorization_uri = format!("{}?client_id={}&redirect_uri={}&response_type=code&scope={}",
                                    INSTAGRAM_OAUTH_URI,
                                    client_id,
                                    redirect_url,
                                    "public_content".to_string());

    let router = router!(
        index: get "/" => move |req: &mut Request| {
            match req.url.clone().query() {
                Some(query) => {
                    let code = query.split("=").last().expect("query parsing is failed").to_string();
                    let params = [
                        ("client_id", client_id.clone()),
                        ("client_secret", client_secret.clone()),
                        ("grant_type", GRANT_TYPE.clone().to_string()),
                        ("redirect_uri", redirect_url.clone()),
                        ("code", code.to_string())
                    ];

                    let http_client = reqwest::Client::new().expect("Create HTTP client is failed");
                    let mut result = http_client.post("https://api.instagram.com/oauth/access_token")
                        .form(&params)
                        .send()
                        .expect("send Request failed");

                    let result_json = result.json::<HashMap<String, Value>>().expect("Parse JSON failed");
                    let data = match result_json.get("access_token") {
                        Some(at) => {
                            let access_token = at.as_str().unwrap();
                            req.session().set(AccessToken(access_token.to_string())).unwrap();

                            let url = format!("https://api.instagram.com/v1/tags/nofilter/media/recent?access_token={}", access_token);

                            let result = http_client
                                .get(url.as_str())
                                .send()
                                .expect("send Request failed")
                                .json::<HashMap<String, Value>>()
                                .expect("Parse JSON failed");

                            let mut buffer = HashMap::<String, Json>::new();
                            for (key, value) in result.into_iter() {
                                buffer.insert(key, value_to_json(value));
                            }
                            buffer
                        },
                        None => HashMap::<String, Json>::new(),
                    };

                    let mut resp = Response::new();
                    resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
                    Ok(Response::with((status::Found, Redirect(
                        Url::parse(redirect_url.as_str()).expect("parse url failed")
                    ))))
                },
                None => {
                    let mut resp = Response::new();
                    let data = BTreeMap::<String, Json>::new();
                    resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
                    Ok(resp)
                },
            }
        },
        oauth: get "/oauth" => move |_: &mut Request| {
            Ok(Response::with((status::Found, Redirect(
                Url::parse(authorization_uri.as_str()).expect(format!("authorization_uri is invalid => {}", authorization_uri).as_str())
            ))))
        },
        api_username: get "/api/username" => move |req: &mut Request| {
            let username = match req.url.clone().query() {
                Some(query) => query.split("=").last().expect("query parsing is failed"),
                _ => ""
            }.to_string();
            
            let access_token = match try!(req.session().get::<AccessToken>()) {
                Some(y) => y.0,
                None => "Access token is Not Found".to_string(),
            };
            
            if access_token.len() == 0 {
                return Ok(Response::with((ContentType::json().0, status::Ok, "{}")))
            };

            let url = format!("https://api.instagram.com/v1/users/search?q={}&access_token={}", username, access_token.to_string());

            let http_client = reqwest::Client::new().expect("Create HTTP client is failed");
            let mut buffer = String::new();
            http_client
                .get(url.as_str())
                .send()
                .expect("send Request failed")
                .read_to_string(&mut buffer)
                .expect("read JSON string failed")
                ;

            Ok(Response::with((ContentType::json().0, status::Ok, buffer)))
        },

        api_hashtag: get "/api/hashtag" => move |req: &mut Request| {
            fn get_query(x: Option<&Vec<String>>) -> &str {
                match x {
                    Some(y) => match y.first() {
                        Some(z) => z.as_str(),
                        None => "",
                    },
                    None => "",
                }
            }
            let access_token = match try!(req.session().get::<AccessToken>()) {
                Some(y) => y.0,
                None => "Access token is Not Found".to_string(),
            };
            
            let (user_id, hashtag) = match req.get_ref::<UrlEncodedQuery>() {
                Ok(queries) => (get_query(queries.get("user_id")), get_query(queries.get("hashtag"))),
                _ => ("", "")
            };
            
            let url = format!(
                "https://api.instagram.com/v1/users/{}/media/recent/?access_token={}",
                user_id.to_string(),
                access_token.to_string()
            );

            let http_client = reqwest::Client::new().expect("Create HTTP client is failed");
            let mut buffer = String::new();
            http_client
                .get(url.as_str())
                .send()
                .expect("send Request failed")
                .read_to_string(&mut buffer)
                .expect("read JSON string failed")
                ;

            Ok(Response::with((ContentType::json().0, status::Ok, buffer)))
        }
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
    let session = SessionStorage::new(SignedCookieBackend::new(b"my_cookie_secret".to_vec()));

    chain.link_around(session);
    chain.link_after(hbse);

    println!("Server start on {}", port);
    Iron::new(chain).http(format!("0.0.0.0:{}", port)).expect("Server start process is failed.");
}

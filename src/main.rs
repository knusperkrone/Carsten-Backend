#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate reqwest;
extern crate scraper;
extern crate serde;
extern crate serde_json;

mod error;
mod spotify;
mod youtube;

use spotify::{CreateTokenRequest, RefreshTokenRequest};

use rocket::request::Form;
use rocket::response::status::BadRequest;
use rocket_contrib::json::JsonValue;

#[get("/callback")]
fn login_callback() -> &'static str {
    "<!DOCTYPE html><html><head></head><body></body></html>" // Ingore token and state
}

#[post("/create", data = "<code>")]
fn create_token(code: Form<CreateTokenRequest>) -> Result<JsonValue, BadRequest<JsonValue>> {
    match spotify::create_token(code.into_inner()) {
        Ok(resp) => Ok(json!(resp)),
        Err(e_resp) => {
            println!("Invalid token creation: {}", e_resp);
            Err(BadRequest(Some(json!(e_resp))))
        }
    }
}

#[post("/refresh", data = "<token>")]
fn refresh_token(token: Form<RefreshTokenRequest>) -> Result<JsonValue, BadRequest<JsonValue>> {
    match spotify::refresh_token(token.into_inner()) {
        Ok(resp) => Ok(json!(resp)),
        Err(e_resp) => {
            println!("Invalid token refresh: {}", e_resp);
            Err(BadRequest(Some(json!(e_resp))))
        }
    }
}

#[get("/search?<q>")]
fn search(q: String) -> Result<JsonValue, BadRequest<JsonValue>> {
    match youtube::search(q) {
        Ok(resp) => Ok(json!(resp)),
        Err(e_resp) => {
            println!("Invalid search: {}", e_resp);
            Err(BadRequest(Some(json!(e_resp))))
        }
    }
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!(error::ErrorResponse::new(
        String::from("error"),
        String::from("Ressource not found")
    ))
}

#[catch(422)]
fn invalid_form() -> JsonValue {
    json!(error::ErrorResponse::new(
        String::from("error"),
        String::from("Invalid form")
    ))
}

fn main() {
    rocket::ignite()
        .mount("/api/youtube", routes![search])
        .mount("/api/spotify", routes![login_callback, create_token, refresh_token])
        .register(catchers![not_found, invalid_form])
        .launch();
}

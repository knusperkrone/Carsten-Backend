#![feature(proc_macro_hygiene, decl_macro)]

extern crate chrono;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate reqwest;
extern crate rocket_slog;
extern crate scraper;
extern crate serde;
extern crate serde_json;
extern crate sloggers;
#[macro_use]
extern crate slog;
extern crate once_cell;

mod error;
mod logging;
mod spotify;
mod youtube;

use spotify::{CreateTokenRequest, RefreshTokenRequest};

use chrono::Utc;
use std::error::Error;
use std::io::Cursor;

use rocket::config::{LoggingLevel, RocketConfig};
use rocket::http::{ContentType, Status};
use rocket::request::Form;
use rocket::response::status::BadRequest;
use rocket::response::Response;
use rocket_contrib::json::JsonValue;
use rocket_slog::SlogFairing;

use sloggers::{
    file::FileLoggerBuilder,
    terminal::{Destination, TerminalLoggerBuilder},
    types::Severity,
    Build,
};

#[get("/")]
fn root() -> Result<Response<'static>, Status> {
    Response::build()
        .header(ContentType::HTML)
        .sized_body(Cursor::new(
            "<!DOCTYPE html><html><head></head><body></body></html>",
        ))
        .ok()
}

#[get("/robots.txt")]
fn robots() -> &'static str {
    ""
}

#[get("/callback")]
fn login_callback() -> Result<Response<'static>, Status> {
    Response::build()
        .header(ContentType::HTML)
        .sized_body(Cursor::new(
            "<!DOCTYPE html><html><head></head><body></body></html>",
        ))
        .ok()
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
fn search(q: String) -> Result<Response<'static>, BadRequest<JsonValue>> {
    match youtube::search(q) {
        Ok(resp) => {
            let resp_json = serde_json::to_string(&resp).unwrap();
            Response::build()
                .header(ContentType::JSON)
                .header(ContentType::JSON)
                .header(ContentType::JSON)
                .raw_header("Access-Control-Allow-Origin", "*")
                .sized_body(Cursor::new(resp_json))
                .ok()
        }
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

fn main() -> Result<(), Box<dyn Error>> {
    let fairing: SlogFairing;
    if cfg!(debug_assertions) {
        // Terminal logging
        let mut builder = TerminalLoggerBuilder::new();
        builder.level(Severity::Debug);
        builder.destination(Destination::Stderr);
        let logger = builder.build()?;
        fairing = SlogFairing::new(logger);
    } else {
        // File logging
        let web_file = format!(
            "./private/log_web_{}.txt",
            Utc::now().format("%d-%m-%Y_%H:%M")
        );
        let builder = FileLoggerBuilder::new(web_file);
        let weblogger = builder.build()?;
        fairing = SlogFairing::new(weblogger);
    }

    let mut config = RocketConfig::read().unwrap().active().clone();
    config.set_log_level(LoggingLevel::Off);
    rocket::custom(config.clone())
        .attach(fairing)
        .mount("/", routes![root, robots])
        .mount("/api/youtube", routes![search])
        .mount(
            "/api/spotify",
            routes![login_callback, create_token, refresh_token],
        )
        .register(catchers![not_found, invalid_form])
        .launch();
    Ok(())
}

#[macro_use]
extern crate slog;

mod error;
mod logging;
mod spotify;
mod youtube;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};

use crate::logging::APP_LOGGING;
use crate::spotify::{CreateTokenRequest, RefreshTokenRequest};
use actix_cors::Cors;

async fn root() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body("<!DOCTYPE html><html><head></head><body></body></html>")
}

async fn robots() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body("")
}

async fn spotify_login_callback() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body("<!DOCTYPE html><html><head></head><body></body></html>")
}

async fn spotify_create_token(code: web::Form<CreateTokenRequest>) -> HttpResponse {
    match spotify::create_token(code.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(resp) => {
            println!("Invalid token creation: {}", resp);
            HttpResponse::BadRequest().json(resp)
        }
    }
}

async fn spotify_refresh_token(token: web::Form<RefreshTokenRequest>) -> HttpResponse {
    match spotify::refresh_token(token.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(resp) => {
            warn!(APP_LOGGING, "Invalid token refresh: {}", resp);
            HttpResponse::BadRequest().json(resp)
        }
    }
}

#[derive(serde::Deserialize)]
struct SearchParams {
    q: String,
}

async fn youtube_search(web::Query(params): web::Query<SearchParams>) -> HttpResponse {
    match youtube::search(params.q).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(resp) => {
            warn!(APP_LOGGING, "Invalid search: {}", resp);
            HttpResponse::BadRequest().json(resp)
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let bind_addr: &str;
    if let Some(addr) = args.get(1) {
        bind_addr = addr;
    } else {
        bind_addr = "0.0.0.0:8443";
    }
    info!(APP_LOGGING, "Binding to address: {}", bind_addr);

    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .data(web::JsonConfig::default().limit(4096))
            .wrap(middleware::Logger::default())
            .wrap(Cors::default())
            .service(web::resource("/").route(web::get().to(root)))
            .service(web::resource("/robots.txt").route(web::get().to(robots)))
            .service(
                web::resource("/api/spotify/callback").route(web::get().to(spotify_login_callback)),
            )
            .service(
                web::resource("/api/spotify/create").route(web::post().to(spotify_create_token)),
            )
            .service(
                web::resource("/api/spotify/refresh").route(web::post().to(spotify_refresh_token)),
            )
            .service(web::resource("/api/youtube/search").route(web::get().to(youtube_search)))
    })
    .bind(bind_addr)
    .unwrap()
    .run()
    .await
}

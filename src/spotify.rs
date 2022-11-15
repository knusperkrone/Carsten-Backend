use crate::error::ErrorResponse;
use crate::logging::APP_LOGGING;

use once_cell::sync::Lazy;
use actix_web::client::Client;
use serde::{Deserialize, Serialize};

static TOKEN: Lazy<String> = Lazy::new(|| {
    std::env::var("SPOTIFY_TOKEN").expect("SPOTIFY_TOKEN not provided")
});

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    auth_code: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
    refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpotifyError {
    error: String,
    error_description: String,
}

pub async fn create_token(req: CreateTokenRequest) -> Result<CreateTokenResponse, ErrorResponse> {
    info!(&APP_LOGGING, "Create token: {}", req.auth_code);

    let token_url = "https://accounts.spotify.com/api/token";
    let redirect_url = "https://integration.if-lab.de/arme-spotitube-backend/api/spotify/callback";
    let client = Client::new();

    let mut res = client
        .post(token_url)
        .header("Authorization", format!("Basic {}", TOKEN.as_str()))
        .send_form(&[
            ("grant_type", "authorization_code"),
            ("code", &req.auth_code),
            ("redirect_uri", redirect_url),
        ])
        .await?;
    Ok(res.json::<CreateTokenResponse>().await?)
}

pub async fn refresh_token(
    req: RefreshTokenRequest,
) -> Result<RefreshTokenResponse, ErrorResponse> {
    info!(&APP_LOGGING, "Refresh token: {}", req.refresh_token);

    let client = Client::new();
    let refresh_token_url = "https://accounts.spotify.com/api/token";

    let mut res = client
        .post(refresh_token_url)
        .header("Authorization", format!("Basic {}", TOKEN.as_str()))
        .send_form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", &req.refresh_token),
        ])
        .await?;
    Ok(res.json::<RefreshTokenResponse>().await?)
}

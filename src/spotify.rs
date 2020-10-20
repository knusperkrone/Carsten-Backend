use crate::error::ErrorResponse;
use crate::logging::APP_LOGGING;

use reqwest::Client;
use serde::{Deserialize, Serialize};

static PRIVATE_TOKEN: &'static str =
    "MmIyMTdhMzI4NTc2NDViNzllNjBkZGEwYTU2YjIyNjg6N2E4NTQ5NDMxZTljNGU0Yzk0ODAyYThmYmE2ZjVlOGQ";

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

async fn handle_response<T: for<'de> Deserialize<'de>>(
    res: reqwest::Result<reqwest::Response>,
) -> Result<T, ErrorResponse> {
    match res {
        Ok(resp) => {
            //info!(APP_LOGGING, "{}", resp);
            if !resp.status().is_success() {
                let resp = resp.json::<SpotifyError>().await?;
                Err(ErrorResponse::new(resp.error, resp.error_description))
            } else {
                let parsed = resp.json::<T>().await;
                match parsed {
                    Ok(token) => Ok(token),
                    Err(e) => Err(ErrorResponse::from_reqwest(e)),
                }
            }
        }
        Err(e) => Err(ErrorResponse::from_reqwest(e)),
    }
}

pub async fn create_token(req: CreateTokenRequest) -> Result<String, ErrorResponse> {
    let token_url = "https://accounts.spotify.com/api/token";
    let redirect_url = "https://integration.if-lab.de/arme-spotitube-backend/api/spotify/callback";

    let client = Client::new();
    let res = client
        .post(token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &req.auth_code),
            ("redirect_uri", redirect_url),
        ])
        .header("Authorization", format!("Basic {}", PRIVATE_TOKEN))
        .header("user-agent", "curl/7.69.1")
        .send()
        .await;

    handle_response(res).await
}

pub async fn refresh_token(
    req: RefreshTokenRequest,
) -> Result<RefreshTokenResponse, ErrorResponse> {
    info!(&APP_LOGGING, "Refresh token: {}", req.refresh_token);
    let body = format!(
        "grant_type=refresh_token&refresh_token={}",
        req.refresh_token
    );

    let token_url = "https://accounts.spotify.com/api/token";
    let client = Client::new();
    let res = client
        .post(token_url)
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Basic {}", PRIVATE_TOKEN))
        .send()
        .await;

    handle_response(res).await
}

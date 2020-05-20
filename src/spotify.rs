use crate::error::ErrorResponse;
use crate::logging::APP_LOGGING;

use reqwest::Client;
use serde::{Deserialize, Serialize};

static TOKEN_URL: &'static str = "https://accounts.spotify.com/api/token";
static REDIRECT_URL: &'static str = "https://spotitube.if-lab.de/api/spotify/callback";
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
            if !resp.status().is_success() {
                let resp = resp.json::<ErrorResponse>().await?;
                Err(resp)
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

pub async fn create_token(req: CreateTokenRequest) -> Result<CreateTokenResponse, ErrorResponse> {
    info!(&APP_LOGGING, "Create token: {}", req.auth_code);
    let body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}",
        req.auth_code, REDIRECT_URL
    );

    let client = Client::new();
    let res = client
        .post(TOKEN_URL)
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Basic {}", PRIVATE_TOKEN))
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

    let client = Client::new();
    let res = client
        .post(TOKEN_URL)
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Basic {}", PRIVATE_TOKEN))
        .send()
        .await;

    handle_response(res).await
}

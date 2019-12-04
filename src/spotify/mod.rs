use crate::error::ErrorResponse;

use reqwest::Client;
use serde::{Deserialize, Serialize};

static TOKEN_URL: &'static str = "https://accounts.spotify.com/api/token";
static REDIRECT_URL: &'static str = "http://spotitube.if-lab.de/api/spotify/callback";
static PRIVATE_TOKEN: &'static str =
    "MmIyMTdhMzI4NTc2NDViNzllNjBkZGEwYTU2YjIyNjg6N2E4NTQ5NDMxZTljNGU0Yzk0ODAyYThmYmE2ZjVlOGQ";

#[derive(FromForm)]
pub struct CreateTokenRequest {
    auth_code: String,
}

#[derive(FromForm)]
pub struct RefreshTokenRequest {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenResponse {
    access_token: String,
    token_type: String,
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

impl SpotifyError {
    fn to_error_response(self) -> ErrorResponse {
        ErrorResponse::new(self.error, self.error_description)
    }
}

impl std::fmt::Display for SpotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ \"error\": \"{}\",\"error_description\": \"{}\" }}",
            self.error, self.error_description
        )
    }
}

fn handle_response<T: for<'de> Deserialize<'de>>(
    res: reqwest::Result<reqwest::Response>,
) -> Result<T, ErrorResponse> {
    match res {
        Ok(mut resp) => {
            if !resp.status().is_success() {
                let e_resp = resp.json::<SpotifyError>()?;
                Err(e_resp.to_error_response())
            } else {
                let parsed = resp.json::<T>();
                match parsed {
                    Ok(token) => Ok(token),
                    Err(e) => Err(ErrorResponse::from_reqwest(e)),
                }
            }
        }
        Err(e) => Err(ErrorResponse::from_reqwest(e)),
    }
}

pub fn create_token(req: CreateTokenRequest) -> Result<CreateTokenResponse, ErrorResponse> {
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
        .send();

    handle_response(res)
}

pub fn refresh_token(req: RefreshTokenRequest) -> Result<RefreshTokenResponse, ErrorResponse> {
    let body = format!("grant_type=refresh_token&refresh_token={}", req.token);

    let client = Client::new();
    let res = client
        .post(TOKEN_URL)
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Basic {}", PRIVATE_TOKEN))
        .send();

    handle_response(res)
}

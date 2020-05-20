use serde::{Serialize, Deserialize};
use std::convert::From;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    status: String,
    reason: String,
}

impl ErrorResponse {
    pub fn new(status: String, reason: String) -> ErrorResponse {
        ErrorResponse {
            status: status,
            reason: reason,
        }
    }

    pub fn from_reqwest(e: reqwest::Error) -> ErrorResponse {
        ErrorResponse::new(String::from("Request Error"), e.to_string())
    }
}

impl From<reqwest::Error> for ErrorResponse {
    fn from(e: reqwest::Error) -> Self {
        ErrorResponse::from_reqwest(e)
    }
} 

impl From<serde_json::Error> for ErrorResponse {
    fn from(e: serde_json::Error) -> Self {
        ErrorResponse::new(String::from("Json error"), e.to_string())
    }
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ \"status\": \"{}\",\"reason\": \"{}\" }}",
            self.status, self.reason
        )
    }
}

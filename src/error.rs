use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Serialize, Deserialize, Debug)]
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
}

impl From<actix_web::error::PayloadError> for ErrorResponse {
    fn from(e: actix_web::error::PayloadError) -> Self {
        println!("{:?}", e);
        ErrorResponse::new(String::from("Payload error"), e.to_string())
    }
}

impl From<actix_web::client::SendRequestError> for ErrorResponse {
    fn from(e: actix_web::client::SendRequestError) -> Self {
        ErrorResponse::new(String::from("Send error"), e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ErrorResponse {
    fn from(e: std::string::FromUtf8Error) -> Self {
        ErrorResponse::new(String::from("String encode error"), e.to_string())
    }
}

impl From<actix_web::client::JsonPayloadError> for ErrorResponse {
    fn from(e: actix_web::client::JsonPayloadError) -> Self {
        ErrorResponse::new(String::from("Json error"), e.to_string())
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

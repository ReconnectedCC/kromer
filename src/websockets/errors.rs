use thiserror::Error;

use crate::errors::krist::KristErrorExt;

#[derive(Debug, Error)]
pub enum WebSocketServerError {
    #[error("WebSocket token was not found in cache")]
    TokenNotFound,
}

impl KristErrorExt for WebSocketServerError {
    fn error_type(&self) -> &'static str {
        match self {
            WebSocketServerError::TokenNotFound => "token_not_found",
        }
    }
}

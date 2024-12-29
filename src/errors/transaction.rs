use actix_web::{error, http::StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Transaction not found")]
    NotFound,

    #[error("Invalid amount")]
    InvalidAmount,

    #[error("Failed to create transaction")]
    FailedCreate,

    #[error("Sender has insufficient funds")]
    InsufficientFunds
}

impl error::ResponseError for TransactionError {
    fn status_code(&self) -> StatusCode {
        match self {
            TransactionError::NotFound => StatusCode::NOT_FOUND,
            TransactionError::InvalidAmount => StatusCode::BAD_REQUEST,
            TransactionError::FailedCreate => StatusCode::INTERNAL_SERVER_ERROR,
            TransactionError::InsufficientFunds => StatusCode::BAD_REQUEST,
        }
    }
}

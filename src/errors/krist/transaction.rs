use actix_web::{error, http::StatusCode, HttpResponse};
use thiserror::Error;

use super::{KristErrorExt, KristErrorResponse};

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Transaction not found")]
    NotFound,

    #[error("Transactions disabled")]
    Disabled,

    #[error("Transaction conflict for parameter {0}")]
    Conflict(String),
}

impl KristErrorExt for TransactionError {
    fn error_type(&self) -> &'static str {
        match self {
            TransactionError::InsufficientFunds => "insufficient_funds",
            TransactionError::NotFound => "transaction_not_found",
            TransactionError::Disabled => "transactions_disabled",
            TransactionError::Conflict(_) => "transaction_conflict",
        }
    }
}

impl error::ResponseError for TransactionError {
    fn status_code(&self) -> StatusCode {
        match self {
            TransactionError::InsufficientFunds => StatusCode::FORBIDDEN,
            TransactionError::NotFound => StatusCode::NOT_FOUND,
            TransactionError::Disabled => StatusCode::LOCKED,
            TransactionError::Conflict(_) => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let error = KristErrorResponse {
            ok: false,
            error: self.error_type(),
            message: self.to_string(),
            info: None,
        };

        HttpResponse::build(self.status_code()).json(error)
    }
}

//! Responses and error types for the krist api routes
pub mod address;
pub mod generic;
pub mod name;
pub mod transaction;

use actix_web::{error, http::StatusCode, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct KristErrorResponse {
    pub ok: bool,
    pub error: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum KristError {
    #[error(transparent)]
    Address(#[from] address::AddressError),

    #[error(transparent)]
    Generic(#[from] generic::GenericError),

    #[error(transparent)]
    Name(#[from] name::NameError),

    #[error(transparent)]
    Transaction(#[from] transaction::TransactionError),

    #[error(transparent)]
    Database(#[from] surrealdb::Error), // Do we really want to expose all of this to the client?
}

pub trait KristErrorExt {
    /// Get the error type for the `message` field in a krist error response
    fn error_type(&self) -> &'static str;
}

impl KristErrorExt for KristError {
    fn error_type(&self) -> &'static str {
        match self {
            KristError::Address(e) => e.error_type(),
            KristError::Generic(e) => e.error_type(),
            KristError::Name(e) => e.error_type(),
            KristError::Transaction(e) => e.error_type(),
            KristError::Database(_) => "internal_server_error",
        }
    }
}

impl error::ResponseError for KristError {
    fn status_code(&self) -> StatusCode {
        // TODO: Evaluate whether or not programs might break when returning the correct error code or not.
        //       In Krist, responses are always error code 200 because of a long standing bug.
        //       For some reason, that bug was never fixed and is just set there for forever, pretty stupid if you ask me.
        match self {
            KristError::Address(e) => e.status_code(),
            KristError::Generic(e) => e.status_code(),
            KristError::Name(e) => e.status_code(),
            KristError::Transaction(e) => e.status_code(),
            KristError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            KristError::Address(e) => e.error_response(),
            KristError::Generic(e) => e.error_response(),
            KristError::Name(e) => e.error_response(),
            KristError::Transaction(e) => e.error_response(),
            KristError::Database(_) => {
                let error = KristErrorResponse {
                    ok: false,
                    error: self.error_type(),
                    message: self.to_string(),
                    info: None,
                };

                HttpResponse::build(self.status_code()).json(error)
            }
        }
    }
}

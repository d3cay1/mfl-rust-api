use crate::mfl_api::MflError;
// src/errors.rs
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use derive_more::Display;
use serde::Serialize;
use thiserror::Error;
// *** USE thiserror::Error ***



// Ensure MflError implements std::error::Error (it should if using this error)

#[derive(Debug, Display, Error)] // Keep derive(Error) from derive_more
pub enum ServiceError {
    #[display("Internal Server Error")]
    InternalServerError, // No inner field, no source

    // String variants: The String is just a message, NOT a source error.
    #[display("Bad Request: {}", _0)]
    BadRequest(String), // NO #[source]

    #[display("Unauthorized: {}", _0)]
    Unauthorized(String), // NO #[source]

    // MflError variants: The inner MflError IS the source. Mark it.
    #[display("MFL API Error: {}", _0)]
    MflApiError(#[source] MflError), // <<< ADD #[source] attribute here

    #[display("MFL Login Error: {}", _0)]
    MflLoginError(#[source] MflError), // <<< ADD #[source] attribute here

    #[display("Not Found: {}", _0)]
    NotFound(String), // NO #[source]
}




#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// --- ResponseError Implementation (Review status codes for MFL errors) ---
impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            // Consider if 500/502 is better for MflApiError if the API itself fails
            ServiceError::MflApiError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // Consider if 500 is better for MflLoginError if the API itself fails
            ServiceError::MflLoginError(_) => StatusCode::UNAUTHORIZED,
            ServiceError::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        // Use the Display implementation derived by derive_more
        let message = self.to_string();
        HttpResponse::build(self.status_code()).json(ErrorResponse { message })
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(_err: std::io::Error) -> Self {
        ServiceError::InternalServerError
    }
}
use anyhow::Error;
use axum::http::StatusCode;
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorBody {
    ok: bool,
    error: String,
}

pub enum ApiError {
    BadRequest(Error),
    NotFound(Error),
    Conflict(Error),
    Internal(Error),
}

impl<E: Into<Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        ApiError::Internal(e.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(e) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    ok: false,
                    error: e.to_string(),
                }),
            )
                .into_response(),
            ApiError::NotFound(e) => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    ok: false,
                    error: e.to_string(),
                }),
            )
                .into_response(),
            ApiError::Conflict(e) => (
                StatusCode::CONFLICT,
                Json(ErrorBody {
                    ok: false,
                    error: e.to_string(),
                }),
            )
                .into_response(),
            ApiError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    ok: false,
                    error: e.to_string(),
                }),
            )
                .into_response(),
        }
    }
}

use std::fmt;

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(e) => write!(f, "BadRequest: {}", e),
            ApiError::NotFound(e) => write!(f, "NotFound: {}", e),
            ApiError::Conflict(e) => write!(f, "Conflict: {}", e),
            ApiError::Internal(e) => write!(f, "Internal: {}", e),
        }
    }
}

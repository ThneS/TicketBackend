use super::error::ErrorCode;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: ErrorCode::Ok.code(),
            message: ErrorCode::Ok.default_message().to_string(),
            data: Some(data),
        }
    }
    pub fn error(code: ErrorCode, msg: Option<String>) -> Self {
        Self {
            code: code.code(),
            message: msg.unwrap_or_else(|| code.default_message().to_string()),
            data: None,
        }
    }
}

// Helpers to produce responses
pub fn ok<T: Serialize>(data: T) -> Response {
    Json(ApiResponse::success(data)).into_response()
}

pub fn bad_request(msg: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::<serde_json::Value>::error(
            ErrorCode::Validation,
            Some(msg.into()),
        )),
    )
        .into_response()
}

pub fn not_found(msg: impl Into<String>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::<serde_json::Value>::error(
            ErrorCode::ShowNotFound,
            Some(msg.into()),
        )),
    )
        .into_response()
}

pub fn internal_error(msg: impl Into<String>) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse::<serde_json::Value>::error(
            ErrorCode::Internal,
            Some(msg.into()),
        )),
    )
        .into_response()
}

use super::response::{bad_request, internal_error, not_found};
use axum::response::Response;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize)]
#[repr(i32)]
pub enum ErrorCode {
    Ok = 0,
    Validation = 1000,
    ParseIdInvalid = 1001,
    JsonInvalid = 1002,
    QueryInvalid = 1003,
    ShowNotFound = 2000,
    Database = 9001,
    Decode = 9002,
    Internal = 9000,
}

impl ErrorCode {
    pub fn code(self) -> i32 {
        self as i32
    }
    pub fn default_message(self) -> &'static str {
        match self {
            ErrorCode::Ok => "ok",
            ErrorCode::Validation => "validation error",
            ErrorCode::ParseIdInvalid => "invalid show id",
            ErrorCode::JsonInvalid => "invalid json body",
            ErrorCode::QueryInvalid => "invalid query params",
            ErrorCode::ShowNotFound => "show not found",
            ErrorCode::Database => "database error",
            ErrorCode::Decode => "decode error",
            ErrorCode::Internal => "internal error",
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("invalid show id: {0}")]
    ParseIdInvalid(String),
    #[error("invalid json body: {0}")]
    JsonInvalid(String),
    #[error("invalid query params: {0}")]
    QueryInvalid(String),
    #[error("show not found: {0}")]
    ShowNotFound(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("decode error: {0}")]
    Decode(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::Validation(_) => ErrorCode::Validation,
            AppError::ParseIdInvalid(_) => ErrorCode::ParseIdInvalid,
            AppError::JsonInvalid(_) => ErrorCode::JsonInvalid,
            AppError::QueryInvalid(_) => ErrorCode::QueryInvalid,
            AppError::ShowNotFound(_) => ErrorCode::ShowNotFound,
            AppError::Database(_) => ErrorCode::Database,
            AppError::Decode(_) => ErrorCode::Decode,
            AppError::Internal(_) => ErrorCode::Internal,
        }
    }
    pub fn to_response(&self) -> Response {
        match self.code() {
            ErrorCode::Validation
            | ErrorCode::ParseIdInvalid
            | ErrorCode::JsonInvalid
            | ErrorCode::QueryInvalid => bad_request(self.to_string()),
            ErrorCode::ShowNotFound => not_found(self.to_string()),
            ErrorCode::Database | ErrorCode::Decode => {
                internal_error(self.to_string())
            }
            ErrorCode::Internal => internal_error(self.to_string()),
            ErrorCode::Ok => bad_request("unexpected ok code"),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        // 简单分类，可按需细化
        AppError::Database(e.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AppError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AppError::Internal(e.to_string())
    }
}

// (AppResult alias removed as current handlers return Response directly)

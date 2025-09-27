use axum::http::StatusCode;
use serde::Deserialize;
use std::{fmt, result::Result as StdResult};

pub const DEFAULT_LIMIT: i64 = 20;
pub const MAX_LIMIT: i64 = 1000;

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

#[derive(Debug, Clone)]
pub struct ValidationError(pub String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ValidationError {}

pub trait Validate: Sized {
    type Err;
    fn validate(self) -> StdResult<Self, Self::Err>;
}

impl Validate for Pagination {
    type Err = ValidationError;

    fn validate(mut self) -> StdResult<Self, Self::Err> {
        if self.limit <= 0 {
            self.limit = DEFAULT_LIMIT;
        }
        if self.limit > MAX_LIMIT {
            self.limit = MAX_LIMIT;
        }
        if self.offset < 0 {
            self.offset = 0;
        }
        Ok(self)
    }
}

// 便捷扩展：任何实现了 Validate<Err=ValidationError> 的参数都可直接 .validated()
pub trait ParamsValidateExt: Sized {
    fn validated(self) -> StdResult<Self, ValidationError>;
}

impl<T> ParamsValidateExt for T
where
    T: Validate<Err = ValidationError>,
{
    fn validated(self) -> StdResult<Self, ValidationError> {
        self.validate()
    }
}

// 可选：标准化错误消息（若未来用于统一 400 响应）
impl From<ValidationError> for (StatusCode, String) {
    fn from(err: ValidationError) -> Self {
        (StatusCode::BAD_REQUEST, err.0)
    }
}

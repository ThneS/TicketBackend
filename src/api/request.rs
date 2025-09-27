use super::schema::{ParamsValidateExt, Validate, ValidationError};
use crate::api::error::AppError;
use crate::api::request_id_header;
use axum::extract::Json;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::{
    extract::{FromRequest, FromRequestParts, Path, Query},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
// 不使用 async_trait 以匹配 axum 0.8 FromRequestParts 的返回类型签名

fn extract_request_id(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(request_id_header())
        .and_then(|v| v.to_str().ok())
}

/// 通用：校验后的查询参数
pub struct ValidatedQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    T: Validate<Err = ValidationError>
        + DeserializeOwned
        + Send
        + Sync
        + 'static,
    S: Send + Sync + 'static,
{
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send
    {
        async move {
            match Query::<T>::from_request_parts(parts, state).await {
                Ok(Query(val)) => match val.validated() {
                    Ok(v) => {
                        let rid = extract_request_id(&parts.headers);
                        tracing::debug!(
                            target = "extractor",
                            extractor = "ValidatedQuery",
                            request_id = rid.unwrap_or(""),
                            "query validated"
                        );
                        Ok(ValidatedQuery(v))
                    }
                    Err(e) => {
                        let rid = extract_request_id(&parts.headers);
                        tracing::debug!(target="extractor", extractor="ValidatedQuery", error=%e.0, request_id = rid.unwrap_or(""), "query validation failed");
                        Err(AppError::QueryInvalid(e.0).to_response())
                    }
                },
                Err(_) => {
                    let rid = extract_request_id(&parts.headers);
                    tracing::debug!(
                        target = "extractor",
                        extractor = "ValidatedQuery",
                        request_id = rid.unwrap_or(""),
                        "query deserialize failed"
                    );
                    Err(AppError::QueryInvalid("invalid query params".into())
                        .to_response())
                }
            }
        }
    }
}

/// 通用：校验后的路径参数（适用于将整个 Path 反序列化为一个结构体并实现 Validate）
pub struct ValidatedPath<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedPath<T>
where
    T: Validate<Err = ValidationError>
        + DeserializeOwned
        + Send
        + Sync
        + 'static,
    S: Send + Sync + 'static,
{
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send
    {
        async move {
            match Path::<T>::from_request_parts(parts, state).await {
                Ok(Path(val)) => match val.validated() {
                    Ok(v) => {
                        let rid = extract_request_id(&parts.headers);
                        tracing::debug!(
                            target = "extractor",
                            extractor = "ValidatedPath",
                            request_id = rid.unwrap_or(""),
                            "path validated"
                        );
                        Ok(ValidatedPath(v))
                    }
                    Err(e) => {
                        let rid = extract_request_id(&parts.headers);
                        tracing::debug!(target="extractor", extractor="ValidatedPath", error=%e.0, request_id = rid.unwrap_or(""), "path validation failed");
                        Err(AppError::Validation(e.0).to_response())
                    }
                },
                Err(_) => {
                    let rid = extract_request_id(&parts.headers);
                    tracing::debug!(
                        target = "extractor",
                        extractor = "ValidatedPath",
                        request_id = rid.unwrap_or(""),
                        "path deserialize failed"
                    );
                    Err(AppError::Validation("invalid path params".into())
                        .to_response())
                }
            }
        }
    }
}

/// 通用：校验后的 JSON Body（包装 axum::Json）
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync + 'static,
    T: Validate<Err = ValidationError> + DeserializeOwned + Send + 'static,
{
    type Rejection = Response;
    fn from_request(
        req: axum::extract::Request,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send
    {
        // 先克隆 headers，再交给内部 Json extractor 消费请求体
        let headers_snapshot = req.headers().clone();
        async move {
            let Json(val) = match Json::<T>::from_request(req, state).await {
                Ok(json) => json,
                Err(_) => {
                    let rid = headers_snapshot
                        .get(request_id_header())
                        .and_then(|v| v.to_str().ok());
                    tracing::debug!(
                        target = "extractor",
                        extractor = "ValidatedJson",
                        request_id = rid.unwrap_or(""),
                        "json deserialize failed"
                    );
                    return Err(AppError::JsonInvalid(
                        "invalid json body".into(),
                    )
                    .to_response());
                }
            };
            match val.validate() {
                Ok(v) => {
                    let rid = extract_request_id(&headers_snapshot);
                    tracing::debug!(
                        target = "extractor",
                        extractor = "ValidatedJson",
                        request_id = rid.unwrap_or(""),
                        "json validated"
                    );
                    Ok(ValidatedJson(v))
                }
                Err(e) => {
                    let rid = extract_request_id(&headers_snapshot);
                    tracing::debug!(target="extractor", extractor="ValidatedJson", error=%e.0, request_id = rid.unwrap_or(""), "json validation failed");
                    Err(AppError::JsonInvalid(e.0).to_response())
                }
            }
        }
    }
}

// ==== 特殊：ShowId Path 处理（十进制 / 0x 十六进制）====

#[macro_export]
macro_rules! path_u256_extractor {
    ($name:ident) => {
        pub struct $name(pub crate::utils::uint256::DbU256);
        impl<S> axum::extract::FromRequestParts<S> for $name
        where
            S: Send + Sync + 'static,
        {
            type Rejection = axum::response::Response;
            fn from_request_parts(
                parts: &mut axum::http::request::Parts,
                state: &S,
            ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
                async move {
                    match axum::extract::Path::<String>::from_request_parts(parts, state).await {
                        Ok(axum::extract::Path(raw)) => match raw.parse::<crate::utils::uint256::DbU256>() {
                            Ok(id) => Ok($name(id)),
                            Err(e) => {
                                tracing::debug!(target="extractor", extractor=stringify!($name), error=%e, "u256 parse failed");
                                Err(crate::api::error::AppError::ParseIdInvalid(e.to_string()).to_response())
                            }
                        },
                        Err(_) => {
                            tracing::debug!(target="extractor", extractor=stringify!($name), "path raw extract failed");
                            Err(crate::api::error::AppError::Validation("invalid path".into()).to_response())
                        }
                    }
                }
            }
        }
    };
}

path_u256_extractor!(PathShowId);
path_u256_extractor!(PathIdU256);
#[macro_export]
macro_rules! json_validated {
    ($json_pat:ident) => {{
        match $json_pat.validate() {
            Ok(v) => v,
            Err(e) => {
                return crate::api::error::AppError::JsonInvalid(e.0)
                    .to_response();
            }
        }
    }};
}
// 末尾原重复定义清理

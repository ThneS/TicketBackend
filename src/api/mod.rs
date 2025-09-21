use crate::db::Db;
pub mod show_manager;
use crate::config;
use bytes::Bytes;
use eyre::Result;
use tower_http::request_id::{
    MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer,
};
use tower_http::trace::TraceLayer;
#[derive(Debug, Clone)]
pub struct ApiContext {
    pub db: Db,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub api: ApiContext,
}

pub async fn listen_app() -> Result<()> {
    let state = AppState {
        api: ApiContext {
            db: Db::connect(config::get().database_url.as_str(), 5).await?,
        },
    };
    let log_headers = std::env::var("LOG_HTTP_HEADERS")
        .ok()
        .map_or(false, |v| v == "1");
    let log_body = std::env::var("LOG_HTTP_BODY")
        .ok()
        .map_or(false, |v| v == "1");
    let req_id_header: axum::http::HeaderName = std::env::var("REQ_ID_HEADER")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| axum::http::HeaderName::from_static("x-request-id"));

    let req_id_for_span = req_id_header.clone();
    let trace = TraceLayer::new_for_http()
        .make_span_with(move |req: &axum::http::Request<_>| {
            let ua = req
                .headers()
                .get(axum::http::header::USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default();
            let rid = req
                .headers()
                .get(&req_id_for_span)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            tracing::info_span!(
                "http.request",
                method = %req.method(),
                uri = %req.uri(),
                version = ?req.version(),
                user_agent = %ua,
                request_id = %rid,
            )
        })
        .on_request(
            move |req: &axum::http::Request<_>, _span: &tracing::Span| {
                if log_headers {
                    tracing::debug!(headers = ?req.headers(), "http headers");
                }
            },
        )
        .on_response({
            let req_id_on_resp = req_id_header.clone();
            move |res: &axum::http::Response<_>, _latency: std::time::Duration, _span: &tracing::Span| {
                let status = res.status().as_u16();
                let size = res
                    .headers()
                    .get(axum::http::header::CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                let rid = res
                    .headers()
                    .get(&req_id_on_resp)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                match size {
                    Some(len) => tracing::info!(status, size = len, request_id = %rid, "http response"),
                    None => tracing::info!(status, request_id = %rid, "http response"),
                }
            }
        })
        .on_eos(
            |_trailers: Option<&axum::http::HeaderMap>,
             latency: std::time::Duration,
             _span: &tracing::Span| {
                tracing::info!(?latency, "http completed");
            },
        )
        .on_body_chunk(
            move |chunk: &Bytes,
                  latency: std::time::Duration,
                  _span: &tracing::Span| {
                if log_body {
                    let size = chunk.len();
                    tracing::debug!(size, ?latency, "http body chunk");
                }
            },
        );

    let app = axum::Router::new()
        .route(
            "/shows/{id}",
            axum::routing::get(show_manager::show_with_id),
        )
        // Compose layers in correct order (outermost last): Propagate (inner) -> Trace -> Set (outer)
        .layer(PropagateRequestIdLayer::new(req_id_header.clone()))
        .layer(trace)
        .layer(SetRequestIdLayer::new(
            req_id_header.clone(),
            MakeRequestUuid::default(),
        ))
        .with_state(state);
    tracing::info!("Listening on 127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let _ = axum::serve(listener, app).await;
    Ok(())
}

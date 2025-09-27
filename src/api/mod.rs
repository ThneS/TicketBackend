use crate::db::Db;
pub mod error;
pub mod request;
pub mod response;
pub mod schema;
pub mod show_manager;
use crate::config;
use axum::http::HeaderName;
use axum::http::{HeaderValue, Method};
use bytes::Bytes;
use eyre::Result;
#[cfg(feature = "otel")]
use opentelemetry::trace::TracerProvider;
#[cfg(feature = "otel")]
use opentelemetry::{KeyValue, global};
#[cfg(feature = "otel")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "otel")]
use opentelemetry_sdk::{Resource, trace as sdktrace};
use std::sync::OnceLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{
    MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer,
};
use tower_http::trace::TraceLayer;
#[cfg(feature = "otel")]
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};
#[derive(Debug, Clone)]
pub struct ApiContext {
    pub db: Db,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub api: ApiContext,
}

// 全局缓存 request-id header 名称，避免多处重复读取 env。
static REQ_ID_HEADER: OnceLock<HeaderName> = OnceLock::new();

pub fn request_id_header() -> &'static HeaderName {
    REQ_ID_HEADER.get_or_init(|| {
        std::env::var("REQ_ID_HEADER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| HeaderName::from_static("x-request-id"))
    })
}

pub async fn listen_app() -> Result<()> {
    init_tracing();
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
    let req_id_header = request_id_header().clone();
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
            // 先创建 span，request_id 字段留空，再动态 record，满足“直接 .record” 的需求
            let span = tracing::info_span!(
                "http.request",
                method = %req.method(),
                uri = %req.uri(),
                version = ?req.version(),
                user_agent = %ua,
                request_id = tracing::field::Empty,
            );
            span.record("request_id", &rid);
            span
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

    // CORS: 允许前端读取自定义 request-id 头，需要显式 expose
    let cors = {
        // 可按需从环境读取允许来源，这里默认允许所有（开发阶段）
        let allow_origin = std::env::var("CORS_ALLOW_ORIGIN").ok();
        let base = if let Some(o) = allow_origin {
            if o == "*" {
                CorsLayer::new().allow_origin(Any)
            } else {
                // 支持多个以逗号分隔
                let origins: Vec<HeaderValue> = o
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                let mut layer = CorsLayer::new();
                for hv in origins {
                    layer = layer.allow_origin(hv);
                }
                layer
            }
        } else {
            CorsLayer::new().allow_origin(Any)
        };
        base.allow_headers(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
            ])
            .expose_headers([request_id_header().clone()])
    };

    let app = axum::Router::new()
        .route(
            "/show/{id}",
            axum::routing::get(show_manager::show_with_id)
                .put(show_manager::update_show)
                .delete(show_manager::delete_show),
        )
        .route(
            "/show",
            axum::routing::post(show_manager::create_show),
        )
        .route("/shows", axum::routing::get(show_manager::list_shows))
        // Compose layers in correct order (outermost last): Propagate (inner) -> Trace -> Set (outer)
        .layer(PropagateRequestIdLayer::new(request_id_header().clone()))
        .layer(trace)
        .layer(cors)
        .layer(SetRequestIdLayer::new(
            request_id_header().clone(),
            MakeRequestUuid::default(),
        ))
        .with_state(state);
    tracing::info!("Listening on 127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let _ = axum::serve(listener, app).await;
    Ok(())
}

fn init_tracing() {
    // 无论是否启用 otel，都设置基础 EnvFilter（若用户外部已设置则忽略错误）
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        #[cfg(feature = "otel")]
        {
            let service_name = std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "ticket-backend".into());
            let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:4317".into());
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint);
            let tracer_provider = sdktrace::TracerProvider::builder()
                .with_simple_exporter(exporter.build_span_exporter().unwrap())
                .with_config(sdktrace::Config::default().with_resource(
                    Resource::new(vec![KeyValue::new(
                        "service.name",
                        service_name,
                    )]),
                ))
                .build();
            let tracer = tracer_provider.tracer("api");
            let otel_layer = OpenTelemetryLayer::new(tracer);
            let subscriber = Registry::default().with(filter).with(otel_layer);
            let _ = tracing::subscriber::set_global_default(subscriber);
            global::set_tracer_provider(tracer_provider);
        }
        #[cfg(not(feature = "otel"))]
        {
            let subscriber = Registry::default().with(filter);
            let _ = tracing::subscriber::set_global_default(subscriber);
        }
    });
}

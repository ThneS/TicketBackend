use axum::response::IntoResponse;
use axum::{
    Router,
    http::{Request, StatusCode},
    routing::get,
};
use backend::api::request_id_header;
use tower::ServiceExt;
use tower_http::request_id::{
    MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer,
};

async fn handler() -> axum::response::Response {
    // 回显：如果需要在 Body 中也返回，可再加 request_id
    axum::Json(serde_json::json!({"ok": true})).into_response()
}

#[tokio::test]
async fn test_custom_request_id_header_roundtrip() {
    // 准备一个自定义 header 名称（使用默认 x-request-id 即可）
    let hdr = request_id_header().clone();
    let app = Router::new()
        .route("/ping", get(handler))
        .layer(PropagateRequestIdLayer::new(hdr.clone()))
        .layer(SetRequestIdLayer::new(
            hdr.clone(),
            MakeRequestUuid::default(),
        ));
    let req = Request::builder()
        .uri("/ping")
        .header(request_id_header(), "test-rid-123")
        .body(axum::body::Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    // 响应中应该保留相同 request-id（SetRequestIdLayer 不会覆盖已存在的 header）
    let got = res
        .headers()
        .get(request_id_header())
        .and_then(|v| v.to_str().ok());
    assert_eq!(got, Some("test-rid-123"));
}

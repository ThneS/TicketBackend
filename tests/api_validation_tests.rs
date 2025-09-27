use axum::http::{Request, StatusCode};
use axum::{
    Router,
    response::Response,
    routing::{get, post},
};
use backend::api::request::{ValidatedJson, ValidatedQuery};
use backend::api::response::ok;
use serde::Deserialize;
use tower::ServiceExt; // trait for .oneshot

#[derive(Deserialize)]
struct DummyQuery {
    limit: i64,
    offset: i64,
}
impl backend::api::schema::Validate for DummyQuery {
    type Err = backend::api::schema::ValidationError;
    fn validate(self) -> Result<Self, Self::Err> {
        Ok(self)
    }
}

#[derive(Deserialize)]
struct DummyBody {
    name: String,
}
impl backend::api::schema::Validate for DummyBody {
    type Err = backend::api::schema::ValidationError;
    fn validate(self) -> Result<Self, Self::Err> {
        if self.name.is_empty() {
            return Err(backend::api::schema::ValidationError(
                "name empty".into(),
            ));
        }
        Ok(self)
    }
}

#[derive(Clone)]
struct AppState;

async fn query_handler(
    ValidatedQuery(_q): ValidatedQuery<DummyQuery>,
) -> Response {
    ok("q ok")
}
async fn json_handler(ValidatedJson(_b): ValidatedJson<DummyBody>) -> Response {
    ok("j ok")
}

#[tokio::test]
async fn test_query_invalid() {
    let app = Router::new().route("/q", get(query_handler));
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/q?limit=abc")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_json_invalid() {
    let app = Router::new().route("/j", post(json_handler));
    let req = Request::builder()
        .method("POST")
        .uri("/j")
        .header("content-type", "application/json")
        .body(axum::body::Body::from("{\"name\":\"\"}"))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

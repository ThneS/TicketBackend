use crate::db::Db;
pub mod show_manager;
use crate::config;
use eyre::Result;
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
    let app = axum::Router::new()
        .route("/show/{id}", axum::routing::get(show_manager::show_with_id))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let _ = axum::serve(listener, app).await;
    Ok(())
}

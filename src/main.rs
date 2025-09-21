use backend::{
    api::listen_app,
    contract::{listen_chain, providers},
    db::Db,
};
use eyre::{Ok, Result};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Init env and logging first
    dotenv::dotenv().ok();
    backend::logging::init();
    let config = backend::config::init_from_env()?;
    let db = Db::connect(&config.database_url, 5).await?;
    let pool: &'static providers::ProviderPool = providers::init_pool().await?;

    tokio::select! {
        res = listen_app() => { if let Err(e) = res { tracing::error!(error = ?e, "Error in listen_app"); } }
        res = listen_chain(&config, db.clone(), pool) => { if let Err(e) = res { tracing::error!(error = ?e, "Error in listen_chain"); } }
        _ = signal::ctrl_c() => { tracing::info!("Received Ctrl+C, shutting down."); }
    }
    Ok(())
}

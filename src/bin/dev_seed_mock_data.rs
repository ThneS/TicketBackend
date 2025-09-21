use backend::{config, db::Db};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let cfg = config::init_from_env()?;
    let db = Db::connect(&cfg.database_url, 5).await?;
    let _ = backend::db::run_migrations(db.pool()).await;
    backend::tools::seed_mock(&db).await
}

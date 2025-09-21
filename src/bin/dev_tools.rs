use alloy::primitives::U256 as AlloyU256;
use backend::{config, contract::providers, db::Db};
use clap::{Parser, Subcommand};
use eyre::Result;

#[derive(Parser)]
#[command(
    name = "dev-tools",
    about = "Dev utilities: seed DB, update show name"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Seed mock data into Postgres
    Seed {},
    /// Call ShowManager.updateShow on chain
    UpdateShow {
        /// Show ID (u64)
        show_id: u64,
        /// New show name
        name: String,
        /// Metadata URI
        metadata_uri: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    backend::logging::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Seed {} => seed().await,
        Commands::UpdateShow {
            show_id,
            name,
            metadata_uri,
        } => update_show(show_id, name, metadata_uri).await,
    }
}

async fn seed() -> Result<()> {
    let cfg = config::init_from_env()?;
    let db = Db::connect(&cfg.database_url, 5).await?;
    let _ = backend::db::run_migrations(db.pool()).await;

    // reuse the seeder logic from shared tools module
    backend::tools::seed_mock(&db).await
}

async fn update_show(
    show_id: u64,
    name: String,
    metadata_uri: String,
) -> Result<()> {
    // init config + signer pool
    let _ = config::init_from_env()?;
    providers::init_signer_pool_from_env()?;
    let provider = providers::signer_pool().default_provider().await?;

    let id = AlloyU256::from(show_id);
    backend::contract::providers::demo_update_show_name(
        provider,
        id,
        name,
        metadata_uri,
    )
    .await?;
    tracing::info!(show_id, "Submitted updateShow transaction");
    Ok(())
}

// seed_impl removed; logic moved to backend::tools::seed_mock

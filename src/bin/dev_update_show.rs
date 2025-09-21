use alloy::primitives::U256;
use eyre::{Result, bail};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env and global config
    let _ = dotenv::dotenv();
    backend::config::init_from_env()?;
    backend::contract::providers::init_signer_pool_from_env()?;

    // Args: <show_id:u64> <name:string> <metadata_uri:string>
    let mut args = std::env::args().skip(1);
    let show_id_s = args.next().unwrap_or_default();
    let name = args.next().unwrap_or_default();
    let uri = args.next().unwrap_or_default();
    if show_id_s.is_empty() || name.is_empty() || uri.is_empty() {
        eprintln!(
            "Usage: cargo run --bin dev-update-show -- <show_id> <name> <metadata_uri>"
        );
        bail!("missing arguments");
    }
    let show_id_u: u64 = show_id_s.parse()?;
    let show_id = U256::from(show_id_u);

    let provider = backend::contract::providers::signer_pool()
        .default_provider()
        .await?;

    backend::contract::providers::demo_update_show_name(
        provider, show_id, name, uri,
    )
    .await?;
    println!("Submitted updateShow transaction for show_id={}", show_id_u);
    Ok(())
}

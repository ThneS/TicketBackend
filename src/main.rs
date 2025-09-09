pub mod bindings;
mod db;
mod event;
mod repo;
use std::{env, str::FromStr};

use crate::{db::Db, event::router::route_log};
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter},
};

use dotenv::dotenv;
use eyre::{Ok, Result};
use futures_util::stream::StreamExt;

#[derive(Clone, Debug)]
pub struct FeatureFlags {
    pub print_raw_logs: bool,
    pub print_unknown: bool,
}

#[derive(Clone, Debug)]
pub struct AddressMap {
    pub did_registry: Address,
    pub show_manager: Address,
}
use tokio::signal;

async fn listen_app() -> Result<()> {
    let app = axum::Router::new();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let _ = axum::serve(listener, app).await;
    Ok(())
}

async fn listen_chain(
    addr_map: AddressMap,
    flags: FeatureFlags,
    db: Db,
) -> Result<()> {
    let ws = WsConnect::new("ws://127.0.0.1:8545");
    let provider = ProviderBuilder::new().connect_ws(ws).await?;
    let filter = Filter::new().from_block(BlockNumberOrTag::Latest);
    // Subscribe to logs.
    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        route_log(log, &addr_map, &flags, &db).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let did = Address::from_str(&env::var("DID_REGISTRY_ADDRESS")?)?;
    let show = Address::from_str(&env::var("SHOW_MANAGER_ADDRESS")?)?;
    let addr_map = AddressMap {
        did_registry: did,
        show_manager: show,
    };
    let flags = FeatureFlags {
        print_raw_logs: env::var("PRINT_RAW_LOGS").ok().as_deref() == Some("1"),
        print_unknown: env::var("PRINT_UNKNOWN_LOGS").ok().as_deref()
            == Some("1"),
    };

    let database_url = env::var("DATABASE_URL")?;
    let db = Db::connect(&database_url, 5).await?;

    tokio::select! {
        res = listen_app() => { if let Err(e) = res { eprintln!("Error in listen_app: {:?}", e); } }
        res = listen_chain(addr_map, flags, db.clone()) => { if let Err(e) = res { eprintln!("Error in listen_chain: {:?}", e); } }
        _ = signal::ctrl_c() => { println!("Received Ctrl+C, shutting down."); }
    }
    Ok(())
}

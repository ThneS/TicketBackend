pub mod bindings;
pub mod contracts;
pub mod event;
pub mod providers;
use alloy::{
    providers::Provider,
    rpc::types::{BlockNumberOrTag, Filter},
};
use eyre::Result;
use futures_util::stream::StreamExt;

use crate::{config::Config, db::Db};

/// 监听链上日志并路由到对应模块。
pub async fn listen_chain(
    config: &Config,
    db: Db,
    pool: &providers::ProviderPool,
) -> Result<()> {
    let provider = pool.ws_listener();
    let filter = Filter::new().from_block(BlockNumberOrTag::Latest);
    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        event::router::route_log(
            log,
            provider.clone(),
            &config.addresses,
            &config.flags,
            &db,
        )
        .await;
    }
    Ok(())
}

// Re-export commonly used types for convenience
pub use crate::config::{AddressMap, FeatureFlags};

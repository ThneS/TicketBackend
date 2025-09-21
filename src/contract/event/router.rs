use crate::{
    contract::contracts::show_manager::parse_event as parse_show_created,
    db::Db,
};
use alloy::{providers::Provider, rpc::types::Log};

pub async fn route_log<P: Provider + Clone + Send + Sync + 'static>(
    log: Log,
    provider: P,
    addr_map: &crate::contract::AddressMap,
    flags: &crate::contract::FeatureFlags,
    db: &Db,
) {
    // raw log debug is handled below per-address when enabled
    match log.address() {
        addr if *addr == *addr_map.show_manager => {
            if flags.print_raw_logs {
                tracing::debug!(?log, "RAW LOG");
            }
            if let Err(e) = parse_show_created(&log, provider.clone(), db).await
            {
                if flags.print_unknown {
                    tracing::warn!(error = ?e, "Unknown ShowManager event");
                }
            }
        }
        _ => {
            if flags.print_unknown {
                tracing::debug!(addr = %format!("0x{}", hex::encode(log.address().as_slice())), "Log from unknown address");
            }
        }
    }
}

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
    if flags.print_raw_logs {
        println!("RAW LOG: {:?}", log);
    }
    match log.address() {
        addr if *addr == *addr_map.show_manager => {
            if let Err(e) = parse_show_created(&log, provider.clone(), db).await
            {
                if flags.print_unknown {
                    println!("Unknown ShowManager event: {:?}", e);
                }
            }
        }
        _ => {
            if flags.print_unknown {
                println!("Log from unknown address: {:?}", log.address());
            }
        }
    }
}

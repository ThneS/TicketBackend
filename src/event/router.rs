use crate::{db::Db, event::show_manager::parse_event as parse_show_created};
use alloy::rpc::types::Log;

pub async fn route_log(
    log: Log,
    addr_map: &crate::AddressMap,
    flags: &crate::FeatureFlags,
    db: &Db,
) {
    if flags.print_raw_logs {
        println!("RAW LOG: {:?}", log);
    }
    match log.address() {
        addr if *addr == *addr_map.show_manager => {
            if let Err(e) = parse_show_created(&log, db).await {
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

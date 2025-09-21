use crate::{
    contract::bindings::ShowManager::{
        Show as OnchainShow, ShowCreated, ShowManagerInstance,
    },
    db::Db,
    repo::show_repo::{
        ShowCreatedDetailRecord, ShowCreatedRecord, ShowDataRecord,
        ShowStatus::{Active, Cancelled, Ended, Upcoming},
        upsert_show_all,
    },
    utils::uint256::DbU256,
};
use alloy::{providers::Provider, rpc::types::Log, sol_types::SolEvent};
use eyre::{Result, bail};

async fn insert_show_data_value(
    tx_hash: Option<String>,
    block_number: Option<DbU256>,
    address: String,
    log_index: Option<DbU256>,
    db: &Db,
    show_data: OnchainShow,
) -> Result<()> {
    let show_id = show_data.id;
    // First insert basic event row
    let basic = ShowCreatedRecord {
        show_id: DbU256(show_id),
        tx_hash,
        block_number,
        organizer: address.clone(),
        log_index,
        created_at: chrono::Utc::now(),
    };
    let detail = ShowCreatedDetailRecord {
        show_id: DbU256(show_id),
        start_time: DbU256(show_data.startTime),
        end_time: DbU256(show_data.endTime),
        total_tickets: DbU256(show_data.totalTickets),
        ticket_price: DbU256(show_data.ticketPrice),
        decimal: 18_i64, // Ethereum standard
        ticket_sold: DbU256(show_data.ticketsSold),
        organizer: address.clone(),
        location: show_data.location.clone(),
        name: show_data.name.clone(),
        description: show_data.description.clone(),
        metadata_uri: if show_data.metadataURI.is_empty() {
            None
        } else {
            Some(show_data.metadataURI)
        },
        status: match show_data.status {
            0 => Upcoming,
            1 => Active,
            2 => Ended,
            3 => Cancelled,
            _ => crate::repo::show_repo::ShowStatus::Upcoming, // default/fallback
        },
        created_at: chrono::Utc::now(),
    };
    let data = ShowDataRecord {
        id: DbU256(show_data.id),
        name: show_data.name.clone(),
        description: show_data.description.clone(),
        location: show_data.location.clone(),
        event_time: DbU256(show_data.startTime),
        ticket_price: DbU256(show_data.ticketPrice),
        max_tickets: DbU256(show_data.totalTickets),
        sold_tickets: DbU256(show_data.ticketsSold),
        is_active: matches!(show_data.status, 1), // Active status
        organizer: address,
        created_at: chrono::Utc::now(),
    };
    // 将事务聚合到 repo 层统一管理
    upsert_show_all(db.pool(), &basic, &detail, &data).await?;

    Ok(())
}

async fn get_show_data<P: Provider + Clone + Send + Sync + 'static>(
    provider: P,
    show_id: alloy::primitives::U256,
) -> Result<OnchainShow> {
    let addr = crate::config::get().addresses.show_manager;
    let inst = ShowManagerInstance::new(addr, provider);
    let res = inst.getShow(show_id).call().await?;
    let show: OnchainShow = res;
    Ok(show)
}

pub async fn parse_event<P: Provider + Clone + Send + Sync + 'static>(
    log: &Log,
    provider: P,
    db: &Db,
) -> Result<()> {
    let inner = &log.inner; // primitives::Log
    if let Some(topic0) = inner.topics().first() {
        if *topic0 == ShowCreated::SIGNATURE_HASH {
            let event = ShowCreated::decode_log(inner)?;
            tracing::info!(?event, "Parsed ShowCreated event");

            let tx_hash = log
                .transaction_hash
                .map(|h| format!("0x{}", hex::encode(h.as_slice())));
            let block_number: Option<DbU256> = log
                .block_number
                .map(|b| DbU256(alloy::primitives::U256::from(b)));
            let address =
                format!("0x{}", hex::encode(inner.address.as_slice()));
            let log_index: Option<DbU256> = log
                .log_index
                .map(|i| DbU256(alloy::primitives::U256::from(i)));

            // Fetch on-chain show detail (simple version), ignore errors to avoid blocking ingestion
            if let Ok(show) = get_show_data(provider, event.showId).await {
                tracing::debug!(?show, "On-chain Show detail fetched");
                insert_show_data_value(
                    tx_hash,
                    block_number,
                    address,
                    log_index,
                    db,
                    show,
                )
                .await?;
            } else {
                bail!(
                    "failed to fetch on-chain show data for showId {:?}",
                    event.showId
                );
            }

            return Ok(());
        }
    }
    eyre::bail!("not show created")
}

use crate::{
    bindings::ShowManager::ShowCreated,
    db::Db,
    repo::show_repo::{ShowCreatedRecord, insert_show_created},
};
use alloy::{rpc::types::Log, sol_types::SolEvent};
use eyre::Result;

pub async fn parse_event(log: &Log, db: &Db) -> Result<()> {
    let inner = &log.inner; // primitives::Log
    if let Some(topic0) = inner.topics().first() {
        if *topic0 == ShowCreated::SIGNATURE_HASH {
            let event = ShowCreated::decode_log(inner)?;
            println!("Parsed ShowCreated event: {:?}", event);
            // Event signature: ShowCreated(showId, organizer, name, startTime, endTime, venue)
            let tx_hash = log
                .transaction_hash
                .map(|h| format!("0x{}", hex::encode(h.as_slice())));
            let block_number: Option<i64> = log.block_number.map(|b| b as i64);
            let address = format!("0x{}", hex::encode(inner.address.as_slice()));

            // Insert with ON CONFLICT DO NOTHING if we have a natural key.
            // For now use (tx_hash, log_index) uniqueness if available.
            let log_index: Option<i64> = log.log_index.map(|i| i as i64);

            // Serialize structured json
            let full_json = serde_json::json!({
                "show_id": event.showId,
                "organizer": format!("0x{}", hex::encode(event.organizer.0)),
                "name": event.name,
                "start_time": event.startTime,
                "end_time": event.endTime,
                "venue": event.venue,
            });

            sqlx::query(
                r#"
                INSERT INTO show_created_events
                    (tx_hash, block_number, contract_address, log_index, raw_event)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (tx_hash, log_index) DO NOTHING
            "#,
            )
            .bind(tx_hash.clone())
            .bind(block_number)
            .bind(address)
            .bind(log_index)
            .bind(full_json)
            .execute(db.pool())
            .await?;
            // Upsert into detail table
            let organizer_hex = format!("0x{}", hex::encode(event.organizer.0));
            // Convert U256 fields (assumed < i64::MAX) using low 64 bits
            let show_id_u64: u64 = event.showId.try_into().unwrap_or_default();
            let start_u64: u64 = event.startTime.try_into().unwrap_or_default();
            let end_u64: u64 = event.endTime.try_into().unwrap_or_default();
            let rec = ShowCreatedRecord {
                show_id: show_id_u64 as i64,
                organizer: &organizer_hex,
                name: &event.name,
                start_time: start_u64 as i64,
                end_time: end_u64 as i64,
                venue: &event.venue,
                tx_hash: tx_hash.as_deref(),
                block_number,
                log_index,
            };
            insert_show_created(db.pool(), &rec).await?;
            return Ok(());
        }
    }
    eyre::bail!("not show created")
}

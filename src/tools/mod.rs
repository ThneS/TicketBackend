use crate::{
    db::Db,
    repo::show_repo::{
        ShowCreatedDetailRecord, ShowCreatedRecord, ShowDataRecord, ShowStatus,
    },
    utils::uint256::{DbU256, U256},
};
use chrono::Utc;
use eyre::Result;
use std::str::FromStr;

pub async fn seed_mock(db: &Db) -> Result<()> {
    let shows = vec![
        (
            "1",
            "Demo Show One",
            "First demo show",
            "City Hall",
            "ipfs://demo1",
            1735689600u64,
            1735696800u64,
            1000u64,
            1_000_000_000_000_000_000u128,
            10u64,
            true,
            "alice",
        ),
        (
            "2",
            "Demo Show Two",
            "Second demo show",
            "Opera House",
            "ipfs://demo2",
            1735776000u64,
            1735783200u64,
            2000u64,
            5_000_000_000_000_000u128,
            0u64,
            true,
            "bob",
        ),
        (
            "3",
            "Demo Show Three",
            "Third demo show",
            "Open Air Stage",
            "ipfs://demo3",
            1735862400u64,
            1735869600u64,
            500u64,
            2_000_000_000_000_000_000u128,
            250u64,
            false,
            "carol",
        ),
    ];
    for (
        id,
        name,
        desc,
        loc,
        uri,
        start,
        end,
        total,
        price,
        sold,
        active,
        org,
    ) in shows
    {
        let id = DbU256::from_str(id).map_err(|e| eyre::eyre!(e))?;
        let start = DbU256(U256::from(start));
        let end = DbU256(U256::from(end));
        let total = DbU256(U256::from(total));
        let price = DbU256(U256::from(price));
        let sold = DbU256(U256::from(sold));

        let basic = ShowCreatedRecord {
            show_id: id.clone(),
            tx_hash: None,
            block_number: None,
            organizer: org.to_string(),
            log_index: None,
            created_at: Utc::now(),
        };
        let detail = ShowCreatedDetailRecord {
            show_id: id.clone(),
            start_time: start.clone(),
            end_time: end.clone(),
            total_tickets: total.clone(),
            ticket_price: price.clone(),
            decimal: 18,
            ticket_sold: sold.clone(),
            organizer: org.to_string(),
            location: loc.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            metadata_uri: Some(uri.to_string()),
            status: if active {
                ShowStatus::Active
            } else {
                ShowStatus::Upcoming
            },
            created_at: Utc::now(),
        };
        let data = ShowDataRecord {
            id: id.clone(),
            name: name.to_string(),
            description: desc.to_string(),
            location: loc.to_string(),
            event_time: start.clone(),
            ticket_price: price.clone(),
            max_tickets: total.clone(),
            sold_tickets: sold.clone(),
            is_active: active,
            organizer: org.to_string(),
            created_at: Utc::now(),
        };
        crate::repo::show_repo::upsert_show_all(
            db.pool(),
            &basic,
            &detail,
            &data,
        )
        .await?;
        println!("Seeded show {} - {}", id.to_string(), name);
    }
    println!("Mock data seeding complete.");
    Ok(())
}

use backend::{
    db::Db,
    repo::show_repo::{
        ShowCreatedDetailRecord, ShowCreatedRecord, ShowDataRecord, ShowStatus,
        get_show_by_id, upsert_show_all,
    },
    utils::uint256::{DbU256, U256},
};
use chrono::Utc;

// NOTE: These tests require a running Postgres with DATABASE_URL and migrated schema.
// They are ignored by default to avoid CI/local failures when DB is unavailable.

#[tokio::test]
#[ignore]
async fn upsert_and_get_show_roundtrip() {
    dotenv::dotenv().ok();
    let cfg = backend::config::init_from_env().expect("config");
    let db = Db::connect(&cfg.database_url, 5).await.expect("db");
    let _ = backend::db::run_migrations(db.pool()).await; // best effort

    let id = DbU256(U256::from(42u64));

    let basic = ShowCreatedRecord {
        show_id: id.clone(),
        tx_hash: Some(format!("0x{:064x}", 0xfeed_u64)),
        block_number: Some(DbU256(U256::from(100u64))),
        organizer: "tester".to_string(),
        log_index: Some(DbU256(U256::from(1u64))),
        created_at: Utc::now(),
    };
    let detail = ShowCreatedDetailRecord {
        show_id: id.clone(),
        start_time: DbU256(U256::from(1_735_689_600u64)),
        end_time: DbU256(U256::from(1_735_696_800u64)),
        total_tickets: DbU256(U256::from(1000u64)),
        ticket_price: DbU256(U256::from(1_000_000_000_000_000_000u128)),
        decimal: 18,
        ticket_sold: DbU256(U256::from(0u64)),
        organizer: "tester".to_string(),
        location: "Somewhere".to_string(),
        name: "Test Show".to_string(),
        description: "A test show".to_string(),
        metadata_uri: Some("ipfs://meta".to_string()),
        status: ShowStatus::Active,
        created_at: Utc::now(),
    };
    let data = ShowDataRecord {
        id: id.clone(),
        name: "Test Show".to_string(),
        description: "A test show".to_string(),
        location: "Somewhere".to_string(),
        event_time: DbU256(U256::from(1_735_689_600u64)),
        ticket_price: DbU256(U256::from(1_000_000_000_000_000_000u128)),
        max_tickets: DbU256(U256::from(1000u64)),
        sold_tickets: DbU256(U256::from(0u64)),
        is_active: true,
        organizer: "tester".to_string(),
        created_at: Utc::now(),
    };

    upsert_show_all(db.pool(), &basic, &detail, &data)
        .await
        .expect("upsert");

    let found = get_show_by_id(db.pool(), id.clone()).await.expect("get");
    let found = found.expect("some");
    assert_eq!(found.id.to_string(), id.to_string());
    assert_eq!(found.name, data.name);
}

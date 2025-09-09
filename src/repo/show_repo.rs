use eyre::Result;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub struct ShowCreatedRecord<'a> {
    pub show_id: i64,
    pub organizer: &'a str,
    pub name: &'a str,
    pub start_time: i64,
    pub end_time: i64,
    pub venue: &'a str,
    pub tx_hash: Option<&'a str>,
    pub block_number: Option<i64>,
    pub log_index: Option<i64>,
}

pub async fn insert_show_created(pool: &PgPool, rec: &ShowCreatedRecord<'_>) -> Result<()> {
    sqlx::query(r#"
        INSERT INTO show_created_events_detail
            (show_id, organizer, name, start_time, end_time, venue, tx_hash, block_number, log_index)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        ON CONFLICT (show_id) DO UPDATE SET
            organizer = EXCLUDED.organizer,
            name = EXCLUDED.name,
            start_time = EXCLUDED.start_time,
            end_time = EXCLUDED.end_time,
            venue = EXCLUDED.venue
    "#)
        .bind(rec.show_id)
        .bind(rec.organizer)
        .bind(rec.name)
        .bind(rec.start_time)
        .bind(rec.end_time)
        .bind(rec.venue)
        .bind(rec.tx_hash)
        .bind(rec.block_number)
        .bind(rec.log_index)
        .execute(pool)
        .await?;
    Ok(())
}

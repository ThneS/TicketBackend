use eyre::Result;
use serde::Serialize;
use sqlx::{PgPool, prelude::FromRow};

// 结构化 ShowCreated 详情记录（拥有所有权字段，便于跨异步边界传递与查询返回）。
#[derive(Debug, Serialize, FromRow)]
pub struct ShowCreatedRecord {
    pub show_id: i64,
    pub organizer: String,
    pub name: String,
    pub start_time: i64,
    pub end_time: i64,
    pub venue: String,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub log_index: Option<i64>,
}

/// Upsert 一条 ShowCreated 详情记录；以 show_id 为主键，重复则更新基础字段。
pub async fn insert_show_created(
    pool: &PgPool,
    rec: &ShowCreatedRecord,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO show_created_events_detail
            (show_id, organizer, name, start_time, end_time, venue, tx_hash, block_number, log_index)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        ON CONFLICT (show_id) DO UPDATE SET
            organizer = EXCLUDED.organizer,
            name = EXCLUDED.name,
            start_time = EXCLUDED.start_time,
            end_time = EXCLUDED.end_time,
            venue = EXCLUDED.venue,
            tx_hash = COALESCE(EXCLUDED.tx_hash, show_created_events_detail.tx_hash),
            block_number = COALESCE(EXCLUDED.block_number, show_created_events_detail.block_number),
            log_index = COALESCE(EXCLUDED.log_index, show_created_events_detail.log_index)
        "#,
    )
    .bind(rec.show_id)
    .bind(&rec.organizer)
    .bind(&rec.name)
    .bind(rec.start_time)
    .bind(rec.end_time)
    .bind(&rec.venue)
    .bind(&rec.tx_hash)
    .bind(rec.block_number)
    .bind(rec.log_index)
    .execute(pool)
    .await?;
    Ok(())
}

/// 按 show_id 查询单条记录。
pub async fn get_show_by_id(
    pool: &PgPool,
    show_id: i64,
) -> Result<Option<ShowCreatedRecord>> {
    let rec = sqlx::query_as::<_, ShowCreatedRecord>(
        r#"
        SELECT show_id, organizer, name, start_time, end_time, venue, tx_hash, block_number, log_index
        FROM show_created_events_detail
        WHERE show_id = $1
        "#,
    )
    .bind(show_id)
    .fetch_optional(pool)
    .await?;
    Ok(rec)
}

/// 最近 100 条（按 show_id 倒序）。
#[allow(unused)]
pub async fn list_shows(pool: &PgPool) -> Result<Vec<ShowCreatedRecord>> {
    let recs = sqlx::query_as::<_, ShowCreatedRecord>(
        r#"
        SELECT show_id, organizer, name, start_time, end_time, venue, tx_hash, block_number, log_index
        FROM show_created_events_detail
        ORDER BY show_id DESC
        LIMIT 100
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(recs)
}

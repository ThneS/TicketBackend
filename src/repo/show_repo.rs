use crate::utils::uint256::DbU256;
use chrono::{DateTime, Utc};
use eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres, Transaction, prelude::FromRow};

// 结构化 ShowCreated 详情记录（拥有所有权字段，便于跨异步边界传递与查询返回）。
#[derive(Debug, Serialize, FromRow)]
pub struct ShowCreatedRecord {
    pub show_id: DbU256,
    pub tx_hash: Option<String>,
    pub block_number: Option<DbU256>,
    pub organizer: String,
    pub log_index: Option<DbU256>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::Type)]
#[sqlx(type_name = "show_status", rename_all = "UPPERCASE")]
pub enum ShowStatus {
    Upcoming,
    Active,
    Ended,
    Cancelled,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ShowCreatedDetailRecord {
    pub show_id: DbU256,
    pub start_time: DbU256,
    pub end_time: DbU256,
    pub total_tickets: DbU256,
    pub ticket_price: DbU256,
    pub decimal: i64,
    pub ticket_sold: DbU256,
    pub organizer: String,
    pub location: String,
    pub name: String,
    pub description: String,
    pub metadata_uri: Option<String>,
    pub status: ShowStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ShowDataRecord {
    pub id: DbU256,
    pub name: String,
    pub description: String,
    pub location: String,
    pub event_time: DbU256,
    pub ticket_price: DbU256,
    pub max_tickets: DbU256,
    pub sold_tickets: DbU256,
    pub is_active: bool,
    pub organizer: String,
    pub created_at: DateTime<Utc>,
}
/// Upsert 一条 ShowCreated 详情记录；以 show_id 为主键，重复则更新基础字段。
pub async fn insert_show_created(
    pool: &PgPool,
    rec: &ShowCreatedRecord,
) -> Result<()> {
    let res = sqlx::query(
        r#"
        INSERT INTO show_created_events (show_id, tx_hash, block_number, organizer, log_index, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (show_id) DO UPDATE
        SET tx_hash = EXCLUDED.tx_hash,
            block_number = EXCLUDED.block_number,
            organizer = EXCLUDED.organizer,
            log_index = EXCLUDED.log_index,
            created_at = EXCLUDED.created_at;
        "#,
    )
    .bind(rec.show_id.clone())
    .bind(&rec.tx_hash)
    .bind(rec.block_number.clone())
    .bind(&rec.organizer)
    .bind(rec.log_index.clone())
    .execute(pool)
    .await?;
    println!("Inserted/Updated show_created_events: {:?}", res);
    Ok(())
}

pub async fn insert_show_created_tx(
    tx: &mut Transaction<'_, Postgres>,
    rec: &ShowCreatedRecord,
) -> Result<()> {
    let query = sqlx::query(
        r#"
        INSERT INTO show_created_events (show_id, tx_hash, block_number, organizer, log_index, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (show_id) DO UPDATE
        SET tx_hash = EXCLUDED.tx_hash,
            block_number = EXCLUDED.block_number,
            organizer = EXCLUDED.organizer,
            log_index = EXCLUDED.log_index,
            created_at = EXCLUDED.created_at;
        "#,
    )
    .bind(rec.show_id.clone())
    .bind(&rec.tx_hash)
    .bind(rec.block_number.clone())
    .bind(&rec.organizer)
    .bind(rec.log_index.clone());
    let res = tx.execute(query).await?;
    println!("Inserted/Updated show_created_events (tx): {:?}", res);
    Ok(())
}

pub async fn insert_show_created_detail(
    pool: &PgPool,
    rec: &ShowCreatedDetailRecord,
) -> Result<()> {
    let query = sqlx::query(
        r#"
        INSERT INTO show_created_events_detail (show_id, start_time, end_time, total_tickets, ticket_price, decimal, ticket_sold, organizer, location, name, description, metadata_uri, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
        ON CONFLICT (show_id) DO UPDATE
        SET start_time = EXCLUDED.start_time,
            end_time = EXCLUDED.end_time,
            total_tickets = EXCLUDED.total_tickets,
            ticket_price = EXCLUDED.ticket_price,
            decimal = EXCLUDED.decimal,
            ticket_sold = EXCLUDED.ticket_sold,
            organizer = EXCLUDED.organizer,
            location = EXCLUDED.location,
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            metadata_uri = EXCLUDED.metadata_uri,
            status = EXCLUDED.status,
            created_at = EXCLUDED.created_at
        RETURNING *;
        "#,
    )
    .bind(rec.show_id.clone())
    .bind(rec.start_time.clone())
    .bind(rec.end_time.clone())
    .bind(rec.total_tickets.clone())
    .bind(rec.ticket_price.clone())
    .bind(rec.decimal)
    .bind(rec.ticket_sold.clone())
    .bind(&rec.organizer)
    .bind(&rec.location)
    .bind(&rec.name)
    .bind(&rec.description)
    .bind(&rec.metadata_uri)
    .bind(&rec.status as &ShowStatus);
    let res = pool.execute(query).await?;
    println!("Inserted/Updated show_created_events_detail: {:?}", res);
    Ok(())
}

pub async fn insert_show_created_detail_tx(
    tx: &mut Transaction<'_, Postgres>,
    rec: &ShowCreatedDetailRecord,
) -> Result<()> {
    let query = sqlx::query(
        r#"
        INSERT INTO show_created_events_detail (show_id, start_time, end_time, total_tickets, ticket_price, decimal, ticket_sold, organizer, location, name, description, metadata_uri, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
        ON CONFLICT (show_id) DO UPDATE
        SET start_time = EXCLUDED.start_time,
            end_time = EXCLUDED.end_time,
            total_tickets = EXCLUDED.total_tickets,
            ticket_price = EXCLUDED.ticket_price,
            decimal = EXCLUDED.decimal,
            ticket_sold = EXCLUDED.ticket_sold,
            organizer = EXCLUDED.organizer,
            location = EXCLUDED.location,
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            metadata_uri = EXCLUDED.metadata_uri,
            status = EXCLUDED.status,
            created_at = EXCLUDED.created_at
        RETURNING *;
        "#,
    )
    .bind(rec.show_id.clone())
    .bind(rec.start_time.clone())
    .bind(rec.end_time.clone())
    .bind(rec.total_tickets.clone())
    .bind(rec.ticket_price.clone())
    .bind(rec.decimal)
    .bind(rec.ticket_sold.clone())
    .bind(&rec.organizer)
    .bind(&rec.location)
    .bind(&rec.name)
    .bind(&rec.description)
    .bind(&rec.metadata_uri)
    .bind(&rec.status as &ShowStatus);
    let res = tx.execute(query).await?;
    println!(
        "Inserted/Updated show_created_events_detail (tx): {:?}",
        res
    );
    Ok(())
}

pub async fn insert_show_data(
    pool: &PgPool,
    rec: &ShowDataRecord,
) -> Result<()> {
    let query = sqlx::query(
        r#"
        INSERT INTO shows (id, name, description, location, event_time, ticket_price, max_tickets, sold_tickets, is_active, organizer, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
            description = EXCLUDED.description,
            location = EXCLUDED.location,
            event_time = EXCLUDED.event_time,
            ticket_price = EXCLUDED.ticket_price,
            max_tickets = EXCLUDED.max_tickets,
            sold_tickets = EXCLUDED.sold_tickets,
            is_active = EXCLUDED.is_active,
            organizer = EXCLUDED.organizer,
            created_at = EXCLUDED.created_at;
        "#,
    )
    .bind(rec.id.clone())
    .bind(&rec.name)
    .bind(&rec.description)
    .bind(&rec.location)
    .bind(rec.event_time.clone())
    .bind(rec.ticket_price.clone())
    .bind(rec.max_tickets.clone())
    .bind(rec.sold_tickets.clone())
    .bind(rec.is_active)
    .bind(&rec.organizer);
    let res = pool.execute(query).await?;
    println!("Inserted/Updated shows: {:?}", res);
    Ok(())
}

pub async fn insert_show_data_tx(
    tx: &mut Transaction<'_, Postgres>,
    rec: &ShowDataRecord,
) -> Result<()> {
    let query = sqlx::query(
        r#"
        INSERT INTO shows (id, name, description, location, event_time, ticket_price, max_tickets, sold_tickets, is_active, organizer, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
            description = EXCLUDED.description,
            location = EXCLUDED.location,
            event_time = EXCLUDED.event_time,
            ticket_price = EXCLUDED.ticket_price,
            max_tickets = EXCLUDED.max_tickets,
            sold_tickets = EXCLUDED.sold_tickets,
            is_active = EXCLUDED.is_active,
            organizer = EXCLUDED.organizer,
            created_at = EXCLUDED.created_at;
        "#,
    )
    .bind(rec.id.clone())
    .bind(&rec.name)
    .bind(&rec.description)
    .bind(&rec.location)
    .bind(rec.event_time.clone())
    .bind(rec.ticket_price.clone())
    .bind(rec.max_tickets.clone())
    .bind(rec.sold_tickets.clone())
    .bind(rec.is_active)
    .bind(&rec.organizer);
    let res = tx.execute(query).await?;
    println!("Inserted/Updated shows (tx): {:?}", res);
    Ok(())
}

/// 方便的聚合写接口：在一个事务中写入三张表
pub async fn upsert_show_all(
    pool: &PgPool,
    basic: &ShowCreatedRecord,
    detail: &ShowCreatedDetailRecord,
    data: &ShowDataRecord,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    insert_show_created_tx(&mut tx, basic).await?;
    insert_show_created_detail_tx(&mut tx, detail).await?;
    insert_show_data_tx(&mut tx, data).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_show_by_id(
    pool: &PgPool,
    show_id: DbU256,
) -> Result<Option<ShowDataRecord>> {
    let rec = sqlx::query_as::<_, ShowDataRecord>(
        r#"
        SELECT id, name, description, location, event_time, ticket_price, max_tickets, sold_tickets, is_active, organizer, created_at
        FROM shows
        WHERE id = $1;
        "#,
    )
    .bind(show_id)
    .fetch_optional(pool)
    .await?;
    Ok(rec)
}

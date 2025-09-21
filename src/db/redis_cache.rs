use crate::{repo::show_repo::ShowDataRecord, utils::uint256::DbU256};
use eyre::Result;
use redis::AsyncCommands;

/// 统一构造 Redis 缓存 key（使用 0x 十六进制规范化形式，避免十进制/十六进制混用导致重复）
pub fn show_cache_key(show_id: &DbU256) -> String {
    format!("show:{}", show_id.to_hex0x())
}

pub async fn get_redis_connection(
    redis_url: &str,
) -> Result<redis::aio::MultiplexedConnection> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_multiplexed_async_connection().await?;
    Ok(conn)
}

pub async fn cache_show(
    conn: &mut redis::aio::MultiplexedConnection,
    show_id: DbU256,
    show: &ShowDataRecord,
) -> Result<()> {
    let key = show_cache_key(&show_id);
    let value = serde_json::to_string(show)?;
    let _: () = conn.set_ex(key, value, 3600).await?;
    Ok(())
}
pub async fn get_cached_show(
    conn: &mut redis::aio::MultiplexedConnection,
    show_id: DbU256,
) -> Result<Option<ShowDataRecord>> {
    let key = show_cache_key(&show_id);
    let value: Option<String> = conn.get(key).await?;
    if let Some(json) = value {
        let show: ShowDataRecord = serde_json::from_str(&json)?;
        Ok(Some(show))
    } else {
        Ok(None)
    }
}
pub async fn delete_cached_show(
    conn: &mut redis::aio::MultiplexedConnection,
    show_id: DbU256,
) -> Result<()> {
    let key = show_cache_key(&show_id);
    let _: () = conn.del(key).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::str::FromStr;

    #[test]
    fn test_show_cache_key_normalizes_input() {
        let dec = DbU256::from_str("123456789").unwrap();
        let hex = DbU256::from_str("0x075bcd15").unwrap();
        assert_eq!(show_cache_key(&dec), show_cache_key(&hex));
        assert!(show_cache_key(&dec).starts_with("show:"));
    }

    #[test]
    fn test_showdata_json_roundtrip() {
        let rec = ShowDataRecord {
            id: DbU256::from_str("0x1").unwrap(),
            name: "Concert".to_string(),
            description: "A big show".to_string(),
            location: "City Hall".to_string(),
            event_time: DbU256::from_str("1735689600").unwrap(),
            ticket_price: DbU256::from_str("1000000000000000000").unwrap(),
            max_tickets: DbU256::from_str("1000").unwrap(),
            sold_tickets: DbU256::from_str("10").unwrap(),
            is_active: true,
            organizer: "alice".to_string(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&rec).unwrap();
        let back: ShowDataRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(rec.id.to_string(), back.id.to_string());
        assert_eq!(rec.name, back.name);
        assert_eq!(rec.location, back.location);
        assert_eq!(rec.ticket_price.to_string(), back.ticket_price.to_string());
        assert_eq!(rec.max_tickets.to_string(), back.max_tickets.to_string());
        assert_eq!(rec.sold_tickets.to_string(), back.sold_tickets.to_string());
        assert_eq!(rec.is_active, back.is_active);
        assert_eq!(rec.organizer, back.organizer);
    }
}

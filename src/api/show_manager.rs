use crate::{
    api::{
        AppState,
        error::AppError,
        request::{PathShowId, ValidatedJson, ValidatedQuery},
        response::ok,
        schema::Pagination,
    },
    repo::show_repo::{ShowDataRecord, get_show_by_id, repo_list_shows},
    utils::uint256::{DbU256, U256},
};
use axum::extract::State;
use axum::response::Response;
use chrono::Utc;
use serde::Deserialize;

// === DTOs ===
#[derive(Debug, Deserialize)]
pub struct CreateShowReq {
    pub id: DbU256,
    pub name: String,
    pub description: String,
    pub location: String,
    pub event_time: DbU256,
    pub ticket_price: DbU256,
    pub max_tickets: DbU256,
    pub organizer: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShowReq {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub event_time: Option<DbU256>,
    pub ticket_price: Option<DbU256>,
    pub max_tickets: Option<DbU256>,
    pub is_active: Option<bool>,
}

// 简单校验：示例性实现 Validate trait
impl crate::api::schema::Validate for CreateShowReq {
    type Err = crate::api::schema::ValidationError;
    fn validate(self) -> Result<Self, Self::Err> {
        if self.name.trim().is_empty() {
            return Err(crate::api::schema::ValidationError(
                "name empty".into(),
            ));
        }
        if self.description.len() > 2048 {
            return Err(crate::api::schema::ValidationError(
                "description too long".into(),
            ));
        }
        if self.location.len() > 512 {
            return Err(crate::api::schema::ValidationError(
                "location too long".into(),
            ));
        }
        Ok(self)
    }
}
impl crate::api::schema::Validate for UpdateShowReq {
    type Err = crate::api::schema::ValidationError;
    fn validate(self) -> Result<Self, Self::Err> {
        Ok(self)
    }
}

fn build_record_from_create(req: CreateShowReq) -> ShowDataRecord {
    ShowDataRecord {
        id: req.id,
        name: req.name,
        description: req.description,
        location: req.location,
        event_time: req.event_time,
        ticket_price: req.ticket_price,
        max_tickets: req.max_tickets.clone(),
        sold_tickets: DbU256(U256::from(0u64)),
        is_active: true,
        organizer: req.organizer,
        created_at: Utc::now(),
    }
}

pub async fn show_with_id(
    State(state): State<AppState>,
    PathShowId(show_id): PathShowId,
) -> axum::response::Response {
    let db = &state.api.db;
    match get_show_by_id(db.pool(), show_id.clone()).await {
        Ok(Some(rec)) => ok(rec),
        Ok(None) => AppError::ShowNotFound(show_id.to_string()).to_response(),
        Err(e) => AppError::Internal(e.to_string()).to_response(),
    }
}

pub async fn list_shows(
    State(state): State<AppState>,
    ValidatedQuery(p): ValidatedQuery<Pagination>,
) -> axum::response::Response {
    let db = &state.api.db;
    match repo_list_shows(db.pool(), p.limit, p.offset).await {
        Ok(shows) => ok(shows),
        Err(e) => AppError::Database(e.to_string()).to_response(),
    }
}

pub async fn create_show(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateShowReq>,
) -> Response {
    let db = &state.api.db;
    let rec = build_record_from_create(body);
    // 简单使用 insert_show_data（没有 upsert 冲突处理细节，这里演示）
    match crate::repo::show_repo::insert_show_data(db.pool(), &rec).await {
        Ok(_) => ok(rec),
        Err(e) => AppError::Database(e.to_string()).to_response(),
    }
}

pub async fn update_show(
    State(state): State<AppState>,
    PathShowId(show_id): PathShowId,
    ValidatedJson(update): ValidatedJson<UpdateShowReq>,
) -> Response {
    let db = &state.api.db;
    // 先获取
    let existing = match get_show_by_id(db.pool(), show_id.clone()).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return AppError::ShowNotFound(show_id.to_string()).to_response();
        }
        Err(e) => return AppError::Database(e.to_string()).to_response(),
    };
    // 构造更新 SQL（仅示范：直接替换字段，不做部分字段保持原值的批量 set 生成器）
    let new_rec = ShowDataRecord {
        name: update.name.unwrap_or(existing.name),
        description: update.description.unwrap_or(existing.description),
        location: update.location.unwrap_or(existing.location),
        event_time: update.event_time.unwrap_or(existing.event_time),
        ticket_price: update.ticket_price.unwrap_or(existing.ticket_price),
        max_tickets: update.max_tickets.unwrap_or(existing.max_tickets),
        sold_tickets: existing.sold_tickets, // 不改
        is_active: update.is_active.unwrap_or(existing.is_active),
        organizer: existing.organizer,
        id: existing.id,
        created_at: existing.created_at, // 保留原创建时间
    };
    match crate::repo::show_repo::insert_show_data(db.pool(), &new_rec).await {
        Ok(_) => ok(new_rec),
        Err(e) => AppError::Database(e.to_string()).to_response(),
    }
}

pub async fn delete_show(
    State(state): State<AppState>,
    PathShowId(show_id): PathShowId,
) -> Response {
    let db = &state.api.db;
    let res = sqlx::query("DELETE FROM shows WHERE id = $1")
        .bind(show_id.clone())
        .execute(db.pool())
        .await;
    match res {
        Ok(r) => {
            if r.rows_affected() == 0 {
                AppError::ShowNotFound(show_id.to_string()).to_response()
            } else {
                ok(
                    serde_json::json!({"deleted": true, "id": show_id.to_string()}),
                )
            }
        }
        Err(e) => AppError::Database(e.to_string()).to_response(),
    }
}

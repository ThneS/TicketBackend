use crate::{
    api::AppState, repo::show_repo::get_show_by_id, utils::uint256::DbU256,
};
use axum::extract::{Path, State};

pub async fn show_with_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> String {
    let db = &state.api.db;
    // 支持十进制与 0x/0X 开头的十六进制
    match id.parse::<DbU256>() {
        Ok(show_id) => match get_show_by_id(db.pool(), show_id.clone()).await {
            Ok(Some(rec)) => format!("Found show: {:?}", rec),
            Ok(None) => format!("No show found with id {}", show_id),
            Err(e) => format!("Database error: {}", e),
        },
        Err(e) => format!("Invalid show id: {} ({})", id, e),
    }
}

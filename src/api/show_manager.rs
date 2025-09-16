use crate::AppState;
use axum::extract::{Path, State};

pub async fn show_with_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> String {
    let db = &state.api.db;
    match id.parse::<i64>() {
        Ok(show_id) => {
            match crate::repo::show_repo::get_show_by_id(db.pool(), show_id)
                .await
            {
                Ok(Some(rec)) => format!("Found show: {:?}", rec),
                Ok(None) => format!("No show found with id {}", show_id),
                Err(e) => format!("Database error: {}", e),
            }
        }
        Err(e) => format!("Invalid show id: {} ({})", id, e),
    }
}

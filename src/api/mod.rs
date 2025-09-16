use crate::db::Db;
pub mod show_manager;
#[derive(Debug, Clone)]
pub struct ApiContext {
    pub db: Db,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub api: ApiContext,
}

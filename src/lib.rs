use std::sync::Arc;

use surrealdb::{engine::any::Any, Surreal};
// use websockets::{token_cache::TokenCache, ws_manager::WsDataManager};

pub mod database;
pub mod errors;
pub mod guards;
pub mod models;
pub mod routes;
pub mod utils;
pub mod websockets;

#[derive(Debug)]
pub struct AppState {
    pub db: Arc<Surreal<Any>>,
    // pub token_cache: Arc<Mutex<TokenCache>>,
    // pub ws_manager: Arc<Mutex<WsDataManager>>,
}

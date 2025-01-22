use std::sync::Arc;

use surrealdb::{engine::any::Any, Surreal};
use tokio::sync::Mutex;
use websockets::{token_cache::TokenCache, ws_manager::WsDataManager, ws_server::WsServerHandle};

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
    pub ws_server_handle: WsServerHandle,
    pub token_cache: Arc<Mutex<TokenCache>>,
    pub ws_manager: Arc<Mutex<WsDataManager>>,
}

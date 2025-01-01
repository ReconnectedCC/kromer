use actix::Addr;
use surrealdb::{engine::any::Any, Surreal};
use websockets::server::WebSocketServer;
use ws::actors::server::WebSocketServer as NewWebSocketServer;

pub mod database;
pub mod errors;
pub mod guards;
pub mod routes;
pub mod websockets;
pub mod ws;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Surreal<Any>,
    pub ws_manager: Addr<WebSocketServer>,
    pub new_ws_manager: Addr<NewWebSocketServer>,
}

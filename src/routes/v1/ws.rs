use std::str::FromStr;

//use actix::prelude::*;
use actix_web::{get, post, HttpRequest, Responder};
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use serde_json::json;
use surrealdb::Uuid;
use tokio::task::spawn_local;
use tokio::time::sleep;

use crate::database::models::wallet::Model as Wallet;
use crate::errors::wallet::WalletError;
use crate::errors::websocket::WebSocketError;
use crate::websockets::handler::handle_ws;
use crate::websockets::types::common::WebSocketTokenData;
use crate::websockets::utils;
use crate::websockets::wrapped_ws::WrappedWsData;
use crate::{errors::KromerError, AppState};

#[derive(serde::Deserialize)]
struct WsConnDetails {
    privatekey: String,
}

#[post("/start")]
pub async fn setup_ws(
    _req: HttpRequest,
    state: Data<AppState>,
    details: Option<web::Json<WsConnDetails>>,
    _stream: web::Payload,
) -> Result<HttpResponse, KromerError> {
    let tracing_span = tracing::span!(tracing::Level::DEBUG, "setup_ws_route");
    let _tracing_enter = tracing_span.enter();

    let db = &state.db;
    let token_cache_mutex = state.token_cache.clone();

    let ws_privatekey = details.map(|json_details| json_details.privatekey.clone());
    let mut address = "guest".to_string();
    let ws_privatekey2 = ws_privatekey.clone();

    if let Some(check_key) = ws_privatekey {
        // This should error back in the request if the wallet key is invalid.
        let wallet = Wallet::verify(db, check_key)
            .await
            .map_err(KromerError::Database)?
            .ok_or_else(|| KromerError::Wallet(WalletError::InvalidPassword))?;

        address = wallet.address;
    }

    let uuid = Uuid::new_v4();
    let token_params = WebSocketTokenData {
        address,
        privatekey: ws_privatekey2,
    };

    let mut token_cache = token_cache_mutex.lock().await;
    token_cache.add_token(uuid, token_params);

    // Spawn a green thread that will handle token cleanup.
    let token_cache2 = token_cache_mutex.clone();
    let uuid2 = uuid;
    tokio::spawn(async move {
        let tracing_span = tracing::span!(tracing::Level::DEBUG, "spawned_token_cleanup");
        let _tracing_enter = tracing_span.enter();
        sleep(std::time::Duration::from_secs(30)).await;
        let mut token_cache = token_cache2.lock().await;

        if token_cache.check_token(uuid) {
            tracing::info!("Token expired (30 secs)");
            token_cache.remove_token(uuid2);
        }
    });

    // Make the URL and return it to the user.
    let url = match utils::make_url::make_url(uuid) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(HttpResponse::Ok().json(json!({
        "ok": true,
        "url": url,
        "expires": 30
    })))
}

#[get("/gateway/{token}")]
//#[allow(clippy::await_holding_lock)]
pub async fn gateway(
    req: HttpRequest,
    body: web::Payload,
    state: Data<AppState>,
    token: web::Path<String>,
) -> Result<impl Responder, KromerError> {
    let debug_span = tracing::span!(tracing::Level::INFO, "ws_gateway_route");
    let _tracing_debug_enter = debug_span.enter();

    let token_as_string = token.into_inner();
    tracing::info!("Request with token string: {token_as_string}");

    let uuid_result = Uuid::from_str(&token_as_string)
        .map_err(|_| KromerError::WebSocket(WebSocketError::InvalidUuid));

    // This is a one off message, and we don't want to actually open the server handling
    if uuid_result.is_err() {
        tracing::info!("Token {token_as_string} was not convertible into UUID");
        return send_error_message(req.clone(), body).await;
    }

    // Unwrap should be fine, we checked already if there was an error
    let uuid = uuid_result.unwrap_or_default();

    // Check token, send a one off message if it's not okay, and don't open WS server handling
    let token_cache_mutex = state.token_cache.clone();
    let mut token_cache = token_cache_mutex.lock().await;
    if !token_cache.check_token(uuid) {
        drop(token_cache);
        tracing::info!("Token {uuid} was not found in cache");
        return send_error_message(req.clone(), body).await;
    }

    // Token was valid, now we can remove it from the cache
    tracing::info!("Token {uuid} was valid");
    let token_params = token_cache
        .remove_token(uuid)
        .ok_or_else(|| KromerError::WebSocket(WebSocketError::InvalidUuid))?;
    drop(token_cache);

    // Clone a WsServerHandle so that we already have the Server's Command Channel referenced.
    let ws_server_handle = state.ws_server_handle.clone();
    let (response, session, msg_stream) = actix_ws::handle(&req, body)
        .map_err(|_| KromerError::WebSocket(WebSocketError::HandshakeError))?;

    // Add this data to a struct for easy access to the session information
    let wrapped_ws_data = WrappedWsData::new(uuid, token_params.address, token_params.privatekey);

    spawn_local(handle_ws(
        state.clone(),
        wrapped_ws_data,
        ws_server_handle,
        session,
        msg_stream,
    ));

    Ok(response)
}

async fn send_error_message(
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, KromerError> {
    let (response, mut session, _msg_stream) =
        actix_ws::handle(&req, body).map_err(|_| WebSocketError::HandshakeError)?;

    let error_msg = json!({"ok": false, "error": "invalid_websocket_token", "message": "Invalid websocket token", "type": "error"});

    let _result = session.text(error_msg.to_string()).await;

    Ok(response)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/ws").service(setup_ws).service(gateway));
}

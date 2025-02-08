use std::str::FromStr;

use actix::spawn;
//use actix::prelude::*;
use actix_web::{get, post, HttpRequest, Responder};
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use serde_json::json;
use surrealdb::Uuid;
use tokio::time::sleep;

use crate::database::models::wallet::Model as Wallet;
use crate::errors::krist::{address::AddressError, websockets::WebSocketError, KristError};
use crate::websockets::handler::handle_ws;
use crate::websockets::types::common::WebSocketTokenData;
use crate::websockets::wrapped_ws::WrappedWsData;
use crate::websockets::ws_server::WsServer;
use crate::websockets::{utils, WebSocketServer};
use crate::AppState;

#[derive(serde::Deserialize)]
struct WsConnDetails {
    privatekey: String,
}

#[post("/start")]
#[tracing::instrument(name = "setup_ws_route", level = "debug", skip_all)]
pub async fn setup_ws(
    state: web::Data<AppState>,
    server: web::Data<WebSocketServer>,
    details: Option<web::Json<WsConnDetails>>,
    _stream: web::Payload,
) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let private_key = details.map(|json_details| json_details.privatekey.clone());

    let uuid = match private_key {
        Some(private_key) => {
            let wallet = Wallet::verify_address(db, &private_key)
                .await
                .map_err(|_| KristError::Address(AddressError::AuthFailed))?;
            let model = wallet.address;

            let token_data = WebSocketTokenData::new(model.address, Some(private_key));

            server.obtain_token(token_data).await
        }
        None => {
            let token_data = WebSocketTokenData::new("guest".into(), None);

            server.obtain_token(token_data).await
        }
    };

    // Make the URL and return it to the user.
    let url = match utils::make_url::make_url(uuid) {
        Ok(value) => value,
        Err(_) => return Err(KristError::Custom("server_config_error")),
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
) -> Result<impl Responder, KristError> {
    let debug_span = tracing::span!(tracing::Level::INFO, "ws_gateway_route");
    let _tracing_debug_enter = debug_span.enter();

    let token_as_string = token.into_inner();
    tracing::info!("Request with token string: {token_as_string}");

    let uuid_result = Uuid::from_str(&token_as_string)
        .map_err(|_| KristError::WebSocket(WebSocketError::InvalidWebsocketToken));

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
        .ok_or_else(|| KristError::WebSocket(WebSocketError::InvalidWebsocketToken))?;
    drop(token_cache);

    // TODO: New implementation a few lines down, spawn it per thread (green threaded)
    //let ws_server_handle = state.ws_server_handle.clone();

    // Create the actual handler for the WebSocketSession
    let (response, session, msg_stream) = actix_ws::handle(&req, body)
        .map_err(|_| KristError::WebSocket(WebSocketError::HandshakeError))?;

    // Add this data to a struct for easy access to the session information
    let wrapped_ws_data = WrappedWsData::new(uuid, token_params.address, token_params.privatekey);

    // TODO: Verify this spawns a green thread to handle the WS Server
    let (ws_server, ws_server_handle) = WsServer::new();

    let ws_server_task = spawn(ws_server.run());

    spawn(async move {
        if let Err(err) = ws_server_task.await {
            tracing::error!("WebSocket server error: {:?}", err);
        }
    });

    spawn(handle_ws(
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
) -> Result<HttpResponse, KristError> {
    let (response, mut session, _msg_stream) =
        actix_ws::handle(&req, body).map_err(|_| WebSocketError::HandshakeError)?;

    let error_msg = json!({"ok": false, "error": "invalid_websocket_token", "message": "Invalid websocket token", "type": "error"});

    let _result = session.text(error_msg.to_string()).await;

    Ok(response)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/ws").service(setup_ws).service(gateway));
}

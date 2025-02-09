use std::str::FromStr;

use actix_web::{get, post, HttpRequest};
use actix_web::{web, HttpResponse};
use serde_json::json;
use surrealdb::Uuid;

use crate::database::models::wallet::Model as Wallet;
use crate::errors::krist::KristErrorExt;
use crate::errors::krist::{address::AddressError, websockets::WebSocketError, KristError};
use crate::websockets::types::common::WebSocketTokenData;
use crate::websockets::{utils, WebSocketServer, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};
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
#[tracing::instrument(name = "ws_gateway_route", level = "info", fields(token = *token), skip_all,)]
pub async fn gateway(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<AppState>,
    server: web::Data<WebSocketServer>,
    token: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let token = token.into_inner();
    tracing::info!("Request with token string: {token}");

    let (response, mut session, stream) = actix_ws::handle(&req, body)?;

    let uuid_result = Uuid::from_str(&token)
        .map_err(|_| KristError::WebSocket(WebSocketError::InvalidWebsocketToken));

    if let Err(err) = uuid_result {
        let error = json!({
            "ok": false,
            "error": err.error_type(),
            "message": err.to_string(),
            "type": "error"
        });

        let _ = session.text(error.to_string()).await;

        return Ok(response);
    }
    let uuid = uuid_result.unwrap(); // SAFETY: We handled the error above

    let data_result = server.use_token(&uuid).await;
    if let Err(_) = data_result {
        let error = json!({
            "ok": false,
            "error": "invalid_websocket_token",
            "message": "Invalid websocket token",
            "type": "error"
        });

        let _ = session.text(error.to_string()).await;

        return Ok(response);
    }

    let mut stream = stream
        .max_frame_size(64 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    // This is a one off message, and we don't want to actually open the server handling
    // if uuid_result.is_err() {
    //     tracing::info!("Token {token} was not convertible into UUID");

    //     return send_error_message(req, body).await;
    // }

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

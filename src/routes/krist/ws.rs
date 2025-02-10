use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use actix_web::rt::time;
use actix_web::{get, post, HttpRequest};
use actix_web::{web, HttpResponse};
use actix_ws::AggregatedMessage;
use chrono::Utc;
use serde_json::json;
use surrealdb::Uuid;
use tokio::sync::Mutex;

use crate::database::models::wallet::Model as Wallet;
use crate::errors::krist::KristErrorExt;
use crate::errors::krist::{address::AddressError, websockets::WebSocketError, KristError};
use crate::models::websockets::{WebSocketMessage, WebSocketMessageInner};
use crate::websockets::types::common::WebSocketTokenData;
use crate::websockets::types::convert_to_iso_string;
use crate::websockets::{handler, utils, WebSocketServer, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};
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
    let server = server.into_inner(); // lol
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
    if data_result.is_err() {
        let error = json!({
            "ok": false,
            "error": "invalid_websocket_token",
            "message": "Invalid websocket token",
            "type": "error"
        });

        let _ = session.text(error.to_string()).await;

        return Ok(response);
    }
    let data = data_result.unwrap(); // SAFETY: We handled the error above

    let mut stream = stream
        .max_frame_size(64 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    server.insert_session(uuid, session.clone(), data).await; // Not a big fan of cloning but here it is needed.

    let alive = Arc::new(Mutex::new(Instant::now()));
    let mut session2 = session.clone();
    let alive2 = alive.clone();

    handler::send_hello_message(&mut session).await;

    // Heartbeat handling
    actix_web::rt::spawn(async move {
        let mut interval = time::interval(HEARTBEAT_INTERVAL);

        loop {
            interval.tick().await;
            if session2.ping(b"").await.is_err() {
                break;
            }

            let cur_time = convert_to_iso_string(Utc::now());
            let message = WebSocketMessage {
                ok: None,
                id: None,
                r#type: WebSocketMessageInner::Keepalive {
                    server_time: cur_time,
                },
            };

            let return_message =
                serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_string()); // ...what
            let _ = session2.text(return_message).await;

            if Instant::now().duration_since(*alive2.lock().await) > CLIENT_TIMEOUT {
                let _ = session2.close(None).await;
                break;
            }
        }
    });

    // Messgage handling code here
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.recv().await {
            match msg {
                AggregatedMessage::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        tracing::error!("Failed to send pong back to session");
                        return;
                    }
                }

                AggregatedMessage::Text(string) => {
                    if string.chars().count() > 512 {
                        // TODO: Possibly use error message struct in models
                        // This isn't super necessary though and this shortcut saves some unnecessary error handling...
                        let error_msg = json!({
                            "ok": "false",
                            "error": "message_too_long",
                            "message": "Message larger than 512 characters",
                            "type": "error"
                        })
                        .to_string();
                        tracing::info!("Message received was larger than 512 characters");

                        let _ = session.text(error_msg).await;
                    } else {
                        tracing::debug!("Message received: {string}");

                        let process_result =
                            handler::process_text_msg(&state.db, &server, &uuid, &string).await;

                        if let Ok(message) = process_result {
                            let msg = serde_json::to_string(&message)
                                .expect("Failed to serialize message into string");
                            let _ = session.text(msg).await;
                        } else {
                            tracing::error!("Error in processing message")
                        }
                    }
                }

                AggregatedMessage::Close(reason) => {
                    let _ = session.close(reason).await;

                    tracing::info!("Got close, cleaning up");
                    server.cleanup_session(&uuid).await;

                    return;
                }

                AggregatedMessage::Pong(_) => {
                    *alive.lock().await = Instant::now();
                }

                _ => (), // Binary data is just ignored
            }
        }

        let _ = session.close(None).await;
        server.cleanup_session(&uuid).await;
    });

    Ok(response)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/ws").service(setup_ws).service(gateway));
}

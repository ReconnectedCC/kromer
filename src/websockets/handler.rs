use crate::{
    errors::{websocket::WebSocketError, KromerError},
    models::{
        error::ErrorResponse,
        motd::{Constants, CurrencyInfo, DetailedMotd, PackageInfo},
        websockets::{
            IncomingWebsocketMessage, OutgoingWebSocketMessage, ResponseMessageType,
            WebSocketMessageType, WsSessionModification,
        },
    },
    websockets::routes::{
        addresses::get_address,
        auth::perform_logout,
        subscriptions::{
            get_subscription_level, get_valid_subscription_levels, subscribe, unsubscribe,
        }, transactions::make_transaction,
    },
    AppState,
};
use std::{
    pin::pin,
    sync::Arc,
    time::{Duration, Instant},
};

use super::{
    routes::auth::perform_login, utils::datetime::convert_to_iso_string, wrapped_ws::WrappedWsData,
    ws_server::WsServerHandle,
};

use crate::websockets::routes::me::get_me as route_get_me;

use actix_web::web::Data;
use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    stream::AbortHandle,
    StreamExt,
};
use serde_json::json;
use surrealdb::{engine::any::Any, Surreal, Uuid};
use tokio::{sync::mpsc, task::JoinHandle, time::interval};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
// TODO: 30 seconds for debugging, should probably be around 10 seconds for client timeout in prooduction
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(10);

pub async fn handle_ws(
    state: Data<AppState>,
    wrapped_ws_data: WrappedWsData,
    ws_server_handle: WsServerHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) -> Result<(), KromerError> {
    let debug_span = tracing::span!(tracing::Level::DEBUG, "WS_HANDLER");
    let _tracing_debug_enter = debug_span.enter();

    let mut last_heartbeat = Instant::now();
    let mut heartbeat_interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    // Internally, the WsServer uses a channel ID to facilitate sending messages.
    let channel_id = match ws_server_handle.connect(conn_tx, wrapped_ws_data.token).await {
        Ok(value) => value,
        Err(_) => return Err(KromerError::WebSocket(WebSocketError::HandshakeError)),
    };

    let msg_stream = msg_stream
        .max_frame_size(64 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let (_keepalive_join_handle, keepalive_abort_handle) =
        spawn_keepalive(ws_server_handle.clone(), channel_id).await;

    let mut msg_stream = pin!(msg_stream);

    // TODO: This state needs to be set globally for Event Subscription Lookups, so perhaps a Mutex is still the preferred option here.
    let mut ws_metadata = wrapped_ws_data.clone();

    // Send the hello message
    send_hello_message(&mut session).await;

    let close_reason = loop {
        // Stack pin futures
        let tick = pin!(heartbeat_interval.tick());
        let msg_rx = pin!(conn_rx.recv());

        let messages = pin!(select(msg_stream.next(), msg_rx));

        match select(messages, tick).await {
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => {
                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        // Let's ignore pong errors, as they shouldn't really matter here
                        let _ = match session.pong(&bytes).await {
                            Ok(_) => Ok(()),
                            Err(err) => {
                                tracing::error!("Ping failure: {:?}", err);
                                Err(err)
                            }
                        };
                    }

                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }

                    AggregatedMessage::Text(text) => {
                        if text.chars().count() > 512 {
                            // TODO: Possibly use error message struct in models
                            // This isn't super necessary though and this shortcut saves some unnecessary error handling...
                            let error_msg = json!({
                                "ok": "false",
                                "error": "message_too_long",
                                "message": "Message larger than 512 characters",
                                "type": "error"})
                            .to_string();
                            tracing::info!("Message received was larger than 512 characters");
                            let _ = session.text(error_msg).await;
                        } else {
                            tracing::info!("Message received: {text}");
                            let process_result = process_text_msg(
                                &state.db,
                                &ws_metadata,
                                &ws_server_handle,
                                &mut session,
                                &text,
                            )
                            .await;

                            // If there were updates to the Metadata, we want to do them
                            // TODO: Might need to be a global mutex so subscriptions have access to this as well.
                            if let Ok(Some(new_metadata)) = process_result {
                                ws_metadata = new_metadata;
                            } else if process_result.is_err() {
                                tracing::error!("Error in processing message")
                            }
                        }
                    }

                    AggregatedMessage::Binary(_bin) => {
                        tracing::warn!("unexpected binary message");
                    }

                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            // client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                tracing::error!("{}", err);
                break None;
            }

            // client WebSocket stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,

            // chat messages received from other room participants
            Either::Left((Either::Right((Some(chat_msg), _)), _)) => {
                let _ = session.text(chat_msg).await;
            }

            // all connection's message senders were dropped
            Either::Left((Either::Right((None, _)), _)) => unreachable!(
                "all connection message senders were dropped; chat server may have panicked"
            ),

            // heartbeat internal tick
            Either::Right((_inst, _)) => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    tracing::info!(
                        "Client has not sent heartbeat in over {CLIENT_TIMEOUT:?}; disconnecting"
                    );
                    break None;
                }

                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };

    keepalive_abort_handle.abort();

    let _ = ws_server_handle.disconnect(channel_id);

    let _ = session.close(close_reason).await;

    Ok(())
}

async fn process_text_msg(
    db: &Arc<Surreal<Any>>,
    ws_metadata: &WrappedWsData,
    _ws_server: &WsServerHandle,
    session: &mut actix_ws::Session,
    text: &str,
) -> Result<Option<WrappedWsData>, KromerError> {
    // strip leading and trailing whitespace (spaces, newlines, etc.)
    let msg = text.trim();

    // TODO: potentially change how this serialization is handled, so that we can properly extract "Invalid Parameter" errors.
    let parsed_msg_result: Result<IncomingWebsocketMessage, serde_json::Error> =
        serde_json::from_str(msg);

    let parsed_msg = match parsed_msg_result {
        Ok(value) => value,
        Err(err) => {
            tracing::error!("Serde error: {}", err);
            tracing::info!("Could not parse JSON for UUID: {}", ws_metadata.token);
            return Err(KromerError::WebSocket(WebSocketError::JsonParseRead));
        }
    };

    let msg_type = parsed_msg.message_type;
    let msg_id = parsed_msg.id;

    let mut ws_modification_data = WsSessionModification {
        msg_type: None,
        wrapped_ws_data: None,
    };

    tracing::debug!("{:?}", msg_type);

    match msg_type {
        WebSocketMessageType::Address {
            address,
            fetch_names,
        } => {
            ws_modification_data = get_address(address, fetch_names, msg_id, db).await;
        }

        WebSocketMessageType::Login {
            login_details: Some(login_details),
        } => {
            let auth_result = perform_login(ws_metadata, login_details, db.to_owned()).await;

            // Generate the response if it's okay
            if auth_result.is_ok() {
                let new_auth_data = auth_result.unwrap();
                let wrapped_ws_data = new_auth_data.0;
                let wallet = new_auth_data.1;
                let new_ws_modification_data = WsSessionModification {
                    msg_type: Some(OutgoingWebSocketMessage {
                        ok: Some(true),
                        id: msg_id.clone(),
                        message: WebSocketMessageType::Response {
                            message: ResponseMessageType::Login {
                                address: Some(wallet),
                                is_guest: false,
                            },
                        },
                    }),
                    wrapped_ws_data: Some(wrapped_ws_data),
                };

                ws_modification_data = new_ws_modification_data;
            } else {
                // If the auth failed, we can just perform a "me" request.
                let me_data = route_get_me(msg_id, db, ws_metadata).await;
                if me_data.is_ok() {
                    ws_modification_data = WsSessionModification {
                        msg_type: Some(me_data.unwrap()),
                        wrapped_ws_data: None,
                    }
                }
            }
        }
        WebSocketMessageType::Logout => {
            let auth_result = perform_logout(ws_metadata).await;

            let new_ws_modification_data = WsSessionModification {
                msg_type: Some(OutgoingWebSocketMessage {
                    ok: Some(true),
                    id: msg_id.clone(),
                    message: WebSocketMessageType::Response {
                        message: ResponseMessageType::Logout { is_guest: true },
                    },
                }),
                wrapped_ws_data: Some(auth_result),
            };

            ws_modification_data = new_ws_modification_data;
        }

        WebSocketMessageType::MakeTransaction {
            private_key,
            to,
            amount,
            metadata,
            request_id
        } => {
            ws_modification_data = make_transaction(db, msg_id, private_key, to, amount, metadata, request_id).await;
        }

        WebSocketMessageType::Subscribe { event } => {
            ws_modification_data = subscribe(ws_metadata, msg_id, event);
        }

        WebSocketMessageType::Unsubscribe { event } => {
            ws_modification_data = unsubscribe(ws_metadata, msg_id, event)
        }

        WebSocketMessageType::GetSubscriptionLevel => {
            ws_modification_data = get_subscription_level(ws_metadata, msg_id);
        }

        WebSocketMessageType::GetValidSubscriptionLevels => {
            ws_modification_data = get_valid_subscription_levels(msg_id);
        }

        // Mining will be perma-disabled
        WebSocketMessageType::SubmitBlock => {
            ws_modification_data = WsSessionModification {
                msg_type: Some(OutgoingWebSocketMessage {
                    ok: Some(false),
                    id: msg_id,
                    message: WebSocketMessageType::Error {
                        error: ErrorResponse {
                            error: "mining_disabled".to_string(),
                            message: Some("Mining disabled".to_string()),
                        },
                    },
                }),
                wrapped_ws_data: None,
            }
        }

        WebSocketMessageType::Me => {
            let me_data = route_get_me(msg_id, db, ws_metadata).await;
            if me_data.is_ok() {
                ws_modification_data = WsSessionModification {
                    msg_type: Some(me_data.unwrap()),
                    wrapped_ws_data: None,
                }
            }
        }

        _ => {
            // TODO: This is just an example, we should error here with a good error message.
            // We should tell the user there was a syntax error with the type in their message.
        }
    };

    // This should be a response message here
    if let Some(message) = ws_modification_data.msg_type {
        let _ = session
            .text(serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_string()))
            .await;
    }

    // This should be the updated WS auth data
    if let Some(auth) = ws_modification_data.wrapped_ws_data {
        return Ok(Some(auth));
    }

    Ok(None)
}

async fn spawn_keepalive(ws_server: WsServerHandle, conn: Uuid) -> (JoinHandle<()>, AbortHandle) {
    let (abort_handle, _) = AbortHandle::new_pair();

    let join_handle = tokio::spawn(async move {
        let mut interval = interval(KEEPALIVE_INTERVAL);

        loop {
            interval.tick().await;
            let cur_time = convert_to_iso_string(chrono::offset::Utc::now());
            let keepalive_time = WebSocketMessageType::Keepalive {
                server_time: cur_time.clone(),
            };
            let return_message =
                serde_json::to_string(&keepalive_time).unwrap_or_else(|_| "{}".to_string());
            let _ = ws_server
                .send_message(conn, return_message.to_string())
                .await;
        }
    });

    (join_handle, abort_handle)
}

async fn send_hello_message(session: &mut actix_ws::Session) {
    let hello_message = OutgoingWebSocketMessage {
        ok: Some(true),
        id: "null".to_string(),
        message: WebSocketMessageType::Hello {
            motd: Box::new(DetailedMotd {
                server_time: "server_time".to_string(),
                motd: "Message of the day".to_string(),
                set: None,
                motd_set: None,
                public_url: "https://kromer.uwu".to_string(),
                public_ws_url: "https://kromer.uwu/krist/ws".to_string(),
                mining_enabled: false,
                transactions_enabled: true,
                debug_mode: true,
                work: 500,
                last_block: None,
                package: PackageInfo {
                    name: "Kromer".to_string(),
                    version: "0.2.0".to_string(),
                    author: "ReconnectedCC Team".to_string(),
                    license: "GPL-3.0".to_string(),
                    repository: "https://github.com/ReconnectedCC/kromer/".to_string(),
                },
                constants: Constants {
                    wallet_version: 3,
                    nonce_max_size: 500,
                    name_cost: 500,
                    min_work: 50,
                    max_work: 500,
                    work_factor: 500.0,
                    seconds_per_block: 5000,
                },
                currency: CurrencyInfo {
                    address_prefix: "k".to_string(),
                    name_suffix: "kro".to_string(),
                    currency_name: "Kromer".to_string(),
                    currency_symbol: "Ϗ".to_string(),
                },
                notice: "Some awesome notice will go here".to_string(),
            }),
        },
    };

    let _ = session
        .text(serde_json::to_string(&hello_message).unwrap_or("{}".to_string()))
        .await;
}

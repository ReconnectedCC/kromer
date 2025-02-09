use std::sync::Arc;

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
        },
        transactions::make_transaction,
    },
};

use super::{routes::auth::perform_login, types::convert_to_iso_string, wrapped_ws::WrappedWsData};

use crate::websockets::routes::me::get_me as route_get_me;

use chrono::Utc;
use surrealdb::{engine::any::Any, Surreal};
use tokio::sync::Mutex;

pub async fn process_text_msg(
    db: &Surreal<Any>,
    ws_metadata: Arc<Mutex<WrappedWsData>>,
    session: &mut actix_ws::Session,
    text: &str,
) -> Result<Option<WrappedWsData>, KromerError> {
    // strip leading and trailing whitespace (spaces, newlines, etc.)
    let msg = text.trim();

    // TODO: potentially change how this serialization is handled, so that we can properly extract "Invalid Parameter" errors.
    let parsed_msg_result: Result<IncomingWebsocketMessage, serde_json::Error> =
        serde_json::from_str(msg);
    let ws_metadata = ws_metadata.lock().await;

    let parsed_msg = match parsed_msg_result {
        Ok(value) => value,
        Err(err) => {
            tracing::error!("Serde error: {}", err);
            tracing::info!("Could not parse JSON for UUID: {}", ws_metadata.token);
            return Err(KromerError::WebSocket(WebSocketError::JsonParseRead));
        }
    };

    let msg_type = parsed_msg.message_type;
    tracing::debug!("Message type was: {:?}", msg_type);
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
            let auth_result = perform_login(&ws_metadata, login_details, db).await;

            // Generate the response if it's okay
            if auth_result.is_ok() {
                let new_auth_data = auth_result.unwrap();
                let wrapped_ws_data = new_auth_data.0;
                let wallet = new_auth_data.1;
                let new_ws_modification_data = WsSessionModification {
                    msg_type: Some(OutgoingWebSocketMessage {
                        ok: Some(true),
                        id: Some(msg_id),
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
                let me_data = route_get_me(msg_id, db, &ws_metadata).await;
                if me_data.is_ok() {
                    ws_modification_data = WsSessionModification {
                        msg_type: Some(me_data.unwrap()),
                        wrapped_ws_data: None,
                    }
                }
            }
        }
        WebSocketMessageType::Logout => {
            let auth_result = perform_logout(&ws_metadata).await;

            let new_ws_modification_data = WsSessionModification {
                msg_type: Some(OutgoingWebSocketMessage {
                    ok: Some(true),
                    id: Some(msg_id),
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
            request_id,
        } => {
            ws_modification_data =
                make_transaction(db, msg_id, private_key, to, amount, metadata, request_id).await;
        }

        WebSocketMessageType::Subscribe { event } => {
            ws_modification_data = subscribe(&ws_metadata, msg_id, event);
        }

        WebSocketMessageType::Unsubscribe { event } => {
            ws_modification_data = unsubscribe(&ws_metadata, msg_id, event)
        }

        WebSocketMessageType::GetSubscriptionLevel => {
            ws_modification_data = get_subscription_level(&ws_metadata, msg_id);
        }

        WebSocketMessageType::GetValidSubscriptionLevels => {
            ws_modification_data = get_valid_subscription_levels(msg_id);
        }

        // Mining will be perma-disabled
        WebSocketMessageType::SubmitBlock => {
            ws_modification_data = WsSessionModification {
                msg_type: Some(OutgoingWebSocketMessage {
                    ok: Some(false),
                    id: Some(msg_id),
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
            let me_data = route_get_me(msg_id, db, &ws_metadata).await;
            if me_data.is_ok() {
                ws_modification_data = WsSessionModification {
                    msg_type: Some(me_data.unwrap()),
                    wrapped_ws_data: None,
                }
            }
        }

        // TODO: Gotta split up the WebSocketMessageType so the regular response types don't error out here when nothing provided, small fish though
        _ => {
            // TODO: Maybe verify this against Krist messages?
            // We should tell the user there was a syntax error with the type in their message.

            // TODO: Maybe make all of the fields on the incoming message types options so we can properly capture if they missed a field
            ws_modification_data = WsSessionModification {
                msg_type: Some(OutgoingWebSocketMessage {
                    ok: Some(false),
                    id: Some(msg_id),
                    message: WebSocketMessageType::Error {
                        error: ErrorResponse {
                            error: "invalid_message_type".to_string(),
                            message: Some("Invalid message type".to_string()),
                        },
                    },
                }),
                wrapped_ws_data: None,
            }
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

pub async fn send_hello_message(session: &mut actix_ws::Session) {
    let cur_time = convert_to_iso_string(Utc::now());

    let hello_message = OutgoingWebSocketMessage {
        ok: Some(true),
        id: None,
        message: WebSocketMessageType::Hello {
            motd: Box::new(DetailedMotd {
                server_time: cur_time,
                motd: "Message of the day".to_string(),
                set: None,
                motd_set: None,
                public_url: "http://kromer.reconnected.cc".to_string(),
                public_ws_url: "http://kromer.reconnected.cc/api/krist/ws".to_string(),
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
                    currency_symbol: "KRO".to_string(),
                },
                notice: "Some awesome notice will go here".to_string(),
            }),
        },
    };

    let _ = session
        .text(serde_json::to_string(&hello_message).unwrap_or("{}".to_string()))
        .await;
}

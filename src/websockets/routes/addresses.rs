use surrealdb::{engine::any::Any, Surreal};

use crate::models::{
    addresses::AddressJson,
    error::ErrorResponse,
    websockets::{
        OutgoingWebSocketMessage, ResponseMessageType, WebSocketMessageType, WsSessionModification,
    },
};

use crate::database::models::wallet::Model as Wallet;

pub async fn get_address(
    address: Option<String>,
    _fetch_names: Option<bool>,
    msg_id: String,
    db: &Surreal<Any>,
) -> WsSessionModification {
    let outgoing_message: OutgoingWebSocketMessage;
    if address.is_none() {
        outgoing_message = format_missing_parameter(msg_id);
    } else {
        let address = address.unwrap();
        // We checked if address was none, so unwrap is okay here
        let wallet = Wallet::get_by_address_excl(db, address.clone()).await;

        if wallet.is_ok() {
            // Unwrap should be alright here since we checked
            if let Some(wallet) = wallet.unwrap() {
                outgoing_message = OutgoingWebSocketMessage {
                    ok: Some(true),
                    id: Some(msg_id),
                    message: WebSocketMessageType::Response {
                        message: ResponseMessageType::Address {
                            address: AddressJson::from(wallet),
                        },
                    },
                }
            } else {
                outgoing_message = format_not_found_error(address, msg_id);
            }
        } else {
            outgoing_message = format_not_found_error(address, msg_id)
        }
    }

    WsSessionModification {
        msg_type: Some(outgoing_message),
        wrapped_ws_data: None,
    }
}

fn format_missing_parameter(msg_id: String) -> OutgoingWebSocketMessage {
    OutgoingWebSocketMessage {
        ok: Some(false),
        id: Some(msg_id),
        message: WebSocketMessageType::Error {
            error: ErrorResponse {
                error: "missing_parameter".to_string(),
                message: Some("Missing parameter address".to_string()),
            },
        },
    }
}

fn format_not_found_error(address: String, msg_id: String) -> OutgoingWebSocketMessage {
    OutgoingWebSocketMessage {
        ok: Some(false),
        id: Some(msg_id),
        message: WebSocketMessageType::Error {
            error: ErrorResponse {
                error: "address_not_found".to_string(),
                message: Some(format!("Address {} not found", address).to_string()),
            },
        },
    }
}

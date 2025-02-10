use std::sync::Arc;

use chrono::Utc;
use surrealdb::{engine::any::Any, Surreal, Uuid};
use tokio::sync::Mutex;

use super::{types::convert_to_iso_string, WebSocketServer};
use crate::{
    errors::{websocket::WebSocketError, KromerError},
    models::{
        error::ErrorResponse,
        motd::{Constants, CurrencyInfo, DetailedMotd, PackageInfo},
        websockets::{WebSocketMessage, WebSocketMessageInner, WebSocketMessageResponse},
    },
    websockets::routes,
};

pub async fn process_text_msg(
    db: &Surreal<Any>,
    server: &WebSocketServer,
    uuid: &Uuid,
    text: &str,
) -> Result<WebSocketMessage, KromerError> {
    // strip leading and trailing whitespace (spaces, newlines, etc.)
    let msg = text.trim();

    // TODO: potentially change how this serialization is handled, so that we can properly extract "Invalid Parameter" errors.
    let parsed_msg_result: Result<WebSocketMessage, serde_json::Error> = serde_json::from_str(msg);

    let parsed_msg = match parsed_msg_result {
        Ok(value) => value,
        Err(err) => {
            tracing::error!("Serde error: {}", err);
            tracing::info!("Could not parse JSON for session {uuid}");
            return Err(KromerError::WebSocket(WebSocketError::JsonParseRead));
        }
    };

    let msg_type = parsed_msg.r#type;
    tracing::debug!("Message type was: {:?}", msg_type);
    let msg_id = parsed_msg.id; // NOTE: This is probably gonna error, lol

    let msg: WebSocketMessage = match msg_type {
        // WebSocketMessageInner::Error { error } => Ok(()), // Not sent by client
        WebSocketMessageInner::Address {
            address,
            fetch_names,
        } => todo!(),
        WebSocketMessageInner::Login { private_key } => {
            routes::auth::perform_login(db, msg_id, private_key).await
        }
        WebSocketMessageInner::Logout => routes::auth::perform_logout(server, uuid, msg_id).await,
        WebSocketMessageInner::Me => todo!(),
        // WebSocketMessageInner::SubmitBlock => todo!(),
        WebSocketMessageInner::Subscribe { event } => {
            routes::subscriptions::subscribe(server, uuid, event, msg_id).await
        }
        WebSocketMessageInner::GetSubscriptionLevel => {
            routes::subscriptions::get_subscription_level(server, uuid, msg_id).await
        }
        WebSocketMessageInner::GetValidSubscriptionLevels => {
            routes::subscriptions::get_valid_subscription_levels(msg_id).await
        }
        WebSocketMessageInner::Unsubscribe { event } => {
            routes::subscriptions::unsubscribe(server, uuid, event, msg_id).await
        }
        WebSocketMessageInner::MakeTransaction {
            private_key,
            to,
            amount,
            metadata,
        } => todo!(),
        WebSocketMessageInner::Work => todo!(),
        _ => todo!(), // Responses not sent by client or unimplemented
    };

    Ok(msg)
}

pub async fn send_hello_message(session: &mut actix_ws::Session) {
    let cur_time = convert_to_iso_string(Utc::now());

    let hello_message = WebSocketMessage {
        ok: Some(true),
        id: None,
        r#type: WebSocketMessageInner::Hello {
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

use std::sync::Arc;

use surrealdb::{engine::any::Any, Surreal};

use crate::{
    database::models::wallet::Model as Wallet,
    errors::{wallet::WalletError, KromerError},
    models::{
        addresses::AddressJson,
        websockets::{
            OutgoingWebSocketMessage, OutgoingWebSocketMessageType, WebSocketMessageType,
        },
    },
    websockets::wrapped_ws::WrappedWsData,
};

pub async fn get_me(
    msg_id: String,
    db: &Arc<Surreal<Any>>,
    ws_metadata: &WrappedWsData,
) -> Result<OutgoingWebSocketMessage, KromerError> {
    if ws_metadata.is_guest() {
        let me_message = OutgoingWebSocketMessageType::Me {
            is_guest: true,
            address: None,
        };
        return Ok(OutgoingWebSocketMessage {
            ok: Some(true),
            id: msg_id,
            message: WebSocketMessageType::Response {
                message: me_message,
            },
        });
    }

    let wallet = Wallet::get_by_address_excl(db, ws_metadata.address.clone()).await?;

    let wallet = match wallet {
        Some(wallet) => Ok(wallet),
        None => Err(KromerError::Wallet(WalletError::NotFound)),
    }?;

    let address_json: AddressJson = wallet.into();

    let me_message = OutgoingWebSocketMessageType::Me {
        is_guest: false,
        address: Some(address_json),
    };

    Ok(OutgoingWebSocketMessage {
        ok: Some(true),
        id: msg_id,
        message: WebSocketMessageType::Response {
            message: me_message,
        },
    })
}

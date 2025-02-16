use surrealdb::{engine::any::Any, Surreal, Uuid};

use crate::{
    database::models::wallet::Model as Wallet,
    models::{
        addresses::AddressJson,
        websockets::{WebSocketMessage, WebSocketMessageInner, WebSocketMessageResponse},
    },
    websockets::WebSocketServer,
};

pub async fn get_myself(
    db: &Surreal<Any>,
    server: &WebSocketServer,
    uuid: &Uuid,
    msg_id: Option<usize>,
) -> WebSocketMessage {
    let inner = server.inner.lock().await;
    let entry = inner
        .sessions
        .get(uuid)
        .expect("Expected session to exist, somehow it does not");

    let session_data = entry.value();
    if session_data.is_guest() {
        return WebSocketMessage {
            ok: Some(true),
            id: msg_id,
            r#type: WebSocketMessageInner::Response {
                responding_to: "me".to_owned(),
                data: WebSocketMessageResponse::Me {
                    is_guest: true,
                    address: None,
                },
            },
        };
    }

    let wallet = Wallet::get_by_address_excl(db, session_data.address.clone()).await;
    if wallet.is_err() {
        let err = wallet.err().unwrap(); // SAFETY: We made sure it's an error
        tracing::error!("Caught an error: {err}");

        return WebSocketMessage {
            ok: Some(false),
            id: msg_id,
            r#type: WebSocketMessageInner::Error {
                error: "internal_server_error".to_owned(),
                message: "Something went wrong while processing your message".to_owned(),
            },
        };
    }

    let wallet = wallet.unwrap(); // SAFETY: We made sure the database did not error.
    let wallet = match wallet {
        Some(wallet) => wallet,
        None => {
            return WebSocketMessage {
                ok: Some(false),
                id: msg_id,
                r#type: WebSocketMessageInner::Error {
                    error: "address_not_found".to_owned(),
                    message: format!("Address {} not found", session_data.address),
                },
            }
        }
    };

    let wallet_resp: AddressJson = wallet.into();

    WebSocketMessage {
        ok: Some(true),
        id: msg_id,
        r#type: WebSocketMessageInner::Response {
            responding_to: "me".to_owned(),
            data: WebSocketMessageResponse::Me {
                is_guest: false,
                address: Some(wallet_resp),
            },
        },
    }
}

// pub async fn get_me(
//     msg_id: String,
//     db: &Surreal<Any>,
//     ws_metadata: &WrappedWsData,
// ) -> Result<OutgoingWebSocketMessage, KromerError> {
//     if ws_metadata.is_guest() {
//         let me_message = ResponseMessageType::Me {
//             is_guest: true,
//             address: None,
//         };
//         return Ok(OutgoingWebSocketMessage {
//             ok: Some(true),
//             id: Some(msg_id),
//             message: WebSocketMessageType::Response {
//                 message: me_message,
//             },
//         });
//     }

//     let wallet = Wallet::get_by_address_excl(db, ws_metadata.address.clone()).await?;

//     let wallet = match wallet {
//         Some(wallet) => Ok(wallet),
//         None => Err(KromerError::Wallet(WalletError::NotFound)),
//     }?;

//     let address_json: AddressJson = wallet.into();

//     let me_message = ResponseMessageType::Me {
//         is_guest: false,
//         address: Some(address_json),
//     };

//     Ok(OutgoingWebSocketMessage {
//         ok: Some(true),
//         id: Some(msg_id),
//         message: WebSocketMessageType::Response {
//             message: me_message,
//         },
//     })
// }

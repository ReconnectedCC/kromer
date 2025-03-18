use surrealdb::Surreal;
use surrealdb::{engine::any::Any, Uuid};

use crate::database::models::wallet::Model as Wallet;
use crate::errors::wallet::WalletError;
use crate::errors::KromerError;
use crate::models::websockets::{
    WebSocketMessage, WebSocketMessageInner, WebSocketMessageResponse,
};
use crate::websockets::WebSocketServer;

pub async fn perform_login(
    db: &Surreal<Any>,
    server: &WebSocketServer,
    uuid: &Uuid,
    private_key: String,
    msg_id: Option<usize>,
) -> WebSocketMessage {
    let wallet = Wallet::verify_address(db, private_key.clone())
        .await
        .map_err(|_| KromerError::Wallet(WalletError::InvalidPassword));

    // TODO: Refactor this fuckass match statement so we dont have a billion nested structs, lol
    match wallet {
        Ok(response) => {
            if response.authed {
                let wallet = response.address;

                let inner = server.inner.lock().await;
                let mut session = inner
                    .sessions
                    .get_mut(uuid)
                    .expect("Expected the session to exist, why doesn't it?");
                session.address = wallet.address.clone();
                session.private_key = Some(private_key);

                WebSocketMessage {
                    ok: Some(true),
                    id: msg_id,
                    r#type: WebSocketMessageInner::Response {
                        responding_to: "login".to_owned(),
                        data: WebSocketMessageResponse::Login {
                            is_guest: false,
                            address: Some(wallet.into()),
                        },
                    },
                }
            } else {
                WebSocketMessage {
                    ok: Some(true),
                    id: msg_id,
                    r#type: WebSocketMessageInner::Response {
                        responding_to: "login".to_owned(),
                        data: WebSocketMessageResponse::Login {
                            is_guest: true,
                            address: None,
                        },
                    },
                }
            }
        }
        Err(_) => WebSocketMessage {
            ok: Some(true),
            id: msg_id,
            r#type: WebSocketMessageInner::Response {
                responding_to: "login".to_owned(),
                data: WebSocketMessageResponse::Login {
                    is_guest: true,
                    address: None,
                },
            },
        },
    }
}

pub async fn perform_logout(
    server: &WebSocketServer,
    uuid: &Uuid,
    msg_id: Option<usize>,
) -> WebSocketMessage {
    let inner = server.inner.lock().await; // seems pretty silly but i dont wanna mess with lifetimes

    let mut session = inner
        .sessions
        .get_mut(uuid)
        .expect("Expected the session to exist, why doesn't it?");
    session.address = String::from("guest");
    session.private_key = None;

    WebSocketMessage {
        ok: Some(true),
        id: msg_id,
        r#type: WebSocketMessageInner::Response {
            responding_to: "logout".to_owned(),
            data: WebSocketMessageResponse::Logout { is_guest: true },
        },
    }
}

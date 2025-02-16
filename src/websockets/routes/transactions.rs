use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use surrealdb::{engine::any::Any, Surreal};

use crate::{
    database::models::transaction::TransactionCreateData,
    models::{
        transactions::TransactionType,
        websockets::{WebSocketMessage, WebSocketMessageInner, WebSocketMessageResponse},
    },
};

use crate::database::models::transaction::Model as Transaction;
use crate::database::models::wallet::Model as Wallet;

pub async fn make_transaction(
    db: &Surreal<Any>,
    private_key: String,
    to: String,
    amount: Decimal,
    metadata: Option<String>,
    msg_id: Option<usize>,
) -> WebSocketMessage {
    if amount < dec!(0.0) {
        return WebSocketMessage {
            ok: Some(false),
            id: msg_id,
            r#type: WebSocketMessageInner::Error {
                error: "invalid_parameter".to_owned(),
                message: "Invalid parameter amount".to_owned(),
            },
        };
    }

    let resp = match Wallet::verify_address(db, private_key).await {
        Ok(resp) => resp,
        Err(_) => {
            return WebSocketMessage {
                ok: Some(false),
                id: msg_id,
                r#type: WebSocketMessageInner::Error {
                    error: "database_error".to_owned(),
                    message: "An error occured in the database".to_owned(),
                },
            }
        }
    };
    if !resp.authed {
        return WebSocketMessage {
            ok: Some(false),
            id: msg_id,
            r#type: WebSocketMessageInner::Error {
                error: "invalid_parameter".to_owned(),
                message: "Invalid parameter privatekey".to_owned(),
            },
        };
    }

    let sender = resp.address;

    let recipient = match Wallet::get_by_address(db, to.clone()).await {
        Ok(model) => model,
        Err(_) => {
            return WebSocketMessage {
                ok: Some(false),
                id: msg_id,
                r#type: WebSocketMessageInner::Error {
                    error: "database_error".to_owned(),
                    message: "An error occured in the database".to_owned(),
                },
            }
        }
    };

    let recipient = match recipient {
        Some(wallet) => wallet,
        None => {
            return WebSocketMessage {
                ok: Some(false),
                id: msg_id,
                r#type: WebSocketMessageInner::Error {
                    error: "address_not_found".to_owned(),
                    message: format!("Address {} not found", to),
                },
            }
        }
    };

    if sender.balance < amount {
        return WebSocketMessage {
            ok: Some(false),
            id: msg_id,
            r#type: WebSocketMessageInner::Error {
                error: "insufficient_funds".to_owned(),
                message: "Insufficient funds".to_owned(),
            },
        };
    }

    let creation_data = TransactionCreateData {
        from: sender.address.clone(),
        to: recipient.address.clone(),
        amount,
        metadata: metadata.clone(),
        transaction_type: TransactionType::Transfer,
    };

    let response: Result<Vec<Transaction>, _> =
        db.insert("transaction").content(creation_data).await;
    if response.is_err() {
        return WebSocketMessage {
            ok: Some(false),
            id: msg_id,
            r#type: WebSocketMessageInner::Error {
                error: "database_error".to_owned(),
                message: "An error occured in the database".to_owned(),
            },
        };
    }
    let response = response.unwrap();
    let first = response.first().unwrap();
    let transaction = first.clone(); // guh

    WebSocketMessage {
        ok: Some(true),
        id: msg_id,
        r#type: WebSocketMessageInner::Response {
            responding_to: "make_transaction".to_owned(),
            data: WebSocketMessageResponse::MakeTransaction {
                transaction: transaction.into(),
            },
        },
    }
}

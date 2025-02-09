use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use surrealdb::{engine::any::Any, Surreal};

use crate::{
    database::models::transaction::TransactionCreateData,
    models::{
        error::ErrorResponse,
        transactions::TransactionType,
        websockets::{OutgoingWebSocketMessage, ResponseMessageType, WebSocketMessageType},
        websockets::{WebSocketMessage, WebSocketMessageInner, WebSocketMessageResponse},
    },
};

use crate::database::models::transaction::Model as Transaction;
use crate::database::models::wallet::Model as Wallet;

// pub async fn make_transaction(
//     db: &Surreal<Any>,
//     msg_id: String,
//     private_key: Option<String>,
//     to: Option<String>,
//     amount: Option<Decimal>,
//     metadata: Option<String>,
//     _request_id: Option<String>,
// ) -> WsSessionModification {
//     let mut outgoing_message: OutgoingWebSocketMessage = OutgoingWebSocketMessage {
//         ok: Some(false),
//         id: Some(msg_id.clone()),
//         message: WebSocketMessageType::Response {
//             message: ResponseMessageType::Me {
//                 is_guest: true,
//                 address: None,
//             },
//         },
//     };

//     if amount.is_some() && private_key.is_some() && to.is_some() {
//         // Unwrap is fine because we checked it.
//         let amount = amount.unwrap();
//         let private_key = private_key.unwrap();
//         let to = to.unwrap();

//         // Check on the server so DB doesnt throw.
//         if amount < dec!(0.0) {
//             outgoing_message = format_invalid_parameter(msg_id.clone(), "amount".to_string());
//         } else if let Ok(verify_response) = Wallet::verify_address(db, private_key).await {
//             let sender = verify_response.address;
//             if let Ok(Some(recipient)) = Wallet::get_by_address(db, to.to_string()).await {
//                 // Make sure to check the request to see if the funds are available.
//                 if sender.balance < amount {
//                     outgoing_message = format_insufficient_funds_error(msg_id.clone())
//                 } else {
//                     // Create the data
//                     let creation_data = TransactionCreateData {
//                         from: sender.address.clone(),
//                         to: recipient.address.clone(),
//                         amount,
//                         metadata: metadata.clone(),
//                         transaction_type: TransactionType::Transfer,
//                     };
//                     let response = db.insert("transaction").content(creation_data).await;

//                     if response.is_ok() {
//                         let _response: Vec<Transaction> = response.unwrap();

//                         let time = chrono::Utc::now().to_rfc3339();
//                         outgoing_message = OutgoingWebSocketMessage {
//                             ok: Some(true),
//                             id: Some(msg_id),
//                             message: WebSocketMessageType::Response {
//                                 message: ResponseMessageType::MakeTransaction {
//                                     from: sender.address,
//                                     to: recipient.address,
//                                     value: amount,
//                                     time,
//                                     name: None,
//                                     metadata,
//                                     sent_metaname: None,
//                                     sent_name: None,
//                                     transaction_type: "transfer".to_string(),
//                                 },
//                             },
//                         }
//                     } else {
//                         outgoing_message = format_database_error(msg_id);
//                     }
//                 }
//             } else {
//                 outgoing_message = format_not_found_error(msg_id, to);
//             }
//         } else {
//             outgoing_message = format_invalid_parameter(msg_id, "privatekey".to_string())
//         }
//     } else if amount.is_none() {
//         outgoing_message = format_missing_parameter(msg_id, "amount".to_string());
//     } else if private_key.is_none() {
//         outgoing_message = format_missing_parameter(msg_id, "privatekey".to_string());
//     } else if to.is_none() {
//         outgoing_message = format_missing_parameter(msg_id, "to".to_string())
//     }
//     WsSessionModification {
//         msg_type: Some(outgoing_message),
//         wrapped_ws_data: None,
//     }
// }

// fn format_invalid_parameter(msg_id: String, parameter: String) -> OutgoingWebSocketMessage {
//     OutgoingWebSocketMessage {
//         ok: Some(false),
//         id: Some(msg_id),
//         message: WebSocketMessageType::Error {
//             error: ErrorResponse {
//                 error: "invalid_parameter".to_string(),
//                 message: Some(format!("Invalid parameter {}", parameter).to_string()),
//             },
//         },
//     }
// }

// fn format_missing_parameter(msg_id: String, parameter: String) -> OutgoingWebSocketMessage {
//     OutgoingWebSocketMessage {
//         ok: Some(false),
//         id: Some(msg_id),
//         message: WebSocketMessageType::Error {
//             error: ErrorResponse {
//                 error: "missing_parameter".to_string(),
//                 message: Some(format!("Missing parameter {}", parameter).to_string()),
//             },
//         },
//     }
// }

// fn format_not_found_error(msg_id: String, address: String) -> OutgoingWebSocketMessage {
//     OutgoingWebSocketMessage {
//         ok: Some(false),
//         id: Some(msg_id),
//         message: WebSocketMessageType::Error {
//             error: ErrorResponse {
//                 error: "address_not_found".to_string(),
//                 message: Some(format!("Address {} not found", address).to_string()),
//             },
//         },
//     }
// }

// fn format_insufficient_funds_error(msg_id: String) -> OutgoingWebSocketMessage {
//     OutgoingWebSocketMessage {
//         ok: Some(false),
//         id: Some(msg_id),
//         message: WebSocketMessageType::Error {
//             error: ErrorResponse {
//                 error: "insufficient_funds".to_string(),
//                 message: Some("Insufficient funds".to_string()),
//             },
//         },
//     }
// }

// fn format_database_error(msg_id: String) -> OutgoingWebSocketMessage {
//     OutgoingWebSocketMessage {
//         ok: Some(false),
//         id: Some(msg_id),
//         message: WebSocketMessageType::Error {
//             error: ErrorResponse {
//                 error: "database_error".to_string(),
//                 message: Some("Database error".to_string()),
//             },
//         },
//     }
// }

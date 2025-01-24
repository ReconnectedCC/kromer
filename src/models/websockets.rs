use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::websockets::types::common::WebSocketSubscriptionType;
use crate::websockets::wrapped_ws::WrappedWsData;

use super::deserialize_number_into_string;
use super::{addresses::AddressJson, auth::LoginDetails};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebSocketSubscription {
    Blocks,
    OwnBlocks,
    Transactions,
    OwnTransactions,
    Names,
    OwnNames,
    Motd,
}

pub struct WsSessionModification {
    pub msg_type: Option<OutgoingWebSocketMessage>,
    pub wrapped_ws_data: Option<WrappedWsData>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum WebSocketMessageType {
    Hello {
        #[serde(flatten)]
        motd: Box<super::motd::DetailedMotd>,
    },
    Error {
        #[serde(flatten)]
        error: super::error::ErrorResponse,
    },
    Response {
        #[serde(flatten)]
        message: ResponseMessageType,
    },
    Keepalive {
        server_time: String,
    },

    // 100% these are missing a lot
    Address {
        address: Option<String>,
        #[serde(rename = "fetchNames")]
        fetch_names: Option<bool>,
    },
    Login {
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        login_details: Option<LoginDetails>,
    },
    Logout,
    Me,
    SubmitBlock,
    Subscribe {
        event: WebSocketSubscriptionType,
    },
    GetSubscriptionLevel,
    GetValidSubscriptionLevels,
    Unsubscribe {
        event: WebSocketSubscriptionType,
    },

    // TODO: These are currently options so that we can catch if serialization does not capture a field
    // This should possibly be handled by a custom serde_json::from_str() implementation that can parse values and error properly.
    MakeTransaction {
        #[serde(rename = "privatekey")]
        private_key: Option<String>,
        to: Option<String>,
        amount: Option<Decimal>,
        metadata: Option<String>,
        #[serde(rename = "requestId")]
        request_id: Option<String>,
    },
    Work,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "responding_to_type")]
pub enum ResponseMessageType {
    Address {
        address: AddressJson,
    },
    Login {
        #[serde(rename = "isGuest")]
        is_guest: bool,
        address: Option<AddressJson>,
    },
    Logout {
        #[serde(rename = "isGuest")]
        is_guest: bool,
    },
    Me {
        #[serde(rename = "isGuest")]
        is_guest: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        address: Option<AddressJson>,
    },
    SubmitBlock,
    Subscribe {
        subscription_level: Vec<String>,
    },
    GetSubscriptionLevel {
        subscription_level: Vec<String>,
    },
    GetValidSubscriptionLevels {
        valid_subscription_levels: Vec<String>,
    },
    Unsubcribe {
        subscription_level: Vec<String>,
    },
    MakeTransaction {
        from: String,
        to: String,
        value: Decimal,
        time: String,
        name: Option<String>,
        metadata: Option<String>,
        sent_metaname: Option<String>,
        sent_name: Option<String>,
        #[serde(rename = "type")]
        transaction_type: String,
    },
    Work,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "event")]
pub enum WebSocketEventType {
    Block {
        block: super::blocks::BlockJson,
        new_work: i64,
    },
    Transaction {
        transaction: super::transactions::TransactionJson,
    },
    Name {
        name: super::names::NameJson,
    },
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct WebSocketTokenData {
    pub address: String,
    #[serde(rename = "privatekey")]
    pub private_key: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct IncomingWebsocketMessage {
    #[serde(deserialize_with = "deserialize_number_into_string")]
    pub id: String,
    #[serde(flatten, rename = "type")]
    pub message_type: WebSocketMessageType,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OutgoingWebSocketMessage {
    pub ok: Option<bool>,
    pub id: String,
    #[serde(flatten)]
    pub message: WebSocketMessageType,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct WebSocketEventMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(flatten)]
    pub event: WebSocketEventType,
}

#[cfg(test)]
mod tests {
    use super::{OutgoingWebSocketMessage, WebSocketMessageType};

    #[test]
    fn test_hello_type() {
        let raw = r#"{"ok":true,"type":"hello","server_time":"2025-01-08T19:18:26.589Z","motd":"The API URL has changed to https://krist.dev\n\nBlock submission is disabled ([more info](https://discord.sc3.io))","set":"2023-03-22T21:14:06.000Z","motd_set":"2023-03-22T21:14:06.000Z","public_url":"krist.dev","public_ws_url":"ws.krist.dev","mining_enabled":false,"transactions_enabled":true,"debug_mode":false,"work":575,"last_block":{"height":2121616,"address":"kristdeath","hash":"00000000009f16ac5ded918793310016ea2d61a29d5a328e244cd8478da6924c","short_hash":"00000000009f","value":1,"time":"2022-07-19T20:43:09.000Z","difficulty":551},"package":{"name":"krist","version":"3.5.2","author":"Lemmmy","licence":"GPL-3.0","repository":"https://github.com/tmpim/Krist"},"constants":{"wallet_version":16,"nonce_max_size":24,"name_cost":500,"min_work":1,"max_work":100000,"work_factor":0.025,"seconds_per_block":300},"currency":{"address_prefix":"k","name_suffix":"kst","currency_name":"Krist","currency_symbol":"KST"},"notice":"Krist was originally created by 3d6 and Lemmmy. It is now owned and operated by tmpim, and licensed under GPL-3.0."}"#;
        let msg: OutgoingWebSocketMessage =
            serde_json::from_str(raw).expect("failed to deserialize outgoing websocket message");
        assert_eq!(msg.ok, Some(true));
        assert_eq!(msg.message.member_str(), "hello");

        match msg.message {
            WebSocketMessageType::Hello { motd: _ } => {
                // TODO: Checks for motd
            }
            _ => panic!("Invalid message type"),
        }
    }

    #[test]
    fn test_keepalive_type() {
        let raw = r#"{"type":"keepalive","server_time":"2025-01-08T19:18:26.596Z"}"#;
        let msg: OutgoingWebSocketMessage =
            serde_json::from_str(raw).expect("failed to deserialize outgoing websocket message");
        assert_eq!(msg.ok, None);
        assert_eq!(msg.message.member_str(), "keepalive");
    }
}

impl WebSocketMessageType {
    /// Return the enum member name as a str
    pub fn member_str(&self) -> &'static str {
        match self {
            WebSocketMessageType::Address { .. } => "address",
            WebSocketMessageType::Login { .. } => "login",
            WebSocketMessageType::Logout => "logout",
            WebSocketMessageType::Me => "me",
            WebSocketMessageType::SubmitBlock => "submit_block",
            WebSocketMessageType::Subscribe { .. } => "subscribe",
            WebSocketMessageType::GetSubscriptionLevel => "get_subscription_level",
            WebSocketMessageType::GetValidSubscriptionLevels => "get_valid_subscription_levels",
            WebSocketMessageType::Unsubscribe { .. } => "unsubscribe",
            WebSocketMessageType::MakeTransaction { .. } => "make_transaction",
            WebSocketMessageType::Work => "work",
            WebSocketMessageType::Hello { .. } => "hello",
            WebSocketMessageType::Error { .. } => "error",
            WebSocketMessageType::Response { .. } => "response",
            WebSocketMessageType::Keepalive { .. } => "keepalive",
            WebSocketMessageType::Unknown => "unknown",
        }
    }
}

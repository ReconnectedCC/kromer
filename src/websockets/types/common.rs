use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct WebSocketTokenData {
    pub address: String,
    pub private_key: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct WebSocketSubscriptionList {
    #[serde(flatten)]
    pub subscriptions: Vec<WebSocketSubscriptionType>,
}

impl WebSocketTokenData {
    #[inline]
    pub fn new(address: String, private_key: Option<String>) -> Self {
        Self {
            address,
            private_key,
        }
    }
}

impl WebSocketSubscriptionList {
    pub fn new() -> Self {
        WebSocketSubscriptionList {
            subscriptions: vec![
                WebSocketSubscriptionType::OwnTransactions,
                WebSocketSubscriptionType::Blocks,
            ],
        }
    }

    pub fn new_all_subs() -> Self {
        WebSocketSubscriptionList {
            subscriptions: vec![
                WebSocketSubscriptionType::Blocks,
                WebSocketSubscriptionType::OwnBlocks,
                WebSocketSubscriptionType::Transactions,
                WebSocketSubscriptionType::OwnTransactions,
                WebSocketSubscriptionType::Names,
                WebSocketSubscriptionType::OwnNames,
                WebSocketSubscriptionType::Motd,
            ],
        }
    }

    pub fn to_string(&self) -> Vec<String> {
        let subscriptions = &self.subscriptions;

        let mut sub_strings: Vec<String> = Vec::new();

        for subscription in subscriptions {
            sub_strings.push(format!("{}", subscription));
        }

        sub_strings
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum WebSocketSubscriptionType {
    Blocks,
    OwnBlocks,
    Transactions,
    OwnTransactions,
    Names,
    OwnNames,
    Motd,
}

impl FromStr for WebSocketSubscriptionType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "blocks" => Ok(Self::Blocks),
            "ownBlocks" => Ok(Self::OwnBlocks),
            "transactions" => Ok(Self::Transactions),
            "ownTransactions" => Ok(Self::OwnTransactions),
            "names" => Ok(Self::Names),
            "ownNames" => Ok(Self::OwnNames),
            "motd" => Ok(Self::Motd),
            _ => Err(()),
        }
    }
}

impl fmt::Display for WebSocketSubscriptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocks => write!(f, "blocks"),
            Self::OwnBlocks => write!(f, "ownBlocks"),
            Self::Transactions => write!(f, "transactions"),
            Self::OwnTransactions => write!(f, "ownTransactions"),
            Self::Names => write!(f, "names"),
            Self::OwnNames => write!(f, "ownNames"),
            Self::Motd => write!(f, "motd"),
        }
    }
}

// #[derive(Clone, Debug)]
// pub enum WebSocketMessageType {
//     Address,
//     Login,
//     Logout,
//     Me,
//     Subscribe,
//     GetSubscriptionLevel,
//     GetValidSubscriptionLevels,
//     Unsubscribe,
//     MakeTransaction,
//     Motd,
// }

// impl FromStr for WebSocketMessageType {
//     type Err = KromerError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "address" => Ok(Self::Address),
//             "login" => Ok(Self::Login),
//             "logout" => Ok(Self::Logout),
//             "me" => Ok(Self::Me),
//             "subscribe" => Ok(Self::Me),
//             "get_subscription_level" => Ok(Self::GetSubscriptionLevel),
//             "get_valid_subscription_levels" => Ok(Self::GetValidSubscriptionLevels),
//             "unsubscribe" => Ok(Self::Unsubscribe),
//             "make_transaction" => Ok(Self::MakeTransaction),
//             "motd" => Ok(Self::Motd),
//             _ => Err(KromerError::WebSocket(WebSocketError::InvalidMessageType)),
//         }
//     }
// }

// impl fmt::Display for WebSocketMessageType {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Address => write!(f, "address"),
//             Self::Login => write!(f, "login"),
//             Self::Logout => write!(f, "logout"),
//             Self::Me => write!(f, "me"),
//             Self::Subscribe => write!(f, "subscribe"),
//             Self::GetSubscriptionLevel => write!(f, "get_subscription_level"),
//             Self::GetValidSubscriptionLevels => write!(f, "get_valid_subscription_levels"),
//             Self::Unsubscribe => write!(f, "unsubscribe"),
//             Self::MakeTransaction => write!(f, "make_transaction"),
//             Self::Motd => write!(f, "motd"),
//         }
//     }
// }

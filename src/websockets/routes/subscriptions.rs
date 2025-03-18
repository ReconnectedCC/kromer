use std::str::FromStr;

use surrealdb::Uuid;

use crate::{
    models::websockets::{WebSocketMessage, WebSocketMessageInner, WebSocketMessageResponse},
    websockets::{types::common::WebSocketSubscriptionType, WebSocketServer},
};

pub async fn subscribe(
    server: &WebSocketServer,
    uuid: &Uuid,
    event: String,
    msg_id: Option<usize>,
) -> WebSocketMessage {
    if WebSocketSubscriptionType::is_valid(&event) {
        let event = WebSocketSubscriptionType::from_str(&event).unwrap(); // Unwrap should be fine, we made sure it is valid above
        server.subscribe_to_event(uuid, event).await;

        let subscription_list = server.get_subscription_list(uuid).await;
        let subscription_list: Vec<String> = subscription_list
            .into_iter()
            .map(|x| x.into_string())
            .collect();

        let message = WebSocketMessage {
            ok: Some(true),
            id: msg_id,
            r#type: WebSocketMessageInner::Response {
                responding_to: "subscribe".to_owned(),
                data: WebSocketMessageResponse::Subscribe {
                    subscription_level: subscription_list,
                },
            },
        };

        return message;
    }

    WebSocketMessage {
        ok: Some(false),
        id: msg_id,
        r#type: WebSocketMessageInner::Error {
            error: "invalid_parameter".to_owned(),
            message: "Invalid parameter event".to_owned(),
        },
    }
}

pub async fn unsubscribe(
    server: &WebSocketServer,
    uuid: &Uuid,
    event: String,
    msg_id: Option<usize>,
) -> WebSocketMessage {
    if WebSocketSubscriptionType::is_valid(&event) {
        let event = WebSocketSubscriptionType::from_str(&event).unwrap(); // Unwrap should be fine, we made sure it is valid above
        server.unsubscribe_from_event(uuid, &event).await;

        let subscription_list = server.get_subscription_list(uuid).await;
        let subscription_list: Vec<String> = subscription_list
            .into_iter()
            .map(|x| x.into_string())
            .collect();

        let message = WebSocketMessage {
            ok: Some(true),
            id: msg_id,
            r#type: WebSocketMessageInner::Response {
                responding_to: "subscribe".to_owned(),
                data: WebSocketMessageResponse::Subscribe {
                    subscription_level: subscription_list,
                },
            },
        };

        return message;
    }

    WebSocketMessage {
        ok: Some(false),
        id: msg_id,
        r#type: WebSocketMessageInner::Error {
            error: "invalid_parameter".to_owned(),
            message: "Invalid parameter event".to_owned(),
        },
    }
}

pub async fn get_subscription_level(
    server: &WebSocketServer,
    uuid: &Uuid,
    msg_id: Option<usize>,
) -> WebSocketMessage {
    let subscription_list = server.get_subscription_list(uuid).await;
    let subscription_list: Vec<String> = subscription_list
        .into_iter()
        .map(|x| x.into_string())
        .collect();

    WebSocketMessage {
        ok: Some(true),
        id: msg_id,
        r#type: WebSocketMessageInner::Response {
            responding_to: "get_subscription_level".to_owned(),
            data: WebSocketMessageResponse::GetSubscriptionLevel {
                subscription_level: subscription_list,
            },
        },
    }
}

pub async fn get_valid_subscription_levels(msg_id: Option<usize>) -> WebSocketMessage {
    let subscription_list = vec![
        WebSocketSubscriptionType::Blocks,
        WebSocketSubscriptionType::OwnBlocks,
        WebSocketSubscriptionType::Transactions,
        WebSocketSubscriptionType::OwnTransactions,
        WebSocketSubscriptionType::Names,
        WebSocketSubscriptionType::OwnNames,
        WebSocketSubscriptionType::Motd,
    ];
    let subscription_list: Vec<String> = subscription_list
        .into_iter()
        .map(|x| x.into_string())
        .collect();

    WebSocketMessage {
        ok: Some(true),
        id: msg_id,
        r#type: WebSocketMessageInner::Response {
            responding_to: "get_valid_subscription_levels".to_owned(),
            data: WebSocketMessageResponse::GetValidSubscriptionLevels {
                valid_subscription_levels: subscription_list,
            },
        },
    }
}

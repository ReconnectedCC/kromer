use crate::{
    models::websockets::{
        OutgoingWebSocketMessage, ResponseMessageType, WebSocketMessageType, WsSessionModification,
    },
    websockets::types::common::WebSocketSubscriptionType,
};

pub fn get_subscription_level(
    ws_metadata: &WrappedWsData,
    msg_id: String,
) -> WsSessionModification {
    // Just need to pull out the subscription levels from the wrapped data, and display it
    let subscription_levels = &ws_metadata.subs;

    WsSessionModification {
        msg_type: Some(OutgoingWebSocketMessage {
            ok: Some(true),
            id: Some(msg_id),
            message: WebSocketMessageType::Response {
                message: ResponseMessageType::GetSubscriptionLevel {
                    subscription_level: subscription_levels.to_owned().to_string(),
                },
            },
        }),
        wrapped_ws_data: None,
    }
}

pub fn get_valid_subscription_levels(msg_id: String) -> WsSessionModification {
    let valid_subscription_levels = WebSocketSubscriptionList::new_all_subs();

    WsSessionModification {
        msg_type: Some(OutgoingWebSocketMessage {
            ok: Some(true),
            id: Some(msg_id),
            message: WebSocketMessageType::Response {
                message: ResponseMessageType::GetValidSubscriptionLevels {
                    valid_subscription_levels: valid_subscription_levels.to_string(),
                },
            },
        }),
        wrapped_ws_data: None,
    }
}

pub fn subscribe(
    ws_metadata: &WrappedWsData,
    msg_id: String,
    event: WebSocketSubscriptionType,
) -> WsSessionModification {
    let mut cur_subs = ws_metadata.subs.clone();

    if !cur_subs.subscriptions.contains(&event) {
        cur_subs.subscriptions.push(event);
    }

    let new_ws_data = WrappedWsData {
        subs: cur_subs.to_owned(),
        ..ws_metadata.to_owned()
    };

    WsSessionModification {
        msg_type: Some(OutgoingWebSocketMessage {
            ok: Some(true),
            id: Some(msg_id),
            message: WebSocketMessageType::Response {
                message: ResponseMessageType::Subscribe {
                    subscription_level: new_ws_data.subs.to_string(),
                },
            },
        }),
        wrapped_ws_data: Some(new_ws_data),
    }
}

pub fn unsubscribe(
    ws_metadata: &WrappedWsData,
    msg_id: String,
    event: WebSocketSubscriptionType,
) -> WsSessionModification {
    let mut cur_subs = ws_metadata.subs.clone();

    // Remove the sub, yeah kinda weird looking tbh
    if let Some(index) = cur_subs.subscriptions.iter().position(|e| e == &event) {
        cur_subs.subscriptions.remove(index);
    }

    let new_ws_data = WrappedWsData {
        subs: cur_subs.to_owned(),
        ..ws_metadata.to_owned()
    };

    WsSessionModification {
        msg_type: Some(OutgoingWebSocketMessage {
            ok: Some(true),
            id: Some(msg_id),
            message: WebSocketMessageType::Response {
                message: ResponseMessageType::Unsubcribe {
                    subscription_level: new_ws_data.subs.to_string(),
                },
            },
        }),
        wrapped_ws_data: Some(new_ws_data),
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{self, Error as SerdeJsonError, Number, Value};

use crate::errors::{websocket::WebSocketError, KromerError};


#[derive(Debug, Deserialize, Serialize)]
struct AbstractData {
    id: Number,
    msg_type: String,
    #[serde(flatten)] 
    extra: HashMap<String, Value>
}

fn parse_json_string(msg: String ) -> Result<Value, SerdeJsonError> {
    serde_json::from_str(&msg)
}

pub fn handle_ws_msg(msg: String) -> Result<Value, KromerError> {
    //let json_msg = parse_json_string(msg);

    let json_msg = match parse_json_string(msg) {
        Ok(message) => message,
        Err(_) => Err(KromerError::WebSocket(WebSocketError::JsonParseRead))? 
    };
    tracing::debug!("[WS_HANDLER] JSON Parsed as: {:?}",json_msg);

    tracing::debug!("[WS_HANDLER] Processing for Message ID: {}, Type: {}", json_msg["id"].to_string(), json_msg["type"].to_string());
    
    Ok(json_msg)

}
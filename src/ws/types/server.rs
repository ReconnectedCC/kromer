// use actix::{Addr, Recipient};

// use crate::ws::session::WebSocketSession;

// use crate::ws::types::actor_message::KromerMessage;

// type Client = Recipient<KromerMessage>;

// type CachedWeVbSocket = Addr<WebSocketSession>;

use super::session::KromerAddress;

pub struct TokenParams {
    pub address: KromerAddress,
    pub privatekey: String
}

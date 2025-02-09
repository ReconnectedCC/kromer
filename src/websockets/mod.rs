pub mod errors;
pub mod handler;
pub mod routes;
pub mod types;
pub mod utils;

use std::{sync::Arc, time::Duration};

use actix_web::rt::time;
use actix_ws::Session;
use bytestring::ByteString;
use dashmap::{DashMap, DashSet};
use errors::WebSocketServerError;
use futures_util::{stream::FuturesUnordered, StreamExt};
use surrealdb::Uuid;
use tokio::sync::Mutex;

use types::common::{WebSocketSessionData, WebSocketSubscriptionType, WebSocketTokenData};

use crate::models::websockets::WebSocketEventMessage;

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
pub const TOKEN_EXPIRATION: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct WebSocketServer {
    inner: Arc<Mutex<WebSocketServerInner>>,
}

#[derive(Clone)]
pub struct WebSocketServerInner {
    sessions: DashMap<Uuid, WebSocketSessionData>,
    pending_tokens: DashMap<Uuid, WebSocketTokenData>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        let inner = WebSocketServerInner {
            sessions: DashMap::new(),
            pending_tokens: DashMap::new(),
        };

        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub async fn insert_session(&self, uuid: Uuid, session: Session, data: WebSocketTokenData) {
        let subscriptions = DashSet::from_iter(vec![
            WebSocketSubscriptionType::OwnTransactions,
            WebSocketSubscriptionType::Blocks,
        ]);

        let session_data = WebSocketSessionData {
            address: data.address,
            private_key: data.private_key,
            session,
            subscriptions,
        };

        self.inner.lock().await.sessions.insert(uuid, session_data);
    }

    pub async fn cleanup_session(&self, uuid: &Uuid) {
        tracing::info!("Cleaning up session {uuid}");
        self.inner.lock().await.sessions.remove(uuid);
    }

    #[tracing::instrument(skip_all, fields(address = token_data.address))]
    pub async fn obtain_token(&self, token_data: WebSocketTokenData) -> Uuid {
        let inner = self.inner.lock().await;
        let inner_clone = self.inner.clone();

        let uuid = Uuid::new_v4();

        let _ = inner.pending_tokens.insert(uuid, token_data);
        tracing::debug!("Inserting token {uuid} into cache");

        actix_web::rt::spawn(async move {
            time::sleep(TOKEN_EXPIRATION).await;

            let inner_mutex = inner_clone.lock().await;
            if inner_mutex.pending_tokens.contains_key(&uuid) {
                tracing::info!("Removing token {uuid}, expired");
                inner_mutex.pending_tokens.remove(&uuid);
            }
        });

        uuid
    }

    pub async fn use_token(
        &self,
        uuid: &Uuid,
    ) -> Result<WebSocketTokenData, errors::WebSocketServerError> {
        let inner = self.inner.lock().await;

        tracing::debug!("Using token {uuid}");

        let (_uuid, token) = inner
            .pending_tokens
            .remove(uuid)
            .ok_or_else(|| WebSocketServerError::TokenNotFound)?;

        Ok(token)
    }

    /// Broadcast an event to all connected clients
    pub async fn broadcast_event(&self, event: WebSocketEventMessage) {
        let msg =
            serde_json::to_string(&event).expect("Failed to turn event message into a string");

        self.broadcast(msg).await;
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, msg: impl Into<ByteString>) {
        let msg = msg.into();

        let inner = self.inner.lock().await;
        let mut futures = FuturesUnordered::new();

        for mut entry in inner.sessions.iter_mut() {
            let msg = msg.clone();
            tracing::info!("Sending msg: {msg}");

            futures.push(async move {
                let data = entry.value_mut();
                data.session.text(msg).await
            });
        }

        while let Some(result) = futures.next().await {
            if let Err(_) = result {
                tracing::warn!("Got an unexpected closed session");
            }
        }
    }
}

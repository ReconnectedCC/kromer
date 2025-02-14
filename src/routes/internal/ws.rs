use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use surrealdb::Uuid;
use crate::errors::KromerError;
use crate::websockets::types::common::WebSocketSubscriptionType;
use crate::websockets::WebSocketServer;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionQuery {
    pub session: String,
}

#[derive(Debug, Serialize)]
pub struct BasicSessionDataResponse {
    pub uuid: Uuid,
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct SessionDataResponse<'a> {
    pub address: &'a str,
    pub private_key: &'a Option<String>,
    pub subscriptions: Vec<WebSocketSubscriptionType>,
}

#[get("/session")]
async fn get_session(server: web::Data<WebSocketServer>, params: web::Query<SessionQuery>) -> Result<HttpResponse, KromerError> {
    let sessions = &server.inner.lock().await.sessions;

    let target_uuid = match params.session.parse::<Uuid>() {
        Ok(uuid) => uuid,
        Err(err) => {
            tracing::error!("Parse error: {}", err);
            return Err(KromerError::Internal("Failed to parse UUID"));
        }
    };

    let session_ref = match sessions.get(&target_uuid) {
        Some(data) => data,
        None => {
            tracing::error!("Invalid session: {}", target_uuid);
            return Err(KromerError::Internal("Session not found"));
        }
    };

    let session_data = session_ref.value();

    let vec: Vec<WebSocketSubscriptionType> = session_data.subscriptions
        .iter()
        .map(|entry| entry.key().clone())
        .collect();

    let resp = SessionDataResponse {
        address: &session_data.address,
        private_key: &session_data.private_key,
        subscriptions: vec,
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[get("/sessions")]
async fn get_sessions(server: web::Data<WebSocketServer>) -> Result<HttpResponse, KromerError> {
    let sessions = &server.inner.lock().await.sessions;

    let vec: Vec<BasicSessionDataResponse> = sessions
        .iter()
        .map(|entry| BasicSessionDataResponse { uuid: entry.key().clone(), address: entry.address.clone() })
        .collect();

    Ok(HttpResponse::Ok().json(vec))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ws")
            .service(get_session)
            .service(get_sessions)
    );
}
use actix_web::web;

use crate::guards;

pub mod docs;
pub mod index;
pub mod internal;
pub mod krist;
pub mod not_found;
pub mod v1;

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::IntoParams)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(50),
            offset: Some(0),
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1").configure(v1::config));
    cfg.service(web::scope("/api/krist").configure(krist::config));
    cfg.service(
        web::scope("/api/_internal")
            .guard(guards::internal_key_guard)
            .configure(internal::config),
    );
    cfg.service(web::scope("/docs").configure(docs::config));
    cfg.service(web::scope("").service(index::index_get));
}

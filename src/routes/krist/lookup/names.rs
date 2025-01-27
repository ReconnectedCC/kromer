use actix_web::{get, web, HttpResponse};

use crate::{errors::krist::KristError, AppState};

#[get("/{addresses}")]
async fn names_lookup(
    _state: web::Data<AppState>,
    names: web::Path<String>,
) -> Result<HttpResponse, KristError> {
    let names = names.into_inner();
    let _names: Vec<String> = names.split(',').map(|s| s.to_owned()).collect();

    todo!()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(names_lookup);
}

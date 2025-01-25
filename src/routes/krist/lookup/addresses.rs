use actix_web::{get, web, HttpResponse};

use crate::database::models::wallet::Model as Wallet;
use crate::{errors::krist::KristError, AppState};

#[get("/{addresses}")]
async fn addresses_lookup(
    state: web::Data<AppState>,
    addresses: web::Path<String>,
) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let addresses = addresses.into_inner();
    let addresses: Vec<String> = addresses.split(',').map(|s| s.to_owned()).collect();

    let resp = Wallet::lookup(db, addresses).await?;

    todo!()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(addresses_lookup);
}

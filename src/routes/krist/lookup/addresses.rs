use std::collections::HashMap;

use actix_web::{get, web, HttpResponse};

use crate::database::models::wallet::Model as Wallet;
use crate::models::addresses::AddressJson;
use crate::models::webserver::lookup::addresses::AddressLookupResponse;
use crate::{errors::krist::KristError, AppState};

#[get("/{addresses}")]
async fn addresses_lookup(
    state: web::Data<AppState>,
    addresses: web::Path<String>,
) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let addresses = addresses.into_inner();
    let addresses: Vec<String> = addresses.split(',').map(|s| s.to_owned()).collect();
    let addresses_len = addresses.len();

    let resp = Wallet::lookup(db, addresses).await?;
    let len = resp.len();

    let bwah: Vec<AddressJson> = resp
        .into_iter()
        .map(|lookup| AddressJson {
            address: lookup.model.address,
            first_seen: lookup.model.created_at.to_raw(),
            balance: lookup.model.balance,
            total_in: lookup.model.total_in,
            total_out: lookup.model.total_in,
            names: Some(lookup.names),
        })
        .collect();
    let hashmap: HashMap<String, AddressJson> = bwah
        .into_iter()
        .map(|model| (model.address.clone(), model))
        .collect();

    let response = AddressLookupResponse {
        ok: true,
        found: len,
        not_found: addresses_len - len,
        addresses: hashmap,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(addresses_lookup);
}

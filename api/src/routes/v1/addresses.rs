use actix_web::{get, error, web, Error, HttpResponse, Result};

use kromer_economy_entity::addresses;
use kromer_economy_service::Query;
use serde_json::json;

use crate::AppState;

#[get("/")]
async fn list_addresses(_data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hewwo from v1/addresses!!"))
}

#[get("/{address}")]
async fn get_specific_address(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let address = path.into_inner();

    let addr: Option<addresses::Model> = Query::find_address(conn, &address)
        .await
        .expect("could not find post");

    // Kinda cursed but it works
    match addr {
        Some(addr) => Ok(HttpResponse::Ok().json(json!({
            "ok": true,
            "address": {
                "address": addr.address,
                "balance": addr.balance,
                "totalin": addr.total_in,
                "totalout": addr.total_out,
                "firstseen": addr.first_seen,
            }
        }))),
        None => Ok(HttpResponse::NotFound().body(format!("Address: {address} (not found)"))),
    }
}

#[get("/rich/")] // TODO: Fix this, we should just be able to do `/addresses/rich`.
async fn get_richest_addresses(
    _data: web::Data<AppState>,
    path: web::Path<(u32, u32)>,
) -> Result<HttpResponse, Error> {
    let (limit, offset) = path.into_inner();

    Ok(HttpResponse::Ok().body("hai"))
}

#[get("/{address}/transactions")]
async fn get_address_transactions(
    address: web::Path<String>,
    _state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let address = address.into_inner();

    Ok(HttpResponse::Ok().body(address))
}

#[get("/{address}/names")]
async fn get_address_names(
    address: web::Path<String>,
    _state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let address = address.into_inner();

    Ok(HttpResponse::Ok().body(address))
}

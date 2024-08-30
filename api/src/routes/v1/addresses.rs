use actix_web::{get, web, Error, HttpResponse, Result};

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
    path: web::Path<(u64, u64)>,
) -> Result<HttpResponse, Error> {
    let (limit, offset) = path.into_inner();
    let conn = &_data.conn;

    let richest_addresses: Vec<addresses::Model> = Query::find_richest_addresses(conn, limit, offset)
        .await
        .expect("could not retrieve richest addresses"); // TODO: Handle this better

    let total = Query::count_total_addresses(conn)
        .await
        .expect("could not count total addresses"); // TODO: Handle this better

    let response: Vec<serde_json::Value> = richest_addresses
        .into_iter()
        .map(|addr| {
            json!({
                "address": addr.address,
                "balance": addr.balance,
                "totalin": addr.total_in,
                "totalout": addr.total_out,
                "firstseen": addr.first_seen,
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(json!({
        "ok": true,
        "count": response.len(),
        "total": total,
        "addresses": response,
    })))
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

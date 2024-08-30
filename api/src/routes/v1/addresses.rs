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
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let conn = &state.conn;
    let address = path.into_inner(); // TODO: Return if invalid address (may not be possible)

    let addr: Option<addresses::Model> = Query::find_address(conn, &address)
        .await
        .map_err(|e| { error::ErrorInternalServerError(e) })?;

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
        None => Ok(HttpResponse::Ok().json(json!({
                "ok": false,
                "error": "address_not_found"
            }
        ))),
    }
}

#[derive(Debug, serde::Deserialize)]
struct LimitAndOffset {
    limit: Option<u64>,
    offset: Option<u64>,
}

#[get("/rich")]
async fn get_richest_addresses(
    state: web::Data<AppState>,
    path: web::Query<LimitAndOffset>,
) -> Result<HttpResponse, Error> {
    let path = path.into_inner();
    let limit = path.limit.unwrap_or(50);
    let offset = path.offset.unwrap_or(0);

    let conn = &state.conn;

    let richest_addresses: Vec<addresses::Model> =
        Query::find_richest_addresses(conn, limit, offset)
            .await
            .map_err(|e| { error::ErrorInternalServerError(e) })?;

    let total = Query::count_total_addresses(conn)
        .await
        .map_err(|e| { error::ErrorInternalServerError(e) })?;

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
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let address = address.into_inner(); // TODO: Return if invalid address (may not be possible)

    let conn = &state.conn;

    let addr_exists = Query::find_address(conn, &address)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    if addr_exists.is_none() {
        return Ok(HttpResponse::Ok().json(json!({
            "ok": false,
            "error": "address_not_found"
        })));
    }

    // Im not particularly sure about the function name here
    let transaction_count = Query::count_total_transactions_from_address(conn, &address)
        .await
        .map_err(|e| { error::ErrorInternalServerError(e) })?;
    let transactions = Query::find_transactions_from_address(conn, &address)
        .await
        .map_err(|e| { error::ErrorInternalServerError(e) })?;

    // TODO: This is missing 2 fields, `metadata` and `type`, type can be `transfer`, `name_purchase`, `name_a_record`, or `name_transfer`. `metadata` is the CommonMeta shit.
    let response: Vec<serde_json::Value> = transactions
        .into_iter()
        .map(|tx| {
            json!({
                "id": tx.id,
                "from": tx.from,
                "to": tx.to,
                "value": tx.value,
                "time": tx.time,
                "name": tx.name,
                "sent_metaname": tx.sent_metaname,
                "sent_name": tx.sent_name,
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(json!({
        "ok": true,
        "count": response.len(),
        "total": transaction_count,
        "transactions": response,
    })))
}

#[get("/{address}/names")]
async fn get_address_names(
    _state: web::Data<AppState>,
    address: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let address = address.into_inner();

    Ok(HttpResponse::Ok().body(address))
}

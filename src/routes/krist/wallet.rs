use actix_web::{get, web, HttpResponse};

use crate::database::models::wallet::Model as Wallet;
use crate::errors::krist::{address::AddressError, KristError};
use crate::models::addresses::{AddressJson, AddressListResponse, AddressResponse};
use crate::models::names::NameJson;
use crate::models::transactions::{AddressTransactionQuery, TransactionJson};
use crate::{routes::PaginationParams, AppState};

#[get("")]
async fn wallet_list(
    state: web::Data<AppState>,
    pagination: web::Query<PaginationParams>,
) -> Result<HttpResponse, KristError> {
    let pagination = pagination.into_inner();
    let db = &state.db;

    let total = Wallet::count(db).await?;
    let addresses = Wallet::all(db, &pagination)
        .await?
        .into_iter()
        .map(|wallet| wallet.into())
        .collect::<Vec<AddressJson>>();

    Ok(HttpResponse::Ok().json(AddressListResponse {
        ok: true,
        count: addresses.len(),
        total,
        addresses,
    }))
}

#[get("/{address}")]
async fn wallet_get(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> Result<HttpResponse, KristError> {
    let address = address.into_inner();
    let db = &state.db;

    // Fuck the borrow checker.
    let wallet = Wallet::get_by_address_excl(db, address.clone()).await?;

    wallet
        .map(|addr| AddressResponse {
            ok: true,
            address: addr.into(),
        })
        .map(|response| HttpResponse::Ok().json(response))
        .ok_or_else(|| KristError::Address(AddressError::NotFound(address)))
}

#[get("/rich")]
async fn wallet_richest(
    state: web::Data<AppState>,
    pagination: web::Query<PaginationParams>,
) -> Result<HttpResponse, KristError> {
    let pagination = pagination.into_inner();
    let db = &state.db;

    let total = Wallet::count(db).await?;
    let addresses = Wallet::get_richest(db, &pagination)
        .await?
        .into_iter()
        .map(|wallet| wallet.into())
        .collect::<Vec<AddressJson>>();

    Ok(HttpResponse::Ok().json(AddressListResponse {
        ok: true,
        count: addresses.len(),
        total,
        addresses,
    }))
}

#[get("/{address}/transactions")]
async fn wallet_get_transactions(
    state: web::Data<AppState>,
    address: web::Path<String>,
    params: web::Query<AddressTransactionQuery>,
) -> Result<HttpResponse, KristError> {
    let address = address.into_inner();
    let params = params.into_inner();
    let db = &state.db;

    let transactions = Wallet::transactions(db, address, &params).await?;
    let transactions: Vec<TransactionJson> =
        transactions.into_iter().map(|trans| trans.into()).collect();

    Ok(HttpResponse::Ok().json(transactions))
}

#[get("/{address}/names")]
async fn wallet_get_names(
    state: web::Data<AppState>,
    address: web::Path<String>,
    params: web::Query<PaginationParams>,
) -> Result<HttpResponse, KristError> {
    let address = address.into_inner();
    let params = params.into_inner();
    let db = &state.db;

    let names = Wallet::names(db, address, &params).await?;
    let names: Vec<NameJson> = names.into_iter().map(|name| name.into()).collect();

    Ok(HttpResponse::Ok().json(names))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/addresses")
            .service(wallet_richest)
            .service(wallet_get)
            .service(wallet_get_transactions)
            .service(wallet_get_names)
            .service(wallet_list),
    );
}

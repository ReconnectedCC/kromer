use actix_web::{get, post, web, HttpResponse};
use serde_json::json;

use crate::database::models::name::Model as Name;
use crate::database::models::transaction::{Model as Transaction, TransactionCreateData};
use crate::database::models::wallet::Model as Wallet;
use crate::errors::krist::address::AddressError;
use crate::errors::krist::generic::GenericError;
use crate::errors::krist::transaction::TransactionError;
use crate::errors::krist::{name::NameError, KristError};
use crate::models::motd::MINING_CONSTANTS;
use crate::models::names::{
    NameCostResponse, NameJson, NameListResponse, NameResponse, RegisterNameRequest,
};
use crate::models::transactions::TransactionType;
use crate::utils::validation_kromer::is_valid_name;
use crate::{routes::PaginationParams, AppState};

fn clean_name_input(name: String) -> String {
    name.trim().to_lowercase()
}

#[get("")]
async fn name_list(
    state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, KristError> {
    let params = query.into_inner();
    let db = &state.db;

    let total = Name::count(db).await?;

    let names = Name::all(db, &params).await?;
    let names: Vec<NameJson> = names.into_iter().map(|name| name.into()).collect();

    let response = NameListResponse {
        ok: true,
        count: names.len(),
        total,
        names,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/check/{name}")]
async fn name_check(
    state: web::Data<AppState>,
    name: web::Path<String>,
) -> Result<HttpResponse, KristError> {
    let name = name.into_inner();
    let db = &state.db;

    if !is_valid_name(name.clone(), true) {
        return Err(KristError::Generic(GenericError::InvalidParameter(
            "name".to_string(),
        )));
    }
    let name = clean_name_input(name);

    let db_name = Name::get_by_name(db, name).await?;

    Ok(HttpResponse::Ok().json(json!({
        "ok": true,
        "available": db_name.is_none()
    })))
}

#[get("/cost")]
async fn name_cost() -> Result<HttpResponse, KristError> {
    let response = NameCostResponse {
        ok: true,
        name_cost: MINING_CONSTANTS.name_cost,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[get("/new")]
async fn name_new(
    state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, KristError> {
    let params = query.into_inner();
    let db = &state.db;

    let names = Name::all_unpaid(db, &params).await?;
    let names: Vec<NameJson> = names.into_iter().map(|name| name.into()).collect();

    Ok(HttpResponse::Ok().json(names))
}

#[get("/bonus")]
async fn name_bonus(state: web::Data<AppState>) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let name_bonus = Name::count_unpaid(db).await?;

    Ok(HttpResponse::Ok().json(json!({
        "ok": true,
        "name_bonus": name_bonus
    })))
}

#[post("/{name}")]
async fn name_register(
    state: web::Data<AppState>,
    name: web::Path<String>,
    details: web::Json<Option<RegisterNameRequest>>,
) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let name = clean_name_input(name.into_inner());
    let new_name_cost = rust_decimal::Decimal::new(MINING_CONSTANTS.name_cost, 0);

    let private_key = details
        .as_ref()
        .map(|json_details| json_details.private_key.clone());

    // Manual error handling here
    if private_key.is_none() {
        return Err(KristError::Generic(GenericError::MissingParameter(
            "privatekey".to_string(),
        )));
    }
    // if desired_name.is_none() {
    //     return Err(KristError::Generic(GenericError::MissingParameter("desiredName".to_string())))
    // }
    if !is_valid_name(name.clone(), false) {
        return Err(KristError::Generic(GenericError::InvalidParameter(
            "name".to_string(),
        )));
    }

    let verify_addr_resp = Wallet::verify_address(
        db,
        // Unwrap should be okay
        private_key.unwrap().clone(),
    )
    .await?;

    if !verify_addr_resp.authed {
        tracing::info!(
            "Name registration REJECTED for {}",
            verify_addr_resp.address.address
        );
        return Err(KristError::Address(AddressError::AuthFailed));
    }

    // TODO: Rate limit check. Apply a 2x cost to name events

    // Reject insufficient funds
    if verify_addr_resp.address.balance < new_name_cost {
        return Err(KristError::Transaction(TransactionError::InsufficientFunds));
    }

    // Create the transaction
    let creation_data = TransactionCreateData {
        from: verify_addr_resp.address.address.clone(),
        to: "name".to_string(),
        amount: new_name_cost,
        metadata: None,
        transaction_type: TransactionType::NamePurchase,
    };
    let _trans_response: Vec<Transaction> = db.insert("transaction").content(creation_data).await?;

    // Create the new name
    let _name_response =
        Name::register_name(db, name.clone(), verify_addr_resp.address.address).await?;

    Ok(HttpResponse::Ok().json(json!({
        "ok": true,
        "name": name
    })))
}

#[get("/{name}")]
async fn name_get(
    state: web::Data<AppState>,
    name: web::Path<String>,
) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let name = name.into_inner();

    let db_name = Name::get_by_name(db, name.clone()).await?;

    db_name
        .map(|name| NameResponse {
            ok: true,
            name: name.into(),
        })
        .map(|response| HttpResponse::Ok().json(response))
        .ok_or_else(|| KristError::Name(NameError::NameNotFound(name)))

    // Ok(HttpResponse::Ok().json(
    //     json!({
    //         "ok": true,
    //         "name": name
    //     })
    // ))
}

// #[get("/{id}")]
// async fn name_get(
//     state: web::Data<AppState>,
//     id: web::Path<String>,
// ) -> Result<HttpResponse, KristError> {
//     let id = id.into_inner();
//     let db = &state.db;

//     let slim = Name::get_partial(db, &id).await?;

//     slim.map(|name| NameResponse {
//         ok: true,
//         name: name.into(),
//     })
//     .map(|response| HttpResponse::Ok().json(response))
//     .ok_or_else(|| KristError::Name(NameError::NameNotFound(id)))
// }

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/names")
            .service(name_list)
            .service(name_cost)
            .service(name_check)
            .service(name_bonus)
            .service(name_new)
            .service(name_get)
            .service(name_register),
    );
}

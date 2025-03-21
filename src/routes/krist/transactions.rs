use actix_web::{get, post, web, HttpResponse};
use rust_decimal_macros::dec;

use crate::database::models::transaction::{Model as Transaction, TransactionCreateData};
use crate::database::models::wallet::Model as Wallet;
use crate::errors::krist::address::AddressError;
use crate::errors::krist::generic::GenericError;
use crate::errors::krist::{transaction::TransactionError, KristError};
use crate::models::transactions::{
    TransactionDetails, TransactionJson, TransactionListResponse, TransactionResponse,
    TransactionType,
};
use crate::models::websockets::{WebSocketEvent, WebSocketMessage};
use crate::websockets::WebSocketServer;
use crate::{routes::PaginationParams, AppState};

#[utoipa::path(
    get,
    tags = ["Transactions"],
    path = "/transactions",
    operation_id = "List transactions",
    responses(
        (status=200, description="Transactions fetched successfully", body = TransactionListResponse),
        (status=404, description = "Transactions were not found")
    ),
    params(
        PaginationParams
    )
)]
#[get("")]
pub async fn transaction_list(
    state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, KristError> {
    let params = query.into_inner();
    let db = &state.db;

    let total = Transaction::count(db).await?;

    let transactions = Transaction::all(db, &params).await?;
    let transactions: Vec<TransactionJson> =
        transactions.into_iter().map(|trans| trans.into()).collect();

    let response = TransactionListResponse {
        ok: true,
        count: transactions.len(),
        total,
        transactions,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("")]
async fn transaction_create(
    state: web::Data<AppState>,
    server: web::Data<WebSocketServer>,
    details: web::Json<TransactionDetails>,
) -> Result<HttpResponse, KristError> {
    let details = details.into_inner();
    let db = &state.db;

    // Check on the server so DB doesnt throw.
    if details.amount < dec!(0.0) {
        return Err(KristError::Generic(GenericError::InvalidParameter(
            "amount".to_string(),
        )));
    }

    let sender_verify_response = Wallet::verify_address(db, details.password).await?;

    let sender = sender_verify_response.address;

    let recipient = Wallet::get_by_address(db, details.to.clone())
        .await?
        .ok_or_else(|| KristError::Address(AddressError::NotFound(details.to)))?;

    // Make sure to check the request to see if the funds are available.
    if sender.balance < details.amount {
        return Err(KristError::Transaction(TransactionError::InsufficientFunds));
    }

    let creation_data = TransactionCreateData {
        from: sender.address,
        to: recipient.address,
        amount: details.amount,
        metadata: details.metadata,
        transaction_type: TransactionType::Transfer,
    };
    let response: Vec<Transaction> = db.insert("transaction").content(creation_data).await?;
    let model = response.first().unwrap(); // the fuck man
    let response: TransactionJson = model.clone().into();

    let event = WebSocketMessage::new_event(WebSocketEvent::Transaction {
        transaction: response.clone(),
    });
    server.broadcast_event(event).await;

    let final_response = TransactionResponse {
        ok: true,
        transaction: response,
    };

    Ok(HttpResponse::Ok().json(final_response))
}

#[get("/latest")]
async fn transaction_latest(
    state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, KristError> {
    let params = query.into_inner();
    let db = &state.db;

    let total = Transaction::count(db).await?;
    let transactions = Transaction::sorted_by_date(db, &params).await?;

    let transactions: Vec<TransactionJson> =
        transactions.into_iter().map(|trans| trans.into()).collect();

    let response = TransactionListResponse {
        ok: true,
        count: transactions.len(),
        total,
        transactions,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/{id}")]
async fn transaction_get(
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, KristError> {
    let id = id.into_inner();
    let db = &state.db;

    let slim = Transaction::get_partial(db, id).await?;

    slim.map(|trans| TransactionResponse {
        ok: true,
        transaction: trans.into(),
    })
    .map(|response| HttpResponse::Ok().json(response))
    .ok_or_else(|| KristError::Transaction(TransactionError::NotFound))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/transactions")
            .service(transaction_create)
            .service(transaction_latest)
            .service(transaction_get)
            .service(transaction_list),
    );
}

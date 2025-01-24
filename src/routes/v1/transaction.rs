use actix_web::{get, post, web, HttpResponse};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::database::models::transaction::{Model as Transaction, TransactionCreateData};
use crate::database::models::wallet::Model as Wallet;

use crate::errors::wallet::WalletError;
use crate::models::transactions::TransactionType;
use crate::{
    errors::{transaction::TransactionError, KromerError},
    routes::PaginationParams,
    AppState,
};

#[derive(Debug, serde::Deserialize)]
struct TransactionDetails {
    pub password: String,
    pub to: String,
    pub amount: Decimal,
    pub metadata: Option<String>,
}

#[get("/list")]
async fn transaction_list(
    state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, KromerError> {
    let params = query.into_inner();
    let db = &state.db;

    let transactions = Transaction::all(db, &params).await?;

    Ok(HttpResponse::Ok().json(transactions))
}

#[get("/{id}")]
async fn transaction_get(
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, KromerError> {
    let id = id.into_inner();
    let db = &state.db;

    let slim = Transaction::get_partial(db, id).await?;

    Ok(HttpResponse::Ok().json(slim))
}

#[post("/create")]
async fn transaction_create(
    state: web::Data<AppState>,
    details: web::Json<TransactionDetails>,
) -> Result<HttpResponse, KromerError> {
    let details = details.into_inner();
    let db = &state.db;

    // Check on the server so DB doesnt throw.
    if details.amount < dec!(0.0) {
        return Err(KromerError::Transaction(TransactionError::InvalidAmount));
    }

    let sender = Wallet::verify(db, details.password)
        .await?
        .ok_or_else(|| KromerError::Wallet(WalletError::InvalidPassword))?;
    let recipient = Wallet::get_by_address(db, details.to)
        .await?
        .ok_or_else(|| KromerError::Wallet(WalletError::NotFound))?;

    // Make sure to check the request to see if the funds are available.
    if sender.balance < details.amount {
        return Err(KromerError::Transaction(
            TransactionError::InsufficientFunds,
        ));
    }

    let creation_data = TransactionCreateData {
        from: sender.address,
        to: recipient.address,
        amount: details.amount,
        metadata: details.metadata,
        transaction_type: TransactionType::Transfer,
    };
    let response: Vec<Transaction> = db.insert("transaction").content(creation_data).await?;
    let response = response.first().unwrap(); // the fuck man

    Ok(HttpResponse::Ok().json(response))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/transaction")
            .service(transaction_list)
            .service(transaction_create)
            .service(transaction_get),
    );
}

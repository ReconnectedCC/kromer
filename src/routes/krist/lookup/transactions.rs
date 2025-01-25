use actix_web::{get, web, HttpResponse};

use crate::{errors::krist::KristError, AppState};

#[get("/{transactions}")]
async fn transactions_lookup(
    state: web::Data<AppState>,
    transactions: web::Path<String>,
) -> Result<HttpResponse, KristError> {
    let transactions = transactions.into_inner();
    let transactions: Vec<String> = transactions.split(',').map(|s| s.to_owned()).collect();

    todo!()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(transactions_lookup);
}

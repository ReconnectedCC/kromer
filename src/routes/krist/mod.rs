mod lookup;
mod misc;
mod names;
mod transactions;
mod wallet;
mod ws;

use actix_web::web;
use utoipa::OpenApi;
use crate::routes::krist::transactions::__path_transaction_list;
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/lookup").configure(lookup::config));

    cfg.configure(wallet::config);
    cfg.configure(transactions::config);
    cfg.configure(ws::config);
    cfg.configure(names::config);
    cfg.configure(misc::config);
}

#[derive(OpenApi)]
#[openapi(paths(transaction_list))]
pub struct TransactionsApiDoc;
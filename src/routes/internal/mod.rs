pub mod wallet;
pub mod ws;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(wallet::config);
    cfg.configure(ws::config);
}

mod lookup;
mod misc;
mod names;
mod transactions;
mod wallet;
mod ws;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(wallet::config);
    cfg.configure(transactions::config);
    cfg.configure(ws::config);
    cfg.configure(names::config);
    cfg.configure(misc::config);
    cfg.service(web::scope("/lookup").configure(lookup::config));
}

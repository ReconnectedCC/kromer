use actix_web::{get, post, web, HttpResponse};

use crate::database::models::wallet::Model as Wallet;
use crate::models::motd::{Constants, CurrencyInfo, DetailedMotd, PackageInfo};
use crate::AppState;
use crate::{
    errors::krist::KristError,
    models::auth::{AddressAuthenticationResponse, LoginDetails},
};

#[post("/login")]
async fn login_address(
    state: web::Data<AppState>,
    query: web::Json<LoginDetails>,
) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let query = query.into_inner();

    let private_key = query.private_key;
    let result = Wallet::verify(&db, private_key).await?;

    match result {
        Some(model) => Ok(HttpResponse::Ok().json(AddressAuthenticationResponse {
            address: Some(model.address),
            authed: true,
            ok: true,
        })),
        None => Ok(HttpResponse::Ok().json(AddressAuthenticationResponse {
            address: None,
            authed: false,
            ok: true,
        })),
    }
}

#[get("/motd")]
async fn get_motd() -> HttpResponse {
    // This is by far the simplest fucking route in all of Kromer.
    // TODO: Make this actually better.
    let motd = DetailedMotd {
        server_time: "server_time".to_string(),
        motd: "Message of the day".to_string(),
        set: None,
        motd_set: None,
        public_url: "https://kromer.uwu".to_string(),
        public_ws_url: "https://kromer.uwu/krist/ws".to_string(),
        mining_enabled: false,
        transactions_enabled: true,
        debug_mode: true,
        work: 500,
        last_block: None,
        package: PackageInfo {
            name: "Kromer".to_string(),
            version: "0.2.0".to_string(),
            author: "ReconnectedCC Team".to_string(),
            license: "GPL-3.0".to_string(),
            repository: "https://github.com/ReconnectedCC/kromer/".to_string(),
        },
        constants: Constants {
            wallet_version: 3,
            nonce_max_size: 500,
            name_cost: 500,
            min_work: 50,
            max_work: 500,
            work_factor: 500.0,
            seconds_per_block: 5000,
        },
        currency: CurrencyInfo {
            address_prefix: "k".to_string(),
            name_suffix: "kro".to_string(),
            currency_name: "Kromer".to_string(),
            currency_symbol: "Ϗ".to_string(),
        },
        notice: "Some awesome notice will go here".to_string(),
    };

    HttpResponse::Ok().json(motd)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("").service(login_address).service(get_motd));
}

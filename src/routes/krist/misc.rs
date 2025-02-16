use actix_web::{get, post, web, HttpResponse};

use crate::database::models::wallet::Model as Wallet;
use crate::models::misc::{MoneySupplyResponse, PrivateKeyAddressResponse, WalletVersionResponse};
use crate::models::motd::{
    Constants, CurrencyInfo, DetailedMotd, DetailedMotdResponse, PackageInfo,
};
use crate::{
    errors::krist::KristError,
    models::auth::{AddressAuthenticationResponse, LoginDetails},
};
use crate::{utils, AppState};

#[post("/login")]
async fn login_address(
    state: web::Data<AppState>,
    query: web::Json<LoginDetails>,
) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let query = query.into_inner();

    let private_key = query.private_key;
    let result = Wallet::verify_address(db, private_key).await?;

    Ok(HttpResponse::Ok().json(AddressAuthenticationResponse {
        address: result.authed.then_some(result.address.address),
        authed: result.authed,
        ok: true,
    }))
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
        public_url: "http://kromer.reconnected.cc".to_string(),
        public_ws_url: "http://kromer.reconnected.cc/api/krist/ws".to_string(),
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
            currency_symbol: "KRO".to_string(),
        },
        notice: "Some awesome notice will go here".to_string(),
    };

    let motd = DetailedMotdResponse { ok: true, motd };

    HttpResponse::Ok().json(motd)
}

#[get("/walletversion")]
async fn get_walletversion() -> HttpResponse {
    let response = WalletVersionResponse {
        ok: true,
        wallet_version: 3,
    };

    HttpResponse::Ok().json(response)
}

#[get("/supply")]
async fn get_kromer_supply(state: web::Data<AppState>) -> Result<HttpResponse, KristError> {
    let db = &state.db;
    let supply = Wallet::supply(db).await?;

    let response = MoneySupplyResponse { ok: true, supply };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/v2")]
async fn get_v2_address(query: web::Json<LoginDetails>) -> Result<HttpResponse, KristError> {
    let query = query.into_inner();
    let key = query.private_key;

    let address = utils::crypto::make_v2_address(&key, "k");
    let response = PrivateKeyAddressResponse { address, ok: true };

    Ok(HttpResponse::Ok().json(response))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(login_address)
            .service(get_motd)
            .service(get_kromer_supply)
            .service(get_v2_address),
    );
}

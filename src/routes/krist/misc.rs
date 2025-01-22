use actix_web::{post, web, HttpResponse};

use crate::database::models::wallet::Model as Wallet;
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

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("").service(login_address));
}

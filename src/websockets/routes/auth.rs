use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::database::models::wallet::Model as Wallet;
use crate::errors::wallet::WalletError;
use crate::errors::KromerError;
use crate::models::addresses::AddressJson;
use crate::{models::auth::LoginDetails, websockets::wrapped_ws::WrappedWsData};

pub async fn perform_login(
    ws_metadata: &WrappedWsData,
    login_details: LoginDetails,
    db: &Surreal<Any>,
) -> Result<(WrappedWsData, AddressJson), KromerError> {
    // We don't necessarily care if they are logged in to a wallet already, or if they are a guest,
    // so we just want to verify that the WsMessageType has the LoginDetails struct on it,
    // and that it is valid

    // Check the wallet to verify
    let privatekey = login_details.private_key;
    let wallet = Wallet::verify_address(db, privatekey.clone())
        .await
        .map_err(|_| KromerError::Wallet(WalletError::InvalidPassword))?;

    let wallet = wallet.address;
    let address = wallet.address.clone();
    let new_ws_data = WrappedWsData {
        address,
        privatekey: Some(privatekey),
        ..ws_metadata.to_owned()
    };
    let wallet: AddressJson = wallet.into();

    Ok((new_ws_data, wallet))
}

pub async fn perform_logout(ws_metadata: &WrappedWsData) -> WrappedWsData {
    // No matter if they are logged into a wallet or not, we just want to reset the auth details.

    WrappedWsData {
        address: "guest".to_string(),
        privatekey: None,
        ..ws_metadata.to_owned()
    }
}

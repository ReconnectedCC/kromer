use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct WalletVersionResponse {
    pub ok: bool,
    #[serde(rename = "walletVersion")]
    pub wallet_version: u8,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MoneySupplyResponse {
    pub ok: bool,
    pub supply: Decimal,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PrivateKeyAddressResponse {
    pub ok: bool,
    pub address: String,
}

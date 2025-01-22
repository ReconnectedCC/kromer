use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct WalletVersionResponse {
    pub ok: bool,
    #[serde(rename = "walletVersion")]
    pub wallet_version: u8,
}

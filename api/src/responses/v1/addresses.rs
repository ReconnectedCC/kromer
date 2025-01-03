use kromer_economy_entity::addresses;
use sea_orm::sqlx::types::chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressResponse {
    pub ok: bool,
    pub total: u64,
    pub count: u64,
    pub addresses: Vec<Address>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingularAddressResponse {
    pub ok: bool,
    pub address: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
    pub balance: f32,
    #[serde(rename = "totalin")]
    pub total_in: f32,
    #[serde(rename = "totalout")]
    pub total_out: f32,
    #[serde(rename = "firstseen")]
    pub first_seen: DateTime<FixedOffset>,
}

impl From<addresses::Model> for Address {
    fn from(address: addresses::Model) -> Self {
        Address {
            address: address.address,
            balance: address.balance,
            total_in: address.total_in,
            total_out: address.total_out,
            first_seen: address.first_seen,
        }
    }
}

impl From<&addresses::Model> for Address {
    fn from(value: &addresses::Model) -> Self {
        Address {
            address: value.address.clone(),
            balance: value.balance,
            total_in: value.total_in,
            total_out: value.total_out,
            first_seen: value.first_seen,
        }
    }
}

pub mod addresses;
pub mod auth;
pub mod blocks;
pub mod error;
pub mod misc;
pub mod motd;
pub mod names;
pub mod transactions;
pub mod webserver;
pub mod websockets;

use serde::{Deserialize, Deserializer};

pub fn deserialize_number_into_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Number(usize),
    }

    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::String(s) => Ok(s),
        StringOrInt::Number(i) => Ok(i.to_string()),
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::addresses::AddressJson;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookupResponse {
    pub ok: bool,
    pub found: usize,
    #[serde(rename = "notFound")]
    pub not_found: usize,
    pub addresses: HashMap<String, AddressJson>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryParameters {
    #[serde(rename = "fetchNames")]
    pub fetch_names: Option<bool>, // Might be possible to use `#[serde(default)]`?
}

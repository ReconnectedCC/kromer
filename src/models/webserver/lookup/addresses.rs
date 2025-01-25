use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::addresses::AddressJson;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddressLookupResponse {
    pub ok: bool,
    pub found: usize,
    #[serde(rename = "notFound")]
    pub not_found: usize,
    pub addresses: HashMap<String, AddressJson>,
}

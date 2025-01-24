use serde::{Deserialize, Serialize};

use crate::database::models::name;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NameListResponse {
    pub ok: bool,
    /// The count of results.
    pub count: usize,
    /// The total amount of transactions
    pub total: usize,
    pub names: Vec<NameJson>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NameResponse {
    pub ok: bool,
    pub name: NameJson,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NameCostResponse {
    pub ok: bool,
    pub name_cost: i64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DetailedUnpaidResponseRow {
    pub count: i64,
    pub unpaid: i64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RegisterNameRequest {
    //#[serde(rename = "desiredName")]
    //pub desired_name: String,
    #[serde(rename = "privatekey")]
    pub private_key: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NameJson {
    pub name: String,
    pub owner: Option<String>,
    pub original_owner: Option<String>,
    pub registered: Option<String>,
    pub updated: Option<String>,
    pub transfered: Option<String>,
    pub unpaid: i64,
}

impl From<name::Model> for NameJson {
    fn from(name: name::Model) -> Self {
        Self {
            name: name.name,
            owner: Some(name.owner),             // TODO: Use correct values
            original_owner: name.original_owner, // TODO: Populate this.
            registered: Some(name.registered.to_rfc3339()),
            updated: None,
            transfered: None, // TODO: Populate this
            unpaid: 0,
        }
    }
}

pub mod addresses;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockLookupFields {
    Height,
    Address,
    Hash,
    Value,
    Time,
    Difficulty,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionLookupFields {
    Id,
    From,
    To,
    Value,
    Time,
    SentName,
    SentMetaname,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NameLookupFields {
    Name,
    Owner,
    #[serde(rename = "original_owner")]
    OriginalOwner,
    Registered,
    Updated,
    Transfered,
    #[serde(rename = "transferredOrRegistered")]
    TransferedOrRegistered,
    A,
    Unpaid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupQuery {
    pub limit: Option<String>,
    pub offset: Option<String>,
    pub order_by: Option<String>,
    pub order: Option<String>,
}

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::database::models::transaction;
use transaction::TransactionNameData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TransactionListResponse {
    pub ok: bool,

    /// The count of results.
    pub count: usize,

    /// The total amount of transactions
    pub total: usize,

    pub transactions: Vec<TransactionJson>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TransactionDetails {
    pub password: String,
    pub to: String,
    pub amount: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub ok: bool,
    pub transaction: TransactionJson,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AddressTransactionQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(rename = "includeMined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_mined: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TransactionJson {
    /// The ID of this transaction.
    pub id: i64,

    /// The sender of this transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    /// The recipient of this transaction. This may be `name` if the transaction was a name purchase, or `a` if it was a name's data change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,

    /// The amount of Krist transferred in this transaction. Can be 0, notably if the transaction was a name's data change.
    pub value: Decimal,

    /// The time this transaction this was made, as an ISO-8601 string.
    pub time: String,

    /// The name associated with this transaction, without the `.kst` suffix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_metaname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_name: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    #[default]
    Unknown,
    Mined,
    NamePurchase,
    NameARecord,
    NameTransfer,
    Transfer,
}

impl From<transaction::Model> for TransactionJson {
    fn from(transaction: transaction::Model) -> Self {
        let name_data = TransactionNameData::parse_opt_ref(&transaction.metadata);

        Self {
            id: 0, // We dont do incremental IDs, do we give a shit?
            from: Some(transaction.from),
            to: Some(transaction.to),
            value: transaction.amount,
            time: transaction.timestamp.to_raw(),
            name: None, // TODO: Populate this later, maybe with a separate function.
            metadata: transaction.metadata,
            sent_metaname: name_data.meta,
            sent_name: name_data.name,
            transaction_type: transaction.transaction_type,
        }
    }
}

impl From<TransactionType> for &str {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Unknown => "unknown",
            TransactionType::Mined => "mined",
            TransactionType::NamePurchase => "name_purchase",
            TransactionType::NameARecord => "name_a_record",
            TransactionType::NameTransfer => "name_transfer",
            TransactionType::Transfer => "transfer",
        }
    }
}

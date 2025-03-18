use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use utoipa::{
    openapi::{RefOr, Response, ResponseBuilder},
    ToResponse, ToSchema,
};

use crate::database::models::serialize_record_id_opt;
use crate::database::models::transaction;
use transaction::TransactionNameData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, ToResponse, ToSchema)]
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
    #[serde(serialize_with = "serialize_record_id_opt")]
    pub id: Option<Thing>,

    /// The sender of this transaction.
    pub from: String,

    /// The recipient of this transaction. This may be `name` if the transaction was a name purchase, or `a` if it was a name's data change.
    pub to: String,

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

impl<'__r> ToResponse<'__r> for TransactionJson {
    fn response() -> (&'__r str, RefOr<Response>) {
        (
            "TransactionResponse",
            ResponseBuilder::new()
                .description("Transaction Response")
                .build()
                .into(),
        )
    }
}

impl utoipa::ToSchema for TransactionJson {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Transaction")
    }
}

impl utoipa::PartialSchema for TransactionJson {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .property(
                "id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .examples(Some(serde_json::json!({"name": "An example transaction"})))
            .into()
    }
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
            id: transaction.id, // We dont do incremental IDs, do we give a shit?
            from: transaction.from,
            to: transaction.to,
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

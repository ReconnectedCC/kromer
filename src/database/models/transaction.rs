use once_cell::sync::Lazy;
use regex::Regex;
use surrealdb::{
    engine::any::Any,
    sql::{Datetime, Id, Thing},
    Surreal,
};

use rust_decimal::Decimal;

use super::{serialize_record_opt, CountResponse};
use crate::{models::transactions::TransactionType, routes::PaginationParams};

static KST_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:([a-z0-9-_]{1,32})@)?([a-z0-9]{1,64})\.kst").unwrap());

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Model {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_record_opt"
    )]
    pub id: Option<Thing>,
    pub amount: Decimal,
    pub from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    pub timestamp: Datetime,
    pub to: String,
    pub transaction_type: TransactionType,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TransactionCreateData {
    pub from: String,
    pub to: String,
    pub amount: Decimal,
    pub metadata: Option<String>,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct TransactionNameData {
    pub meta: Option<String>,
    pub name: Option<String>,
}

impl Model {
    /// Get a transaction from its unique ID
    pub async fn get<S: AsRef<str>>(
        db: &Surreal<Any>,
        id: S,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = id.as_ref();
        let thing: Thing = id.try_into().unwrap();
        let q = "SELECT * FROM transaction WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get a transaction from its unique ID, not including the table part
    pub async fn get_partial<S: AsRef<str>>(
        db: &Surreal<Any>,
        id: S,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = id.as_ref();
        let id = Id::from(id);
        let thing = Thing::from(("transaction", id));

        let q = "SELECT * FROM transaction WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get all transactions, omitting id.
    pub async fn all(
        db: &Surreal<Any>,
        pagination: &PaginationParams,
    ) -> Result<Vec<Model>, surrealdb::Error> {
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q = "SELECT * OMIT id from transaction LIMIT $limit START $offset";

        let mut response = db
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<Model> = response.take(0)?;

        Ok(models)
    }

    /// Get the total amount of transactions in the database
    pub async fn count(db: &Surreal<Any>) -> Result<usize, surrealdb::Error> {
        let q = "(SELECT count() FROM transaction GROUP BY count)[0] or { count: 0 }";

        let mut response = db.query(q).await?;
        let count: Option<CountResponse> = response.take(0)?;
        let count = count.unwrap_or_default(); // Its fine, we make sure we always get a response with the `or` statement in the query.

        Ok(count.count)
    }

    /// Get all transactions ordered by date in descending order.
    pub async fn sorted_by_date(
        db: &Surreal<Any>,
        pagination: &PaginationParams,
    ) -> Result<Vec<Model>, surrealdb::Error> {
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q =
            "SELECT * OMIT id FROM transaction ORDER BY timestamp DESC LIMIT $limit START $offset;";

        let mut response = db
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<Model> = response.take(0)?;

        Ok(models)
    }
}

impl TransactionNameData {
    /// Parse a transaction name from a string-like type according to CommonMeta format.
    /// Takes any type that can be converted to a string reference.
    ///
    /// If the input is empty, returns a default `TransactionNameData`.
    /// Otherwise parses according to the pattern: `meta@name.kst`
    ///
    /// # Examples
    /// ```
    /// let data = TransactionNameData::parse("meta@name.kst");
    /// assert_eq!(data.meta, Some("meta".to_string()));
    /// assert_eq!(data.name, Some("name".to_string()));
    ///
    /// let empty = TransactionNameData::parse("");
    /// assert_eq!(empty, TransactionNameData::default());
    /// ```
    pub fn parse<S: AsRef<str>>(input: S) -> Self {
        let input = input.as_ref();
        if input.is_empty() {
            return Self::default(); // Don't do useless parsing if the input is empty, thats silly.
        }

        match KST_REGEX.captures(input) {
            Some(captures) => {
                let meta = captures.get(1).map(|m| m.as_str().to_string()); // TODO: Less allocating, should maybe use `&str` on the transaction models
                let name = captures.get(2).map(|m| m.as_str().to_string());

                Self { meta, name }
            }
            None => Self::default(),
        }
    }

    /// Parse a transaction name from an optional string-like type.
    /// If the input is `None`, returns a default `TransactionNameData`.
    /// Otherwise, parses the string according to CommonMeta format.
    ///
    /// # Examples
    /// ```
    /// let data = TransactionNameData::parse_opt(Some("meta@name.kst"));
    /// assert_eq!(data.meta, Some("meta".to_string()));
    /// assert_eq!(data.name, Some("name".to_string()));
    ///
    /// let empty = TransactionNameData::parse_opt(None::<String>);
    /// assert_eq!(empty, TransactionNameData::default());
    /// ```
    pub fn parse_opt<S: AsRef<str>>(input: Option<S>) -> Self {
        if input.is_none() {
            return Self::default(); // Do we really need to parse stuff is there is no value? No, we dont.
        }

        let input = input.unwrap(); // We can do this, we made sure it exists.
        Self::parse(input)
    }

    /// Parse a transaction name from a reference to an optional string-like type.
    /// If the input is `None`, returns a default `TransactionNameData`.
    /// Otherwise, parses the string according to CommonMeta format.
    ///
    /// # Examples
    /// ```
    /// let input = Some("meta@name.kst".to_string());
    /// let data = TransactionNameData::parse_opt_ref(&input);
    /// assert_eq!(data.meta, Some("meta".to_string()));
    /// assert_eq!(data.name, Some("name".to_string()));
    ///
    /// let empty = TransactionNameData::parse_opt_ref(&None::<String>);
    /// assert_eq!(empty, TransactionNameData::default());
    /// ```
    pub fn parse_opt_ref<S: AsRef<str>>(input: &Option<S>) -> Self {
        if input.is_none() {
            return Self::default(); // Do we really need to parse stuff is there is no value? No, we dont.
        }

        let input = input.as_ref().unwrap(); // We can do this, we made sure it exists.
        Self::parse(input)
    }
}

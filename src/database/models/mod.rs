pub mod name;
pub mod player;
pub mod transaction;
pub mod wallet;

use serde::{Deserialize, Serialize, Serializer};
use surrealdb::sql::Thing;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CountResponse {
    pub count: usize,
}

/// Serde serializer function that converts a record ID from a Thing to a raw string in the format `table:id`.
pub fn serialize_record<S>(record: &Thing, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let raw = record.to_raw();
    s.serialize_str(&raw)
}

/// Serde serializer function that converts a record ID from a Thing to a raw string in the format `table:id`.
pub fn serialize_record_opt<S>(record: &Option<Thing>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match record {
        Some(thing) => s.serialize_str(&thing.to_raw()),
        None => s.serialize_none(),
    }
}

/// Serde serializer function that converts a record ID from a Thing to a raw string in the format `id`.
pub fn serialize_record_id<S>(record: &Thing, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let id = record.id.to_raw();
    s.serialize_str(&id)
}

/// Serde serializer function that converts a record ID from a Thing to a raw string in the format `id`.
pub fn serialize_record_id_opt<S>(record: &Option<Thing>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match record {
        Some(record) => {
            let id = record.id.to_raw();
            s.serialize_str(&id)
        }
        None => s.serialize_none(),
    }
}

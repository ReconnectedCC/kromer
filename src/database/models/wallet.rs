use surrealdb::{
    engine::any::Any,
    sql::{Datetime, Id, Thing},
    Surreal,
};

use super::{serialize_table_opt, CountResponse};
use crate::routes::PaginationParams;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Model {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_table_opt"
    )]
    pub id: Option<Thing>,
    pub address: String,
    pub balance: f64,
    pub created_at: Datetime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>, // We dont want to retrieve the hash all the time.
    pub is_shared: bool,
    pub total_in: f64,
    pub total_out: f64,
}

impl Model {
    /// Get a wallet from its unique ID
    pub async fn get(db: &Surreal<Any>, id: String) -> Result<Option<Model>, surrealdb::Error> {
        let thing: Thing = id.try_into().unwrap();
        let q = "SELECT * FROM wallet WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get a wallet from its unique ID, not including the table part
    pub async fn get_partial(
        db: &Surreal<Any>,
        id: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = Id::from(id);
        let thing = Thing::from(("wallet", id));

        let q = "SELECT * FROM wallet WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get wallet from its address
    pub async fn get_by_address(
        db: &Surreal<Any>,
        address: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let q = "SELECT * from wallet where address = $address;";

        let mut response = db.query(q).bind(("address", address)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get wallet from address, omitting hash and id.
    pub async fn get_by_address_excl(
        db: &Surreal<Any>,
        address: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let q = "SELECT * OMIT id, hash from wallet WHERE address = $address;";

        let mut response = db.query(q).bind(("address", address)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get all wallets, omitting hash and id.
    pub async fn all(
        db: &Surreal<Any>,
        pagination: &PaginationParams,
    ) -> Result<Vec<Model>, surrealdb::Error> {
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q = "SELECT * OMIT id, hash from wallet LIMIT $limit START $offset";

        let mut response = db
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<Model> = response.take(0)?;

        Ok(models)
    }

    /// Verify the password of a wallet, returning the given wallet if it exists.
    pub async fn verify(
        db: &Surreal<Any>,
        password: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let q = "SELECT * FROM wallet WHERE crypto::argon2::compare(hash, $password);";

        let mut response = db.query(q).bind(("password", password)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get wallets sorted by their balance, omitting id and hash
    /// Get all wallets, omitting hash and id.
    pub async fn get_richest(
        db: &Surreal<Any>,
        pagination: &PaginationParams,
    ) -> Result<Vec<Model>, surrealdb::Error> {
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q =
            "SELECT * OMIT id, hash FROM wallet ORDER BY balance DESC LIMIT $limit START $offset";

        let mut response = db
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<Model> = response.take(0)?;

        Ok(models)
    }

    /// Get the total amount of wallets in the database
    pub async fn count(db: &Surreal<Any>) -> Result<usize, surrealdb::Error> {
        let q = "(SELECT count() FROM wallet GROUP BY count)[0] or { count: 0}";

        let mut response = db.query(q).await?;
        let count: Option<CountResponse> = response.take(0)?;
        let count = count.unwrap_or_default(); // Its fine, we make sure we always get a response with the `or` statement in the query.

        Ok(count.count)
    }
}

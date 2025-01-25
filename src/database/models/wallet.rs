use rust_decimal_macros::dec;
use surrealdb::{
    engine::any::Any,
    sql::{Datetime, Id, Thing},
    Surreal,
};

use rust_decimal::Decimal;

use super::{name, transaction};
use super::{serialize_table_opt, CountResponse};
use crate::{models::transactions::AddressTransactionQuery, routes::PaginationParams, utils};

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Model {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_table_opt"
    )]
    pub id: Option<Thing>,
    pub address: String,
    pub balance: Decimal,
    pub created_at: Datetime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>, // We dont want to retrieve the hash all the time.
    pub is_shared: bool,
    pub total_in: Decimal,
    pub total_out: Decimal,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VerifyResponse {
    pub authed: bool,
    pub address: Model,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LookupResponse {
    #[serde(flatten)]
    pub model: Model,
    pub names: usize,
}

impl Model {
    /// Get a wallet from its unique ID
    pub async fn get<S: AsRef<str>>(
        db: &Surreal<Any>,
        id: S,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = id.as_ref();
        let thing: Thing = id.try_into().unwrap();
        let q = "SELECT * FROM wallet WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get a wallet from its unique ID, not including the table part
    pub async fn get_partial<S: AsRef<str>>(
        db: &Surreal<Any>,
        id: S,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = id.as_ref();
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

    /// Create a new wallet with a given address and hash
    pub async fn create(
        db: &Surreal<Any>,
        address: String,
        hash: String,
        initial_bal: Option<Decimal>,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let initial_bal = initial_bal.unwrap_or(dec!(100));
        let q = "RETURN fn::create_wallet_ext($address, $hash, $initial_bal)";

        // NOTE: We could use `.create`, dont know if we should.
        let mut response = db
            .query(q)
            .bind(("address", address))
            .bind(("hash", hash))
            .bind(("initial_bal", initial_bal))
            .await?;
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

    /// Get the total amount of kromer held by wallets
    pub async fn supply(db: &Surreal<Any>) -> Result<Decimal, surrealdb::Error> {
        let q = "RETURN math::sum((SELECT balance FROM wallet).balance);";

        let mut response = db.query(q).await?;
        let supply: Option<Decimal> = response.take(0)?;
        let supply = supply.unwrap_or_else(|| dec!(0));

        Ok(supply)
    }

    #[tracing::instrument(skip(db))]
    pub async fn verify_address<S: AsRef<str> + std::fmt::Debug>(
        db: &Surreal<Any>,
        private_key: S,
    ) -> Result<VerifyResponse, surrealdb::Error> {
        let private_key = private_key.as_ref();

        let address = utils::crypto::make_v2_address(private_key, "k");
        let guh = format!("{address}{private_key}"); // SO CURSED I LOVE IT

        tracing::info!("Authentication attempt on address {address}");

        // TODO: Fix the fucking api definition so it doesnt require an owned copy.
        let result = Model::get_by_address(db, address.clone()).await?;
        let hash = utils::crypto::sha256(&guh);

        if result.is_none() {
            let model = Model::create(db, address, hash, Some(dec!(0))).await?;
            let model = model.expect("for some fucking reason, model is none."); // TODO: Figure out if it actually errors or not.
            tracing::debug!("Created a new wallet with an initial balance of 0");

            return Ok(VerifyResponse {
                authed: true,
                address: model,
            });
        }

        let wallet = result.unwrap(); // It's fine to unwrap here, see above if statement, we checked if it exists or not.
        let pkey = &wallet.hash;
        let authed = *pkey == Some(hash); // Cursed i know, it makes the borrow checker happy.

        if !authed {
            tracing::info!("Someone tried to login to an address they do not own");
        }

        return Ok(VerifyResponse {
            authed,
            address: wallet,
        });
    }

    /// Get all transaction made by an address or send by an address
    pub async fn transactions<S: AsRef<str>>(
        db: &Surreal<Any>,
        address: S,
        query: &AddressTransactionQuery,
    ) -> Result<Vec<transaction::Model>, surrealdb::Error> {
        let address = address.as_ref().to_owned();

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q = "SELECT * FROM transaction WHERE from = $address OR to = $address ORDER BY timestamp DESC LIMIT $limit START $offset;";

        let mut response = db
            .query(q)
            .bind(("address", address))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<transaction::Model> = response.take(0)?;

        Ok(models)
    }

    /// Get all names owned by an address
    pub async fn names<S: AsRef<str>>(
        db: &Surreal<Any>,
        address: S,
        query: &PaginationParams,
    ) -> Result<Vec<name::Model>, surrealdb::Error> {
        let address = address.as_ref().to_owned();

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q = "SELECT * FROM name WHERE owner = $address ORDER BY name ASC LIMIT $limit START $offset;";

        let mut response = db
            .query(q)
            .bind(("address", address))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<name::Model> = response.take(0)?;

        Ok(models)
    }

    /// Lookup a series of wallets by address
    pub async fn lookup(
        db: &Surreal<Any>,
        addresses: Vec<String>,
    ) -> Result<Vec<LookupResponse>, surrealdb::Error> {
        let q = "RETURN fn::get_wallets_with_name_count($addresses)";

        let mut response = db.query(q).bind(("addresses", addresses)).await?;
        let models: Vec<LookupResponse> = response.take(0)?;

        Ok(models)
    }
}

use chrono::Utc;
use surrealdb::{
    engine::any::Any,
    sql::{Datetime, Id, Thing},
    Surreal,
};

use super::{serialize_table_opt, CountResponse};
use crate::{
    database::models::wallet::Model as Wallet,
    errors::krist::{address::AddressError, generic::GenericError, name::NameError, KristError},
    models::names::NameDataUpdateBody,
    routes::PaginationParams,
    utils,
};

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Model {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_table_opt"
    )]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transfered: Option<Datetime>,
    pub name: String,
    #[serde(
        skip_serializing_if = "Option::is_none",
        //serialize_with = "serialize_table_opt"
    )]
    pub original_owner: Option<String>,
    pub owner: String,
    pub registered: Datetime,
    pub updated: Option<Datetime>,
    pub transfered: Option<Datetime>,
    pub a: Option<String>,
    pub unpaid: i64,
}

impl Model {
    /// Get a name from its unique ID
    pub async fn get<S: AsRef<str>>(
        db: &Surreal<Any>,
        id: S,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = id.as_ref();
        let thing: Thing = id.try_into().unwrap();
        let q = "SELECT * FROM name WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get a name from its unique ID, not including the table part
    pub async fn get_partial<S: AsRef<str>>(
        db: &Surreal<Any>,
        id: S,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = id.as_ref();
        let id = Id::from(id);

        let thing = Thing::from(("name", id));

        let q = "SELECT * FROM name WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get name from its name field
    pub async fn get_by_name(
        db: &Surreal<Any>,
        name: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let q = "SELECT * from name where name = $name;";

        let mut response = db.query(q).bind(("name", name)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get name from its name field, omitting id.
    pub async fn get_by_name_excl(
        db: &Surreal<Any>,
        name: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let q = "SELECT * OMIT id from name where name = $name;";

        let mut response = db.query(q).bind(("name", name)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get all names, omitting id.
    pub async fn all(
        db: &Surreal<Any>,
        pagination: &PaginationParams,
    ) -> Result<Vec<Model>, surrealdb::Error> {
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q = "SELECT * OMIT id from name LIMIT $limit START $offset";

        let mut response = db
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<Model> = response.take(0)?;

        Ok(models)
    }

    pub async fn all_unpaid(
        db: &Surreal<Any>,
        pagination: &PaginationParams,
    ) -> Result<Vec<Model>, surrealdb::Error> {
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q = "SELECT * OMIT id from name WHERE unpaid > 0 LIMIT $limit START $offset";

        let mut response = db
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<Model> = response.take(0)?;

        Ok(models)
    }

    /// Get the total amount of names in the database
    pub async fn count(db: &Surreal<Any>) -> Result<usize, surrealdb::Error> {
        let q = "(SELECT count() FROM name GROUP BY count)[0] or { count: 0 }";

        let mut response = db.query(q).await?;
        let count: Option<CountResponse> = response.take(0)?;
        let count = count.unwrap_or_default(); // Its fine, we make sure we always get a response with the `or` statement in the query.

        Ok(count.count)
    }

    pub async fn count_unpaid(db: &Surreal<Any>) -> Result<usize, surrealdb::Error> {
        let q = "(SELECT count() FROM name WHERE unpaid > 0 GROUP BY count)[0] or { count: 0 }";

        let mut response = db.query(q).await?;
        let count: Option<CountResponse> = response.take(0)?;
        let count = count.unwrap_or_default();

        Ok(count.count)
    }

    /// Create a new name with a given owner
    pub async fn register_name(
        db: &Surreal<Any>,
        name: String,
        owner: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let response: Option<Model> = db
            .create("name")
            .content(Model {
                id: None,
                last_transfered: None,
                name,
                original_owner: Some(owner.clone()),
                owner,
                registered: surrealdb::sql::Datetime::from(Utc::now()),
                updated: None,
                transfered: None,
                a: None,
                unpaid: 0,
            })
            .await?;

        // TODO: Add graph relation to this, used for address lookups in Krist

        Ok(response)
    }

    /// Modify the data for a given name
    pub async fn modify_data(
        db: &Surreal<Any>,
        name: String,
        data: Option<String>,
    ) -> Result<bool, surrealdb::Error> {
        let q = "UPDATE name SET a = $data WHERE name = $name;";

        let mut response = db
            .query(q)
            .bind(("data", data))
            .bind(("name", name))
            .await?;
        let result: Option<Model> = response.take(0)?; // I hope this is fine lol

        Ok(result.is_some())
    }

    /// Modify the data for a given name with more checks
    pub async fn ctrl_modify_data(
        db: &Surreal<Any>,
        name: String,
        body: NameDataUpdateBody,
    ) -> Result<Model, KristError> {
        // I have this code so much, i hate you, krist.
        let a_record = body.a;
        if a_record.is_none() {
            return Err(KristError::Generic(GenericError::MissingParameter(
                "a".to_owned(),
            )));
        }
        let a_record = a_record.unwrap();

        if !utils::validation_kromer::is_valid_name(&name, false) {
            return Err(KristError::Generic(GenericError::InvalidParameter(
                "name".to_owned(),
            )));
        }

        if !utils::validation_kromer::is_valid_a_record(&a_record) {
            return Err(KristError::Generic(GenericError::InvalidParameter(
                "a".to_owned(),
            )));
        }

        let name = name.trim().to_lowercase();

        let wallet = Wallet::verify_address(db, body.private_key).await?;
        if !wallet.authed {
            tracing::info!("Auth failed on name update");
            return Err(KristError::Address(AddressError::AuthFailed));
        }

        // I dont like this, please stop borrow checker :sob:
        let model = Model::get_by_name(db, name.clone())
            .await?
            .ok_or_else(|| KristError::Name(NameError::NameNotFound(name.clone())))?;

        if model.owner != wallet.address.address {
            return Err(KristError::Name(NameError::NotNameOwner(name)));
        }

        // Don't do anything if the data is the same
        // I WANT TO STOP CLONING AAAAAAAA
        if model.a == Some(a_record.clone()) {
            return Ok(model);
        }

        // NOTE: We don't create a transaction for name updates, should we?
        let _result = Model::modify_data(db, name, Some(a_record)).await?; // dont ask, idk either

        Ok(model)
    }
}

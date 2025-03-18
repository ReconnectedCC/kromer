use surrealdb::{
    engine::any::Any,
    sql::{Datetime, Id, Thing},
    Surreal,
};

use super::serialize_record_opt;
use crate::routes::PaginationParams;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Model {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_record_opt"
    )]
    pub id: Option<Thing>,
    pub name: String,
    pub joined_at: Datetime,
}

impl Model {
    /// Get a player from its unique ID
    pub async fn get<S: AsRef<str>>(
        db: &Surreal<Any>,
        id: S,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = id.as_ref();
        let thing: Thing = id.try_into().unwrap();
        let q = "SELECT * FROM player WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get a player from its unique ID, not including the table part
    pub async fn get_partial<S: AsRef<str>>(
        db: &Surreal<Any>,
        id: S,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let id = id.as_ref();
        let id = Id::from(id);
        let thing = Thing::from(("player", id));

        let q = "SELECT * FROM player WHERE id = $id;";

        let mut response = db.query(q).bind(("id", thing)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get wallet from its name
    pub async fn get_by_name(
        db: &Surreal<Any>,
        name: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let q = "SELECT * from wallet where name = $name;";

        let mut response = db.query(q).bind(("name", name)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get wallet from its name, omitting id
    pub async fn get_by_name_excl(
        db: &Surreal<Any>,
        name: String,
    ) -> Result<Option<Model>, surrealdb::Error> {
        let q = "SELECT * OMIT id from wallet where name = $name;";

        let mut response = db.query(q).bind(("name", name)).await?;
        let model: Option<Model> = response.take(0)?;

        Ok(model)
    }

    /// Get all players, omitting id.
    pub async fn all(
        db: &Surreal<Any>,
        pagination: &PaginationParams,
    ) -> Result<Vec<Model>, surrealdb::Error> {
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        let limit = limit.clamp(1, 1000);

        let q = "SELECT * OMIT id from wallet LIMIT $limit START $offset";

        let mut response = db
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let models: Vec<Model> = response.take(0)?;

        Ok(models)
    }
}

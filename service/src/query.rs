use ::kromer_economy_entity::{addresses, addresses::Entity as Address};
use sea_orm::*;

pub struct Query;

impl Query {
    /// Fetches a single address from the database by its unique id
    ///
    /// # Arguments
    /// * `db` - The database connection
    /// * `id` - The id of the address to fetch
    ///
    /// # Examples
    /// ```
    /// println!("TODO");
    /// ```
    pub async fn find_address_by_id(db: &DbConn, id: i32) -> Result<Option<addresses::Model>, DbErr> {
        Address::find_by_id(id).one(db).await
    }

    /// Fetches a single address from the database
    ///
    /// # Arguments
    /// * `db` - The database connection
    /// * `address` - The address to fetch
    ///
    /// # Examples
    /// ```
    /// println!("TODO");
    /// ```
    pub async fn find_address(db: &DbConn, address: &str) -> Result<Option<addresses::Model>, DbErr> {
        Address::find().filter(addresses::Column::Address.eq(address)).one(db).await
    }
}
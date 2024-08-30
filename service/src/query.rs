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

    /// Fetches the richest addresses from the database
    ///
    /// # Arguments
    /// * `db` - The database connection
    /// * `limit` - The number of addresses to fetch
    /// * `offset` - The offset for pagination
    ///
    /// # Examples
    /// ```
    /// let limit = 10;
    /// let offset = 0;
    /// let richest_addresses = Query::find_richest_addresses(db, limit, offset).await?;
    ///
    /// for (index, address) in richest_addresses.iter().enumerate() {
    ///     println!("{}. Address: {}, Balance: {}", index, address.address, address.balance);
    /// }
    /// ```
    pub async fn find_richest_addresses(db: &DbConn, limit: u64, offset: u64) -> Result<Vec<addresses::Model>, DbErr> {
        Address::find()
            .order_by_desc(addresses::Column::Balance)
            .limit(limit)
            .offset(offset)
            .all(db)
            .await
    }

    /// Counts the total number of addresses in the database
    ///
    /// # Arguments
    /// * `db` - The database connection
    ///
    /// # Returns
    /// The total number of addresses as a `u64`
    ///
    /// # Examples
    /// ```
    /// let total = Query::count_total_addresses(&db).await?;
    /// println!("Total addresses: {}", total);
    /// ```
    pub async fn count_total_addresses(db: &DbConn) -> Result<u64, DbErr> {
        Address::find().count(db).await
    }
}
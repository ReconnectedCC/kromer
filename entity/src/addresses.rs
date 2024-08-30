//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "addresses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub address: String,
    #[sea_orm(column_type = "Float")]
    pub balance: f32,
    #[sea_orm(column_type = "Float")]
    pub total_in: f32,
    #[sea_orm(column_type = "Float")]
    pub total_out: f32,
    pub first_seen: Date,
    pub private_key: Option<String>,
    pub alert: Option<String>,
    pub locked: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

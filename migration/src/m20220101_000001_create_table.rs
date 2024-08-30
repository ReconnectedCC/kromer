use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Addresses::Table)
                    .if_not_exists()
                    .col(pk_auto(Addresses::Id))
                    .col(string_len(Addresses::Address, 10).unique_key().not_null())
                    .col(float(Addresses::Balance))
                    .col(float(Addresses::TotalIn))
                    .col(float(Addresses::TotalOut))
                    .col(timestamp(Addresses::FirstSeen).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Addresses::PrivateKey)
                            .string_len(64)
                            .null()
                            .take(),
                    )
                    .col(
                        ColumnDef::new(Addresses::Alert)
                            .string_len(1024)
                            .null()
                            .take(),
                    )
                    .col(boolean(Addresses::Locked).default(false))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(pk_auto(Transactions::Id))
                    .col(ColumnDef::new(Transactions::From).string_len(10).null())
                    .col(ColumnDef::new(Transactions::To).string_len(10).null())
                    .col(float(Transactions::Value))
                    .col(timestamp(Transactions::Time).timestamp_with_time_zone())
                    .col(ColumnDef::new(Transactions::Name).string_len(128).null())
                    .col(
                        ColumnDef::new(Transactions::SentMetaname)
                            .string_len(32)
                            .null(),
                    )
                    .col(ColumnDef::new(Transactions::SentName).string_len(64).null())
                    .col(uuid(Transactions::RequestId).unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Names::Table)
                    .if_not_exists()
                    .col(pk_auto(Names::Id))
                    .col(
                        ColumnDef::new(Names::Name)
                            .string_len(64)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Names::Owner).string_len(10).not_null())
                    .col(ColumnDef::new(Names::OriginalOwner).string_len(10).null())
                    .col(timestamp(Names::Registered).timestamp_with_time_zone())
                    .col(ColumnDef::new(Names::Updated).date().null())
                    .col(ColumnDef::new(Names::Transferred).date().null())
                    .col(ColumnDef::new(Names::A).string_len(255).null())
                    .col(float(Names::Unpaid))
                    .to_owned(),
            )
            .await

        // TODO: Indexes
        // TODO: Names table
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Addresses::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Names::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Addresses {
    Table,
    Id,
    Address,
    Balance,
    TotalIn,
    TotalOut,
    FirstSeen,
    PrivateKey,
    Alert,
    Locked,
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    Id,
    From,
    To,
    Value,
    Time,
    Name,
    SentMetaname,
    SentName,
    RequestId,
}

#[derive(DeriveIden)]
enum Names {
    Table,
    Id,
    Name,
    Owner,
    OriginalOwner,
    Registered,
    Updated,
    Transferred,
    A,
    Unpaid,
}

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

impl Migration {
    async fn create_transaction_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("transactions_from")
                    .table(Transactions::Table)
                    .col(Transactions::From)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("transactions_from")
                    .table(Transactions::Table)
                    .col(Transactions::To)
                    .to_owned(),
            )
            .await?;

        // transactions_op

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("transactions_name")
                    .table(Transactions::Table)
                    .col(Transactions::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("transactions_sent_name")
                    .table(Transactions::Table)
                    .col(Transactions::SentName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("transactions_sent_metaname")
                    .table(Transactions::Table)
                    .col(Transactions::SentMetaname)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("transactions_sent_metaname_sent_name")
                    .table(Transactions::Table)
                    .col(Transactions::SentMetaname)
                    .col(Transactions::SentName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("transactions_metadata")
                    .table(Transactions::Table)
                    .col(Transactions::Metadata)
                    .to_owned(),
            )
            .await
    }

    async fn create_address_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("addresses_address_unique")
                    .table(Addresses::Table)
                    .col(Addresses::Address)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn create_name_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("names_name_unique")
                    .table(Names::Table)
                    .col(Names::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("names_owner")
                    .table(Names::Table)
                    .col(Names::Owner)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("names_original_owner")
                    .table(Names::Table)
                    .col(Names::OriginalOwner)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("names_unpaid")
                    .table(Names::Table)
                    .col(Names::Unpaid)
                    .to_owned(),
            )
            .await
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Addresses::Table)
                    .if_not_exists()
                    .col(pk_auto(Addresses::Id))
                    .col(char_len(Addresses::Address, 10).unique_key().not_null())
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

        Migration::create_address_indexes(manager).await?;

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
                    .col(
                        ColumnDef::new(Transactions::Metadata)
                            .string_len(512)
                            .null(),
                    )
                    .col(uuid(Transactions::RequestId).unique_key())
                    .to_owned(),
            )
            .await?;

        Migration::create_transaction_indexes(manager).await?;

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
                    .col(ColumnDef::new(Names::Owner).char_len(10).not_null())
                    .col(ColumnDef::new(Names::OriginalOwner).char_len(10).null())
                    .col(timestamp(Names::Registered).timestamp_with_time_zone())
                    .col(ColumnDef::new(Names::Updated).date().null())
                    .col(ColumnDef::new(Names::Transferred).date().null())
                    .col(ColumnDef::new(Names::A).string_len(255).null())
                    .col(float(Names::Unpaid))
                    .to_owned(),
            )
            .await?;

        Migration::create_name_indexes(manager).await

        // TODO: Indexes
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
    Metadata, // Called `op` in Krist.
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

use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    cli::run_cli(kromer_economy_migration::Migrator).await;
}

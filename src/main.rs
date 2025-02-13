use std::env;
use std::sync::Arc;

use actix_web::{middleware, web, App, HttpServer};

use kromer::websockets::WebSocketServer;
use surrealdb::opt::auth::Root;
use surrealdb_migrations::MigrationRunner;

use kromer::database::db::{ConnectionOptions, Database};
use kromer::{errors::KromerError, routes, AppState};

#[actix_web::main]
async fn main() -> Result<(), KromerError> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    // TODO: Factor the database stuff out to a function.
    let surreal_endpoint = env::var("SURREAL_URL").expect("SURREAL_URL is not set in .env file");
    let surreal_user = env::var("SURREAL_USER").expect("SURREAL_USER is not set in .env file");
    let surreal_password =
        env::var("SURREAL_PASSWORD").expect("SURREAL_PASSWORD is not set in .env file");
    let surreal_namespace =
        env::var("SURREAL_NAMESPACE").expect("SURREAL_NAMESPACE is not set in .env file");
    let surreal_database =
        env::var("SURREAL_DATABASE").expect("SURREAL_DATABASE is not set in .env file");

    let connect_options = ConnectionOptions {
        namespace: &surreal_namespace,
        database: &surreal_database,
        credentials: Root {
            username: &surreal_user,
            password: &surreal_password,
        },
    };

    let db = Database::connect(&surreal_endpoint, &connect_options).await?;

    // Perform migrations
    MigrationRunner::new(&db)
        .up()
        .await
        .expect("Failed to apply SurrealDB Migrations");

    let db_arc = Arc::new(db);

    Database::monitor_db_connection(db_arc.clone());

    let krist_ws_server = WebSocketServer::new();

    let state = web::Data::new(AppState { db: db_arc });

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(
                web::FormConfig::default()
                    .error_handler(|err, _req| KromerError::Validation(err.to_string()).into()),
            )
            .app_data(
                web::QueryConfig::default()
                    .error_handler(|err, _req| KromerError::Validation(err.to_string()).into()),
            )
            .app_data(
                web::PathConfig::default()
                    .error_handler(|err, _req| KromerError::Validation(err.to_string()).into()),
            )
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _req| KromerError::Validation(err.to_string()).into()),
            )
            .app_data(state.clone())
            .app_data(web::Data::new(krist_ws_server.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .configure(kromer::routes::config)
            .default_service(web::route().to(routes::not_found::not_found))
    })
    .bind(&server_url)?
    .run();

    http_server.await?;

    Ok(())
}

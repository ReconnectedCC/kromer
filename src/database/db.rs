use crate::errors::KromerError;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::{
    engine::any::{self, Any},
    opt::auth::Root,
    Surreal,
};
use tokio::spawn;
use tokio::time::{sleep, timeout};

pub struct Database;

pub struct ConnectionOptions<'a> {
    pub namespace: &'a str,
    pub database: &'a str,
    pub credentials: Root<'a>,
}

impl Database {
    pub async fn connect<'a>(
        endpoint: &'a str,
        options: &ConnectionOptions<'a>,
    ) -> Result<Surreal<Any>, KromerError> {
        let db = any::connect(endpoint).await?;

        db.signin(options.credentials).await?;

        db.use_ns(options.namespace)
            .use_db(options.database)
            .await?;

        Ok(db)
    }

    pub fn monitor_db_connection(db_arc: Arc<Surreal<Any>>) {
        tracing::debug!("Monitoring DB connection");

        spawn(async {
            inner_monitor_db_connection(db_arc).await;
        });
    }
}

async fn inner_monitor_db_connection(db_arc: Arc<Surreal<Any>>) {
    let health_check_timeout = Duration::from_secs(5);
    let health_check_interval = Duration::from_secs(10);

    loop {
        match timeout(health_check_timeout, db_arc.health()).await {
            Ok(health_result) => {
                if let Err(e) = health_result {
                    tracing::error!("Database connection error: {}", e);
                    std::process::exit(0x102);
                }
            }
            Err(_) => {
                tracing::error!("Database health check timed out (connection likely dead)");
                std::process::exit(0x101);
            }
        }

        sleep(health_check_interval).await;
    }
}

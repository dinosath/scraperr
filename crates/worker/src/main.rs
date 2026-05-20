use anyhow::Result;
use sea_orm::Database;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./scraperr.db".to_string());
    let poll_interval: u64 = std::env::var("WORKER_POLL_INTERVAL_SECS")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);

    let db = Database::connect(&database_url).await?;
    tracing::info!("Standalone worker started");

    // Spawn cron scheduler
    let cron_db = db.clone();
    tokio::spawn(async move {
        if let Err(e) = scraperr_worker::run_cron_scheduler(cron_db).await {
            tracing::error!("Cron scheduler error: {e}");
        }
    });

    // Run polling loop (blocks forever)
    scraperr_worker::run_polling_loop(db, poll_interval).await;

    Ok(())
}

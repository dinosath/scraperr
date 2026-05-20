mod config;
mod error;
mod extractors;
mod grpc;
mod handlers;
mod middleware;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::AppConfig;
use middleware::oidc::OidcConfig;

/// Shared application state available to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
}

// Allow extracting DatabaseConnection from AppState
impl axum::extract::FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

// Allow extracting AppConfig from AppState
impl axum::extract::FromRef<AppState> for AppConfig {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,scraperr_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env()?;
    tracing::info!("Starting Scraperr API server...");

    // Connect to database
    let db = Database::connect(&config.database_url).await?;
    tracing::info!("Connected to database");

    // Run migrations
    scraperr_migration::Migrator::up(&db, None).await?;
    tracing::info!("Migrations complete");

    // Spawn worker background tasks
    let worker_poll_interval: u64 = std::env::var("WORKER_POLL_INTERVAL_SECS")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);

    // Background polling loop for queued jobs
    let worker_db = db.clone();
    tokio::spawn(async move {
        scraperr_worker::run_polling_loop(worker_db, worker_poll_interval).await;
    });

    // Cron scheduler
    let cron_db = db.clone();
    tokio::spawn(async move {
        if let Err(e) = scraperr_worker::run_cron_scheduler(cron_db).await {
            tracing::error!("Cron scheduler error: {e}");
        }
    });
    tracing::info!("Worker background tasks started");

    // Set up OIDC
    let oidc_config = Arc::new(OidcConfig {
        issuer: config.oidc_issuer.clone(),
        audience: config.oidc_audience.clone(),
        jwks_uri: config.oidc_jwks_uri.clone(),
        jwks: Arc::new(tokio::sync::RwLock::new(None)),
    });

    // Attempt to fetch JWKS (non-fatal on startup if IdP isn't ready)
    if let Err(e) = middleware::oidc::refresh_jwks(&oidc_config).await {
        tracing::warn!("Failed to fetch JWKS on startup (will retry): {}", e);
    }

    // Spawn a background task to periodically refresh JWKS
    {
        let oidc = oidc_config.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                if let Err(e) = middleware::oidc::refresh_jwks(&oidc).await {
                    tracing::warn!("JWKS refresh failed: {}", e);
                }
            }
        });
    }

    let state = AppState {
        db: db.clone(),
        config: config.clone(),
    };

    let app = Router::new()
        // Auth
        .route("/api/auth/check", get(handlers::auth::check_auth))
        .route("/api/auth/me", get(handlers::auth::get_me))
        // Jobs
        .route(
            "/api/submit-scrape-job",
            post(handlers::jobs::submit_scrape_job),
        )
        .route(
            "/api/retrieve-scrape-jobs",
            post(handlers::jobs::retrieve_scrape_jobs),
        )
        .route("/api/job/{id}", get(handlers::jobs::get_job))
        .route(
            "/api/trigger-job/{id}",
            post(handlers::jobs::trigger_job),
        )
        .route("/api/update", post(handlers::jobs::update_jobs))
        .route(
            "/api/delete-scrape-jobs",
            post(handlers::jobs::delete_scrape_jobs),
        )
        .route("/api/download", post(handlers::jobs::download))
        // Cron jobs
        .route(
            "/api/schedule-cron-job",
            post(handlers::cron::schedule_cron_job),
        )
        .route(
            "/api/retrieve-cron-jobs",
            get(handlers::cron::retrieve_cron_jobs),
        )
        .route(
            "/api/delete-cron-job",
            post(handlers::cron::delete_cron_job),
        )
        // AI
        .route("/api/ai", post(handlers::ai::ai_chat))
        .route("/api/ai/check", get(handlers::ai::check_ai))
        // Statistics
        .route(
            "/api/statistics/get-average-element-per-link",
            get(handlers::stats::avg_elements_per_link),
        )
        .route(
            "/api/statistics/get-average-jobs-per-day",
            get(handlers::stats::avg_jobs_per_day),
        )
        // Layers
        .layer(Extension(oidc_config))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Spawn gRPC server
    let grpc_addr: std::net::SocketAddr =
        format!("{}:{}", config.host, config.grpc_port).parse()?;
    let grpc_db = db.clone();
    let grpc_config = config.clone();
    tokio::spawn(async move {
        use grpc::proto::auth::auth_service_server::AuthServiceServer;
        use grpc::proto::jobs::job_service_server::JobServiceServer;
        use grpc::services::auth::AuthServiceImpl;
        use grpc::services::jobs::JobServiceImpl;

        // Set up gRPC health check service (following tonic health example)
        let (health_reporter, health_service) =
            tonic_health::server::health_reporter();

        let auth_svc = AuthServiceServer::new(AuthServiceImpl {
            registration_enabled: grpc_config.registration_enabled,
            recordings_enabled: grpc_config.recordings_enabled,
        });
        let job_svc = JobServiceServer::with_interceptor(
            JobServiceImpl { db: grpc_db },
            grpc::interceptor::check_auth,
        );

        // Mark services as serving
        health_reporter
            .set_serving::<AuthServiceServer<AuthServiceImpl>>()
            .await;
        health_reporter
            .set_serving::<JobServiceServer<JobServiceImpl>>()
            .await;

        tracing::info!("gRPC server listening on {}", grpc_addr);
        if let Err(e) = tonic::transport::Server::builder()
            .add_service(health_service)
            .add_service(auth_svc)
            .add_service(job_svc)
            .serve(grpc_addr)
            .await
        {
            tracing::error!("gRPC server error: {}", e);
        }
    });

    // Start HTTP server
    let http_addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&http_addr).await?;
    tracing::info!("HTTP server listening on {}", http_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

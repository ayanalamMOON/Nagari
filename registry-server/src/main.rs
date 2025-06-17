use axum::{routing::{get, post, put, delete}, Router};
use clap::Parser;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod auth;
mod config;
mod db;
mod services;
mod storage;
mod middleware;

use config::Config;
use db::Database;
use storage::StorageBackend;
use services::{package_service::PackageService, user_service::UserService, auth_service::AuthService};
use api::handlers;

#[derive(Parser)]
#[command(name = "nagari-registry")]
#[command(about = "Nagari Package Registry Server")]
pub struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Server host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Server port
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Database URL
    #[arg(long)]
    database_url: Option<String>,

    /// Enable development mode
    #[arg(long)]
    dev: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub storage: StorageBackend,
    pub package_service: PackageService,
    pub user_service: UserService,
    pub auth_service: AuthService,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nagari_registry=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load(args.config.as_deref()).await?;

    // Initialize database
    let database_url = args.database_url
        .or(config.database.url.clone())
        .ok_or_else(|| anyhow::anyhow!("Database URL not provided"))?;

    let db = Database::connect(&database_url).await?;
    db.migrate().await?;

    // Initialize storage backend
    let storage = StorageBackend::new(&config.storage).await?;    // Initialize services
    let package_service = PackageService::new(db.pool.clone());
    let user_service = UserService::new(db.pool.clone());
    let auth_service = AuthService::new(config.auth.clone());

    // Create application state
    let state = AppState {
        db,
        storage,
        package_service,
        user_service,
        auth_service,
        config: config.clone(),
    };

    // Build the application
    let app = create_app(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("Registry server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        // Package endpoints
        .route("/packages", get(handlers::packages::list_packages))
        .route("/packages", post(handlers::packages::publish_package))
        .route("/packages/:name", get(handlers::packages::get_package))
        .route("/packages/:name", delete(handlers::packages::delete_package))
        .route("/packages/:name/:version", get(handlers::packages::get_package_version))
        .route("/packages/:name/:version", delete(handlers::packages::delete_package_version))
        .route("/packages/:name/:version/download", get(handlers::packages::download_package))

        // User endpoints
        .route("/users/register", post(handlers::users::register))
        .route("/users/login", post(handlers::users::login))
        .route("/users/profile", get(handlers::users::get_profile))
        .route("/users/profile", put(handlers::users::update_profile))

        // Search endpoints
        .route("/search", get(handlers::search::search_packages))

        // Stats endpoints
        .route("/stats", get(handlers::stats::get_stats))
        .route("/packages/:name/stats", get(handlers::stats::get_package_stats))

        // Health check
        .route("/health", get(handlers::health::health_check))

        // API documentation
        .route("/docs", get(handlers::docs::api_docs))

        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(middleware::auth::AuthLayer::new())
        )
        .with_state(state)
}

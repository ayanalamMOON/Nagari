use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub mod handlers;

/// Create the API router with all endpoints
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/admin/cache", delete(clear_cache))
        .route("/admin/settings", put(update_settings))
        .route("/admin/maintenance", post(trigger_maintenance))
        .nest("/api/v1", api_v1_routes())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Admin endpoint to clear cache
async fn clear_cache() -> &'static str {
    "Cache cleared successfully"
}

/// Admin endpoint to update settings
async fn update_settings() -> &'static str {
    "Settings updated successfully"
}

/// Admin endpoint to trigger maintenance
async fn trigger_maintenance() -> &'static str {
    "Maintenance triggered successfully"
}

/// API v1 routes
fn api_v1_routes() -> Router {
    Router::new()
        .nest("/packages", handlers::packages::routes())
        .nest("/auth", handlers::auth::routes())
        .nest("/users", handlers::users::routes())
}

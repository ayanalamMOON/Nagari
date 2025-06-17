use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub mod handlers;

/// Create the API router with all endpoints
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1_routes())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// API v1 routes
fn api_v1_routes() -> Router {
    Router::new()
        .nest("/packages", handlers::packages::routes())
        .nest("/auth", handlers::auth::routes())
        .nest("/users", handlers::users::routes())
}

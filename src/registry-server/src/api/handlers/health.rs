use axum::{
    routing::get,
    Router,
};

/// Health check routes
pub fn routes() -> Router {
    Router::new()
        .route("/", get(health_check))
}

/// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

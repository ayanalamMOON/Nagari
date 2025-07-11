use axum::{
    routing::get,
    Router,
};

/// Documentation routes
pub fn routes() -> Router {
    Router::new()
        .route("/", get(api_docs))
}

/// API documentation endpoint
pub async fn api_docs() -> &'static str {
    "API Documentation"
}

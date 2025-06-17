use axum::{
    routing::get,
    Router,
};

/// Statistics routes
pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_stats))
        .route("/packages/:name", get(get_package_stats))
}

/// Get general statistics
pub async fn get_stats() -> &'static str {
    "Get stats"
}

/// Get package statistics
pub async fn get_package_stats() -> &'static str {
    "Get package stats"
}

use axum::{
    routing::get,
    Router,
};

/// Search routes
pub fn routes() -> Router {
    Router::new()
        .route("/", get(search_packages))
}

/// Search packages
pub async fn search_packages() -> &'static str {
    "Search packages"
}

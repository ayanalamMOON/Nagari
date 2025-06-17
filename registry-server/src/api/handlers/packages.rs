use axum::{
    routing::{get, post, put, delete},
    Router,
};

/// Package management routes
pub fn routes() -> Router {
    Router::new()
        .route("/", get(list_packages).post(publish_package))
        .route("/:name", get(get_package))
        .route("/:name/:version", get(get_package_version))
        .route("/:name/:version/download", get(download_package))
        .route("/:name/owners", get(get_package_owners).post(add_package_owner))
        .route("/:name/owners/:username", delete(remove_package_owner))
}

/// List packages
pub async fn list_packages() -> &'static str {
    "List packages"
}

/// Publish a new package
pub async fn publish_package() -> &'static str {
    "Publish package"
}

/// Get package information
pub async fn get_package() -> &'static str {
    "Get package"
}

/// Get specific package version
pub async fn get_package_version() -> &'static str {
    "Get package version"
}

/// Download package
pub async fn download_package() -> &'static str {
    "Download package"
}

/// Delete package
pub async fn delete_package() -> &'static str {
    "Delete package"
}

/// Delete package version
pub async fn delete_package_version() -> &'static str {
    "Delete package version"
}

/// Get package owners
async fn get_package_owners() -> &'static str {
    "Get package owners"
}

/// Add package owner
async fn add_package_owner() -> &'static str {
    "Add package owner"
}

/// Remove package owner
async fn remove_package_owner() -> &'static str {
    "Remove package owner"
}

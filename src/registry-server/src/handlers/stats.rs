use axum::{
    extract::{Path, State},
    response::Json,
};

use crate::{
    AppState,
    models::{PackageStats, PackageDetailStats},
    error::Result,
};

pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<PackageStats>> {
    let stats = state.package_service.get_registry_stats().await?;
    Ok(Json(stats))
}

pub async fn get_package_stats(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<PackageDetailStats>> {
    let stats = state.package_service.get_package_stats(&name).await?;
    Ok(Json(stats))
}

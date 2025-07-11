use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    models::*,
    error::{AppError, Result},
};

#[derive(Debug, Deserialize)]
pub struct ListPackagesQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

pub async fn list_packages(
    State(state): State<AppState>,
    Query(query): Query<ListPackagesQuery>,
) -> Result<Json<PackageSearchResult>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100).max(1);
    let sort = query.sort.unwrap_or_else(|| "updated_at".to_string());
    let order = query.order.unwrap_or_else(|| "desc".to_string());

    let result = state.package_service.list_packages(page, per_page, &sort, &order).await?;
    Ok(Json(result))
}

pub async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Package>> {
    let package = state.package_service.get_package(&name).await?
        .ok_or(AppError::NotFound("Package not found".to_string()))?;

    Ok(Json(package))
}

pub async fn get_package_version(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Json<PackageVersion>> {
    let package_version = state.package_service.get_package_version(&name, &version).await?
        .ok_or(AppError::NotFound("Package version not found".to_string()))?;

    Ok(Json(package_version))
}

pub async fn publish_package(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<Json<PackageVersion>> {
    let mut metadata: Option<PackageMetadata> = None;
    let mut tarball: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Invalid multipart data: {}", e))
    })? {
        let name = field.name().unwrap_or_default();

        match name {
            "metadata" => {
                let data = field.bytes().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read metadata: {}", e))
                })?;
                metadata = Some(serde_json::from_slice(&data).map_err(|e| {
                    AppError::BadRequest(format!("Invalid metadata JSON: {}", e))
                })?);
            }
            "tarball" => {
                tarball = Some(field.bytes().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read tarball: {}", e))
                })?.to_vec());
            }
            _ => continue,
        }
    }

    let metadata = metadata.ok_or_else(|| {
        AppError::BadRequest("Missing metadata".to_string())
    })?;

    let tarball = tarball.ok_or_else(|| {
        AppError::BadRequest("Missing tarball".to_string())
    })?;

    let package_version = state.package_service
        .publish_package(user.id, metadata, tarball)
        .await?;

    Ok(Json(package_version))
}

pub async fn download_package(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, String)>,
) -> Result<axum::response::Response> {
    let package_version = state.package_service.get_package_version(&name, &version).await?
        .ok_or(AppError::NotFound("Package version not found".to_string()))?;

    // Increment download count
    state.package_service.increment_download_count(&name, &version).await?;

    // Get tarball from storage
    let tarball_data = state.storage.get_package_tarball(&name, &version).await?;

    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header("Content-Disposition", format!("attachment; filename=\"{}-{}.tgz\"", name, version))
        .header("Content-Length", tarball_data.len())
        .body(axum::body::Body::from(tarball_data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

pub async fn delete_package(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    // Check if user owns the package or is admin
    let package = state.package_service.get_package(&name).await?
        .ok_or(AppError::NotFound("Package not found".to_string()))?;

    if package.owner_id != user.id && !user.is_admin {
        return Err(AppError::Forbidden("Not authorized to delete this package".to_string()));
    }

    state.package_service.delete_package(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_package_version(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((name, version)): Path<(String, String)>,
) -> Result<StatusCode> {
    // Check if user owns the package or is admin
    let package = state.package_service.get_package(&name).await?
        .ok_or(AppError::NotFound("Package not found".to_string()))?;

    if package.owner_id != user.id && !user.is_admin {
        return Err(AppError::Forbidden("Not authorized to delete this package version".to_string()));
    }

    state.package_service.delete_package_version(&name, &version).await?;
    Ok(StatusCode::NO_CONTENT)
}

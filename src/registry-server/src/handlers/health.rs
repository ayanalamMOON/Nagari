use axum::{
    extract::State,
    response::Json,
};

use crate::{
    AppState,
    models::HealthStatus,
    error::Result,
};

pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthStatus>> {
    let database_status = match state.db.health_check().await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };

    let storage_status = match state.storage.health_check().await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };

    let status = if database_status == "healthy" && storage_status == "healthy" {
        "healthy"
    } else {
        "unhealthy"
    };

    let health = HealthStatus {
        status: status.to_string(),
        database: database_status,
        storage: storage_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: 0, // TODO: Track actual uptime
    };

    Ok(Json(health))
}

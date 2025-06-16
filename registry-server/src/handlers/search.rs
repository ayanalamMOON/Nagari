use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::Deserialize;

use crate::{
    AppState,
    models::PackageSearchResult,
    error::Result,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sort: Option<String>,
}

pub async fn search_packages(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<PackageSearchResult>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100).max(1);
    let sort = query.sort.unwrap_or_else(|| "relevance".to_string());

    let result = state.package_service.search_packages(&query.q, page, per_page, &sort).await?;
    Ok(Json(result))
}

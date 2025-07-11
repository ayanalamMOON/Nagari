use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_count: i64,
    pub latest_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PackageVersion {
    pub id: Uuid,
    pub package_id: Uuid,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub main: Option<String>,
    pub files: Vec<String>,
    pub readme: Option<String>,
    pub changelog: Option<String>,
    pub tarball_url: String,
    pub tarball_size: i64,
    pub tarball_sha256: String,
    pub published_at: DateTime<Utc>,
    pub download_count: i64,
    pub yanked: bool,
    pub yanked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub avatar_url: Option<String>,
    pub password_hash: String,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_admin: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub package: PackageMetadata,
    pub tarball: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub main: Option<String>,
    pub files: Vec<String>,
    pub readme: Option<String>,
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSearchResult {
    pub packages: Vec<PackageSearchItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSearchItem {
    pub name: String,
    pub description: Option<String>,
    pub latest_version: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub download_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStats {
    pub total_packages: i64,
    pub total_versions: i64,
    pub total_downloads: i64,
    pub recent_packages: Vec<PackageSearchItem>,
    pub popular_packages: Vec<PackageSearchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDetailStats {
    pub download_count: i64,
    pub version_count: i64,
    pub latest_version: Option<String>,
    pub daily_downloads: Vec<DailyDownload>,
    pub versions: Vec<VersionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyDownload {
    pub date: String,
    pub downloads: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionStats {
    pub version: String,
    pub downloads: i64,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: String,
    pub storage: String,
    pub version: String,
    pub uptime: u64,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            bio: user.bio,
            website: user.website,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
        }
    }
}

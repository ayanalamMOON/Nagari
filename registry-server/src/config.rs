use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub registry: RegistryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: Option<String>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connect_timeout: Option<u64>,
    pub idle_timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: String, // "filesystem", "s3", etc.
    pub filesystem: Option<FilesystemConfig>,
    pub s3: Option<S3Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    pub root_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64, // in seconds
    pub bcrypt_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub max_package_size: u64, // in bytes
    pub allowed_file_types: Vec<String>,
    pub require_auth_for_publish: bool,
    pub require_email_verification: bool,
}

impl Config {
    pub async fn load(config_path: Option<&str>) -> anyhow::Result<Self> {
        // Try to load from file first
        if let Some(path) = config_path {
            if Path::new(path).exists() {
                let content = tokio::fs::read_to_string(path).await?;
                return Ok(toml::from_str(&content)?);
            }
        }

        // Try default config file
        if Path::new("config.toml").exists() {
            let content = tokio::fs::read_to_string("config.toml").await?;
            return Ok(toml::from_str(&content)?);
        }

        // Fall back to defaults with environment variables
        Ok(Self::default_with_env())
    }

    pub fn default_with_env() -> Self {
        Self {
            server: ServerConfig {
                host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(3000),
                workers: std::env::var("WORKERS")
                    .ok()
                    .and_then(|w| w.parse().ok()),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL").ok(),
                max_connections: std::env::var("DB_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|c| c.parse().ok()),
                min_connections: std::env::var("DB_MIN_CONNECTIONS")
                    .ok()
                    .and_then(|c| c.parse().ok()),
                connect_timeout: std::env::var("DB_CONNECT_TIMEOUT")
                    .ok()
                    .and_then(|t| t.parse().ok()),
                idle_timeout: std::env::var("DB_IDLE_TIMEOUT")
                    .ok()
                    .and_then(|t| t.parse().ok()),
            },
            storage: StorageConfig {
                backend: std::env::var("STORAGE_BACKEND").unwrap_or_else(|_| "filesystem".to_string()),
                filesystem: Some(FilesystemConfig {
                    root_path: std::env::var("STORAGE_ROOT").unwrap_or_else(|_| "./storage".to_string()),
                }),
                s3: if std::env::var("S3_ENDPOINT").is_ok() {
                    Some(S3Config {
                        endpoint: std::env::var("S3_ENDPOINT").unwrap(),
                        bucket: std::env::var("S3_BUCKET").unwrap(),
                        access_key: std::env::var("S3_ACCESS_KEY").unwrap(),
                        secret_key: std::env::var("S3_SECRET_KEY").unwrap(),
                        region: std::env::var("S3_REGION").ok(),
                    })
                } else {
                    None
                },
            },
            auth: AuthConfig {
                jwt_secret: std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "your-secret-key".to_string()),
                jwt_expiration: std::env::var("JWT_EXPIRATION")
                    .ok()
                    .and_then(|e| e.parse().ok())
                    .unwrap_or(86400), // 24 hours
                bcrypt_cost: std::env::var("BCRYPT_COST")
                    .ok()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(12),
            },
            registry: RegistryConfig {
                max_package_size: std::env::var("MAX_PACKAGE_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100 * 1024 * 1024), // 100MB
                allowed_file_types: vec![
                    "application/gzip".to_string(),
                    "application/x-tar".to_string(),
                    "application/x-gzip".to_string(),
                ],
                require_auth_for_publish: std::env::var("REQUIRE_AUTH_FOR_PUBLISH")
                    .ok()
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(true),
                require_email_verification: std::env::var("REQUIRE_EMAIL_VERIFICATION")
                    .ok()
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(false),
            },
        }
    }
}

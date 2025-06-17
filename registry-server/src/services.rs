use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

/// Package registry services
pub mod package_service {
    use super::*;
    use crate::db::{DatabasePool, packages::Package};
    use uuid::Uuid;
    use chrono::Utc;

    #[derive(Debug, Clone)]
    pub struct PackageService {
        pub db_pool: DatabasePool,
    }

    impl PackageService {
        pub fn new(db_pool: DatabasePool) -> Self {
            Self { db_pool }
        }

        pub async fn publish_package(&self, req: PublishRequest) -> Result<Package> {
            let package = Package {
                id: Uuid::new_v4(),
                name: req.name,
                description: req.description,
                version: req.version,
                author_id: req.author_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            crate::db::packages::create_package(&self.db_pool, &package).await?;
            Ok(package)
        }

        pub async fn get_package(&self, name: &str) -> Result<Option<Package>> {
            crate::db::packages::find_package_by_name(&self.db_pool, name).await
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct PublishRequest {
        pub name: String,
        pub description: Option<String>,
        pub version: String,
        pub author_id: Uuid,
    }
}

/// User management services
pub mod user_service {
    use super::*;
    use crate::{auth::User, db::DatabasePool};
    use uuid::Uuid;
    use chrono::Utc;
    use bcrypt::{hash, verify, DEFAULT_COST};

    #[derive(Debug, Clone)]
    pub struct UserService {
        pub db_pool: DatabasePool,
    }

    impl UserService {
        pub fn new(db_pool: DatabasePool) -> Self {
            Self { db_pool }
        }

        pub async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
            let password_hash = hash(req.password, DEFAULT_COST)?;

            let user = User {
                id: Uuid::new_v4(),
                username: req.username,
                email: req.email,
                password_hash,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                is_active: true,
            };

            crate::db::users::create_user(&self.db_pool, &user).await?;
            Ok(user)
        }

        pub async fn authenticate(&self, username: &str, password: &str) -> Result<Option<User>> {
            if let Some(user) = crate::db::users::find_user_by_username(&self.db_pool, username).await? {
                if verify(password, &user.password_hash)? {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct CreateUserRequest {
        pub username: String,
        pub email: String,
        pub password: String,
    }
}

/// Authentication services
pub mod auth_service {
    use super::*;
    use crate::auth::{TokenResponse, Claims};
    use crate::config::AuthConfig;
    use jsonwebtoken::{encode, Header, EncodingKey};
    use chrono::{Utc, Duration};

    #[derive(Debug, Clone)]
    pub struct AuthService {
        pub config: AuthConfig,
    }

    impl AuthService {
        pub fn new(config: AuthConfig) -> Self {
            Self { config }
        }

        pub fn generate_token(&self, user_id: &str) -> Result<TokenResponse> {
            let now = Utc::now();
            let expires_in = Duration::hours(24);
            let exp = (now + expires_in).timestamp() as usize;

            let claims = Claims {
                sub: user_id.to_string(),
                exp,
                iat: now.timestamp() as usize,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            )?;

            Ok(TokenResponse {
                access_token: token.clone(),
                refresh_token: token, // In production, generate a separate refresh token
                token_type: "Bearer".to_string(),
                expires_in: expires_in.num_seconds(),
            })
        }
    }
}

/// Service response structure that uses Serialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ServiceResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Operation completed successfully".to_string(),
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message,
        }
    }
}

/// Configuration structure that uses PathBuf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub storage_path: PathBuf,
    pub cache_path: PathBuf,
    pub temp_path: PathBuf,
}

impl ServiceConfig {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            storage_path: base_path.join("storage"),
            cache_path: base_path.join("cache"),
            temp_path: base_path.join("temp"),
        }
    }
}

/// Registry services manager that uses all service imports  
pub struct RegistryServices {
    config: ServiceConfig,
}

impl RegistryServices {
    pub fn new(config: ServiceConfig) -> Result<Self> {
        Ok(Self {
            config,
        })
    }
    
    pub fn create_package_service(&self, db_pool: crate::db::DatabasePool) -> PackageService {
        PackageService::new(db_pool)
    }
    
    pub fn create_user_service(&self, db_pool: crate::db::DatabasePool) -> UserService {
        UserService::new(db_pool)
    }
      pub fn create_auth_service(&self, auth_config: auth_service::AuthConfig) -> AuthService {
        AuthService::new(auth_config)
    }
    
    pub fn config(&self) -> &ServiceConfig {
        &self.config
    }
}

// Re-export services for easier importing
pub use package_service::PackageService;
pub use user_service::UserService;
pub use auth_service::AuthService;

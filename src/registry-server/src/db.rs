use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;

/// Database connection pool
pub type DatabasePool = PgPool;

/// Database wrapper
#[derive(Debug, Clone)]
pub struct Database {
    pub pool: DatabasePool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        // TODO: Run database migrations
        Ok(())
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

/// Initialize database connection pool
pub async fn init_pool(config: &DatabaseConfig) -> Result<DatabasePool> {
    let pool = PgPool::connect(&config.url).await?;
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(_pool: &DatabasePool) -> Result<()> {
    // TODO: Add actual migrations
    // sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

/// Database operations for users
pub mod users {
    use super::*;
    use crate::auth::User;
    use chrono::Utc;    pub async fn create_user(pool: &DatabasePool, user: &User) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.is_active)
        .execute(pool)
        .await?;
        Ok(())
    }    pub async fn find_user_by_username(pool: &DatabasePool, username: &str) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at, updated_at, is_active
             FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }
}

/// Database operations for packages
pub mod packages {    use super::*;
    use serde::{Deserialize, Serialize};
    use chrono::{DateTime, Utc};
    use sqlx::FromRow;

    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Package {
        pub id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub version: String,
        pub author_id: Uuid,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }    pub async fn create_package(pool: &DatabasePool, package: &Package) -> Result<()> {
        sqlx::query(
            "INSERT INTO packages (id, name, description, version, author_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(package.id)
        .bind(&package.name)
        .bind(&package.description)
        .bind(&package.version)
        .bind(package.author_id)
        .bind(package.created_at)
        .bind(package.updated_at)
        .execute(pool)
        .await?;
        Ok(())
    }    pub async fn find_package_by_name(pool: &DatabasePool, name: &str) -> Result<Option<Package>> {
        let row = sqlx::query_as::<_, Package>(
            "SELECT id, name, description, version, author_id, created_at, updated_at
             FROM packages WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }
}

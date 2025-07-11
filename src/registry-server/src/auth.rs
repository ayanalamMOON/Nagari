use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User authentication data
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// JWT token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Token response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Authentication middleware functions
pub mod middleware {
    use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

    /// JWT authentication middleware
    pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
        // Extract the "Authorization" header
        let auth_header = request
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        if let Some(auth_header) = auth_header {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                // TODO: Replace with your JWT secret
                let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

                // Validate the JWT
                match jsonwebtoken::decode::<super::Claims>(
                    token,
                    &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
                    &jsonwebtoken::Validation::default(),
                ) {
                    Ok(_) => {
                        // Token is valid, continue
                    }
                    Err(_) => {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
        Ok(next.run(request).await)
    }
}

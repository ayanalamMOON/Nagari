use axum::{
    extract::{State, Json as ExtractJson},
    response::Json,
    Extension,
};

use crate::{
    AppState,
    models::*,
    error::{AppError, Result},
};

pub async fn register(
    State(state): State<AppState>,
    ExtractJson(request): ExtractJson<CreateUser>,
) -> Result<Json<UserProfile>> {
    // Validate input
    if request.username.len() < 3 {
        return Err(AppError::BadRequest("Username must be at least 3 characters".to_string()));
    }

    if request.email.is_empty() || !request.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }

    if request.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".to_string()));
    }

    // Check if username or email already exists
    if state.user_service.get_user_by_username(&request.username).await?.is_some() {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    if state.user_service.get_user_by_email(&request.email).await?.is_some() {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    let user = state.user_service.create_user(request).await?;
    Ok(Json(user.into()))
}

pub async fn login(
    State(state): State<AppState>,
    ExtractJson(request): ExtractJson<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let user = state.user_service.authenticate_user(&request.username_or_email, &request.password).await?
        .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("Account is disabled".to_string()));
    }

    let token = state.auth_service.generate_token(&user)?;

    Ok(Json(LoginResponse {
        token,
        user: user.into(),
    }))
}

pub async fn get_profile(
    Extension(user): Extension<User>,
) -> Result<Json<UserProfile>> {
    Ok(Json(user.into()))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    ExtractJson(update): ExtractJson<UpdateUserProfile>,
) -> Result<Json<UserProfile>> {
    let updated_user = state.user_service.update_user_profile(user.id, update).await?;
    Ok(Json(updated_user.into()))
}

#[derive(serde::Deserialize)]
pub struct UpdateUserProfile {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
}

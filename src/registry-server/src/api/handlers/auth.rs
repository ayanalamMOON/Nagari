use axum::{
    routing::{get, post},
    Router,
};

/// Authentication routes
pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/me", get(get_current_user))
}

/// User login
async fn login() -> &'static str {
    "Login"
}

/// User registration
async fn register() -> &'static str {
    "Register"
}

/// User logout
async fn logout() -> &'static str {
    "Logout"
}

/// Refresh JWT token
async fn refresh_token() -> &'static str {
    "Refresh token"
}

/// Get current user info
async fn get_current_user() -> &'static str {
    "Current user"
}

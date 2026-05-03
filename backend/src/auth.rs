use std::sync::Arc;

use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::{PasswordHash, SaltString}};
use axum::{Json, extract::{Request, State}, http::{HeaderMap, StatusCode}, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};

use crate::AppState;

// ── Middleware ────────────────────────────────────────────────────────────────

pub async fn middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let ok = state.sessions.is_valid(&token)
        || state.api_key.as_deref() == Some(token.as_str());

    if ok { Ok(next.run(req).await) } else { Err(StatusCode::UNAUTHORIZED) }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginPayload {
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let hash = PasswordHash::new(&state.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(Json(LoginResponse { token: state.sessions.create() }))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> StatusCode {
    if let Some(token) = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        state.sessions.revoke(token);
    }
    StatusCode::NO_CONTENT
}

// ── Password hashing (used by --hash-password CLI flag) ───────────────────────

pub fn hash_password(password: &str) -> String {
    use rand_core::OsRng;
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("hash failed")
        .to_string()
}

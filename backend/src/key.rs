use std::io::Read;
use std::sync::Arc;
use axum::{Json, extract::State};
use serde::Serialize;
use crate::AppState;

#[derive(Serialize)]
pub struct KeysResponse {
    pub api_key: Option<String>,
}

pub async fn get_keys(State(state): State<Arc<AppState>>) -> Json<KeysResponse> {
    Json(KeysResponse {
        api_key: state.api_key.clone(),
    })
}

pub fn random_hex_key() -> String {
    let mut buf = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| { f.read_exact(&mut buf)?; Ok(()) })
        .ok();
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

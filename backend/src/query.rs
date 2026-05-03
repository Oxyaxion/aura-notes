use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::{db::{NoteRow, SearchResult, TagCount}, AppState};

#[derive(Deserialize)]
pub struct QueryParams {
    pub q: Option<String>,
}

pub async fn handle_query(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Vec<NoteRow>>, StatusCode> {
    let q = params.q.unwrap_or_default();
    let db = state.db.clone();
    let results = tokio::task::spawn_blocking(move || db.query_notes(&q))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(results))
}

pub async fn handle_tags(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<TagCount>> {
    let db = state.db.clone();
    let results = tokio::task::spawn_blocking(move || db.list_tags())
        .await
        .unwrap_or_default();
    Json(results)
}

pub async fn handle_aliases(
    State(state): State<Arc<AppState>>,
) -> Json<HashMap<String, String>> {
    let db = state.db.clone();
    let map = tokio::task::spawn_blocking(move || db.get_aliases())
        .await
        .unwrap_or_default();
    Json(map)
}

#[derive(Deserialize)]
pub struct FieldValuesParams {
    pub field: Option<String>,
}

pub async fn handle_field_values(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FieldValuesParams>,
) -> Json<Vec<String>> {
    let field = params.field.unwrap_or_default();
    let db = state.db.clone();
    let values = tokio::task::spawn_blocking(move || db.get_field_values(&field))
        .await
        .unwrap_or_default();
    Json(values)
}

pub async fn handle_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let q = params.q.unwrap_or_default();
    let db = state.db.clone();
    let results = tokio::task::spawn_blocking(move || db.search(&q))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(results))
}

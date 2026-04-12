use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
struct LinkerRow {
    id: String,
    name: String,
    portfolio_id: String,
    portfolio_name: Option<String>,
    cube_id: String,
    cube_name: Option<String>,
    start_date: String,
    fwd_schedule_json: Option<String>,
    fwd_outstanding_json: Option<String>,
    created_at: String,
}

async fn fetch_linker(pool: &sqlx::PgPool, id: &str) -> Result<Value, StatusCode> {
    let row = sqlx::query_as::<_, LinkerRow>(
        r#"SELECT l.id, l.name,
                  l.portfolio_id, p.name AS portfolio_name,
                  l.cube_id,      c.name AS cube_name,
                  l.start_date::TEXT,
                  l.fwd_schedule_json, l.fwd_outstanding_json,
                  l.created_at::TEXT
           FROM linkers l
           LEFT JOIN portfolios_v3 p ON p.id = l.portfolio_id
           LEFT JOIN curve_cubes   c ON c.id = l.cube_id
           WHERE l.id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(linker_to_json(row))
}

fn linker_to_json(r: LinkerRow) -> Value {
    json!({
        "id": r.id,
        "name": r.name,
        "portfolio_id": r.portfolio_id,
        "portfolio_name": r.portfolio_name,
        "cube_id": r.cube_id,
        "cube_name": r.cube_name,
        "start_date": r.start_date,
        "fwd_schedule_json": r.fwd_schedule_json,
        "fwd_outstanding_json": r.fwd_outstanding_json,
        "created_at": r.created_at,
    })
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkerRequest {
    pub name: String,
    pub portfolio_id: String,
    pub cube_id: String,
    pub start_date: String,
    pub fwd_schedule_json: Option<String>,
    pub fwd_outstanding_json: Option<String>,
}

pub async fn list_linkers(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let rows = sqlx::query_as::<_, LinkerRow>(
        r#"SELECT l.id, l.name,
                  l.portfolio_id, p.name AS portfolio_name,
                  l.cube_id,      c.name AS cube_name,
                  l.start_date::TEXT,
                  l.fwd_schedule_json, l.fwd_outstanding_json,
                  l.created_at::TEXT
           FROM linkers l
           LEFT JOIN portfolios_v3 p ON p.id = l.portfolio_id
           LEFT JOIN curve_cubes   c ON c.id = l.cube_id
           ORDER BY l.created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!(rows.into_iter().map(linker_to_json).collect::<Vec<_>>())))
}

pub async fn get_linker(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(fetch_linker(&state.pool, &id).await?))
}

pub async fn create_linker(
    State(state): State<AppState>,
    Json(body): Json<CreateLinkerRequest>,
) -> Result<Json<Value>, StatusCode> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO linkers
           (id, name, portfolio_id, cube_id, start_date, fwd_schedule_json, fwd_outstanding_json)
           VALUES ($1,$2,$3,$4,$5::DATE,$6,$7)"#,
    )
    .bind(&id)
    .bind(&body.name)
    .bind(&body.portfolio_id)
    .bind(&body.cube_id)
    .bind(&body.start_date)
    .bind(&body.fwd_schedule_json)
    .bind(&body.fwd_outstanding_json)
    .execute(&state.pool)
    .await
    .map_err(|e| { tracing::warn!("create_linker: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(fetch_linker(&state.pool, &id).await?))
}

pub async fn delete_linker(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM linkers WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

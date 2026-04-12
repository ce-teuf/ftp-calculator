use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunoffModel {
    pub id: String,
    pub name: String,
    pub product_type: String,
    pub category: Option<String>,
    pub version: i32,
    pub status: String,
    pub method: String,
    pub profile_json: String,
    pub parameters_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunoffModelRequest {
    pub name: String,
    pub product_type: String,
    pub category: Option<String>,
    pub method: String,
    pub profile_json: String,
    pub parameters_json: Option<String>,
}

fn row_to_runoff(r: &sqlx::postgres::PgRow) -> RunoffModel {
    RunoffModel {
        id: r.get("id"),
        name: r.get("name"),
        product_type: r.get("product_type"),
        category: r.get("category"),
        version: r.get("version"),
        status: r.get("status"),
        method: r.get("method"),
        profile_json: r.get("profile_json"),
        parameters_json: r.get("parameters_json"),
        created_at: r.get::<Option<String>, _>("created_at").unwrap_or_default(),
    }
}

pub async fn list_runoff_models(
    State(state): State<AppState>,
) -> Result<Json<Vec<RunoffModel>>, StatusCode> {
    let rows = sqlx::query(
        r#"SELECT id, name, product_type, category, version, status, method,
                  profile_json, parameters_json, created_at::TEXT
           FROM runoff_models ORDER BY created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.iter().map(row_to_runoff).collect()))
}

pub async fn get_runoff_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RunoffModel>, StatusCode> {
    let row = sqlx::query(
        r#"SELECT id, name, product_type, category, version, status, method,
                  profile_json, parameters_json, created_at::TEXT
           FROM runoff_models WHERE id = $1"#,
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(row_to_runoff(&row)))
}

pub async fn create_runoff_model(
    State(state): State<AppState>,
    Json(payload): Json<CreateRunoffModelRequest>,
) -> Result<(StatusCode, Json<RunoffModel>), StatusCode> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO runoff_models
           (id, name, product_type, category, method, profile_json, parameters_json)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.product_type)
    .bind(&payload.category)
    .bind(&payload.method)
    .bind(&payload.profile_json)
    .bind(&payload.parameters_json)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let model = get_runoff_model(State(state), Path(id)).await?;
    Ok((StatusCode::CREATED, model))
}

pub async fn delete_runoff_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    match sqlx::query("DELETE FROM runoff_models WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
    {
        Ok(r) if r.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

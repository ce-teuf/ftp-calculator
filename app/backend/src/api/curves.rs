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
pub struct RateCurve {
    pub id: String,
    pub name: String,
    pub component: String,
    pub currency: String,
    pub version: i32,
    pub status: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub tenors_json: String,
    pub values_json: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub series_name: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCurveRequest {
    pub name: String,
    pub component: String,
    pub currency: Option<String>,
    pub valid_from: Option<String>,
    pub tenors_json: String,
    pub values_json: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    /// Name of the underlying historical rate series (e.g. "SOFR", "ESTR").
    /// Links this curve to rate_series_data for projection purposes.
    pub series_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCurveRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub tenors_json: Option<String>,
    pub values_json: Option<String>,
    pub notes: Option<String>,
}

fn row_to_curve(r: &sqlx::postgres::PgRow) -> RateCurve {
    RateCurve {
        id:          r.get("id"),
        name:        r.get("name"),
        component:   r.get("component"),
        currency:    r.get("currency"),
        version:     r.get("version"),
        status:      r.get("status"),
        valid_from:  r.get("valid_from"),
        valid_to:    r.get("valid_to"),
        tenors_json: r.get("tenors_json"),
        values_json: r.get("values_json"),
        source:      r.get("source"),
        notes:       r.get("notes"),
        series_name: r.get("series_name"),
        created_at:  r.get::<String, _>("created_at"),
        created_by:  r.get("created_by"),
    }
}

const SELECT_CURVE: &str = r#"
    SELECT id, name, component, currency, version, status,
           valid_from::TEXT, valid_to::TEXT,
           tenors_json, values_json, source, notes, series_name,
           created_at::TEXT AS created_at, created_by
    FROM rate_curves
"#;

pub async fn list_curves(
    State(state): State<AppState>,
) -> Result<Json<Vec<RateCurve>>, StatusCode> {
    let rows = sqlx::query(&format!("{} ORDER BY created_at DESC", SELECT_CURVE))
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.iter().map(row_to_curve).collect()))
}

pub async fn get_curve(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RateCurve>, StatusCode> {
    let row = sqlx::query(&format!("{} WHERE id = $1", SELECT_CURVE))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(row_to_curve(&row)))
}

pub async fn create_curve(
    State(state): State<AppState>,
    Json(payload): Json<CreateCurveRequest>,
) -> Result<(StatusCode, Json<RateCurve>), StatusCode> {
    let id = Uuid::new_v4().to_string();
    let currency = payload.currency.unwrap_or_else(|| "EUR".to_string());

    sqlx::query(
        r#"INSERT INTO rate_curves
           (id, name, component, currency, tenors_json, values_json, source, notes, valid_from, series_name)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::DATE, $10)"#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.component)
    .bind(&currency)
    .bind(&payload.tenors_json)
    .bind(&payload.values_json)
    .bind(&payload.source)
    .bind(&payload.notes)
    .bind(&payload.valid_from)
    .bind(&payload.series_name)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let curve = get_curve(State(state), Path(id)).await?;
    Ok((StatusCode::CREATED, curve))
}

pub async fn update_curve(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCurveRequest>,
) -> Result<Json<RateCurve>, StatusCode> {
    sqlx::query(
        r#"UPDATE rate_curves SET
           name        = COALESCE($2, name),
           status      = COALESCE($3, status),
           tenors_json = COALESCE($4, tenors_json),
           values_json = COALESCE($5, values_json),
           notes       = COALESCE($6, notes),
           version     = version + 1
           WHERE id = $1"#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.status)
    .bind(&payload.tenors_json)
    .bind(&payload.values_json)
    .bind(&payload.notes)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_curve(State(state), Path(id)).await
}

pub async fn delete_curve(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    match sqlx::query("DELETE FROM rate_curves WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
    {
        Ok(r) if r.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

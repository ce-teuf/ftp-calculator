use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::AppState;

// ── DB row ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
struct CubRow {
    id: String,
    name: String,
    description: Option<String>,
    stack_id: String,
    stack_name: Option<String>,
    analysis_start: String,
    analysis_end: String,
    step_months: i32,
    include_proj: bool,
    proj_script: Option<String>,
    mc_scenarios: i32,
    proj_config_json: Option<String>,
    status: String,
    created_at: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Count how many analysis time points a cube has (inclusive, step_months apart).
fn count_analysis_times(start: &str, end: &str, step: i32) -> i32 {
    // Parse YYYY-MM-DD
    fn ym(s: &str) -> Option<(i32, i32)> {
        let parts: Vec<&str> = s.splitn(3, '-').collect();
        if parts.len() < 2 { return None; }
        Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
    }
    match (ym(start), ym(end)) {
        (Some((sy, sm)), Some((ey, em))) => {
            let total_months = (ey - sy) * 12 + (em - sm);
            if total_months < 0 || step <= 0 { return 1; }
            total_months / step + 1
        }
        _ => 1,
    }
}

async fn fetch_cube(pool: &sqlx::PgPool, id: &str) -> Result<Value, StatusCode> {
    let row = sqlx::query_as::<_, CubRow>(
        r#"SELECT c.id, c.name, c.description,
                  c.stack_id, s.name AS stack_name,
                  c.analysis_start::TEXT, c.analysis_end::TEXT,
                  c.step_months, c.include_proj, c.proj_script,
                  c.mc_scenarios, c.proj_config_json, c.status, c.created_at::TEXT
           FROM curve_cubes c
           LEFT JOIN curve_stacks s ON s.id = c.stack_id
           WHERE c.id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let n_times = count_analysis_times(&row.analysis_start, &row.analysis_end, row.step_months);

    Ok(json!({
        "id": row.id,
        "name": row.name,
        "description": row.description,
        "stack_id": row.stack_id,
        "stack_name": row.stack_name,
        "analysis_start": row.analysis_start,
        "analysis_end": row.analysis_end,
        "step_months": row.step_months,
        "include_proj": row.include_proj,
        "proj_script": row.proj_script,
        "mc_scenarios": row.mc_scenarios,
        "proj_config_json": row.proj_config_json,
        "status": row.status,
        "created_at": row.created_at,
        "n_analysis_times": n_times,
    }))
}

// ── Request body ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateCubeRequest {
    pub name: String,
    pub description: Option<String>,
    pub stack_id: String,
    pub analysis_start: String,
    pub analysis_end: String,
    pub step_months: Option<i32>,
    pub include_proj: Option<bool>,
    pub proj_script: Option<String>,
    pub mc_scenarios: Option<i32>,
    /// JSON object keyed by series_name → { method, n_scenarios, seed, params? }
    pub proj_config_json: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_cubes(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let rows = sqlx::query_as::<_, CubRow>(
        r#"SELECT c.id, c.name, c.description,
                  c.stack_id, s.name AS stack_name,
                  c.analysis_start::TEXT, c.analysis_end::TEXT,
                  c.step_months, c.include_proj, c.proj_script,
                  c.mc_scenarios, c.proj_config_json, c.status, c.created_at::TEXT
           FROM curve_cubes c
           LEFT JOIN curve_stacks s ON s.id = c.stack_id
           ORDER BY c.created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            let n = count_analysis_times(&r.analysis_start, &r.analysis_end, r.step_months);
            json!({
                "id": r.id,
                "name": r.name,
                "description": r.description,
                "stack_id": r.stack_id,
                "stack_name": r.stack_name,
                "analysis_start": r.analysis_start,
                "analysis_end": r.analysis_end,
                "step_months": r.step_months,
                "include_proj": r.include_proj,
                "mc_scenarios": r.mc_scenarios,
                "status": r.status,
                "created_at": r.created_at,
                "n_analysis_times": n,
            })
        })
        .collect();

    Ok(Json(json!(result)))
}

pub async fn get_cube(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(fetch_cube(&state.pool, &id).await?))
}

pub async fn create_cube(
    State(state): State<AppState>,
    Json(body): Json<CreateCubeRequest>,
) -> Result<Json<Value>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let step = body.step_months.unwrap_or(1).max(1);
    let mc = body.mc_scenarios.unwrap_or(0).max(0);
    let proj = body.include_proj.unwrap_or(false);

    sqlx::query(
        r#"INSERT INTO curve_cubes
           (id, name, description, stack_id, analysis_start, analysis_end,
            step_months, include_proj, proj_script, mc_scenarios, proj_config_json)
           VALUES ($1,$2,$3,$4,$5::DATE,$6::DATE,$7,$8,$9,$10,$11)"#,
    )
    .bind(&id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.stack_id)
    .bind(&body.analysis_start)
    .bind(&body.analysis_end)
    .bind(step)
    .bind(proj)
    .bind(&body.proj_script)
    .bind(mc)
    .bind(&body.proj_config_json)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::warn!("create_cube error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(fetch_cube(&state.pool, &id).await?))
}

pub async fn update_cube(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CreateCubeRequest>,
) -> Result<Json<Value>, StatusCode> {
    let step = body.step_months.unwrap_or(1).max(1);
    let mc = body.mc_scenarios.unwrap_or(0).max(0);
    let proj = body.include_proj.unwrap_or(false);

    sqlx::query(
        r#"UPDATE curve_cubes SET
           name=$1, description=$2, stack_id=$3,
           analysis_start=$4::DATE, analysis_end=$5::DATE,
           step_months=$6, include_proj=$7, proj_script=$8, mc_scenarios=$9,
           proj_config_json=$10
           WHERE id=$11"#,
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.stack_id)
    .bind(&body.analysis_start)
    .bind(&body.analysis_end)
    .bind(step)
    .bind(proj)
    .bind(&body.proj_script)
    .bind(mc)
    .bind(&body.proj_config_json)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(fetch_cube(&state.pool, &id).await?))
}

pub async fn delete_cube(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM curve_cubes WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

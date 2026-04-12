use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::Row;

use crate::db::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: String,
    pub label: Option<String>,
    pub method: String,
    pub portfolio_id: String,
    pub curve_ids_json: String,
    pub parameters_json: String,
    pub seeds_json: Option<String>,
    pub result_json: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub notes: Option<String>,
    /// Input matrices stored for replay (added by migration 002)
    pub outstanding_json: Option<String>,
    pub profiles_json: Option<String>,
    pub rates_json: Option<String>,
}

fn row_to_execution(r: &sqlx::postgres::PgRow) -> Execution {
    Execution {
        id: r.get("id"),
        label: r.get("label"),
        method: r.get("method"),
        portfolio_id: r.get("portfolio_id"),
        curve_ids_json: r.get("curve_ids_json"),
        parameters_json: r.get("parameters_json"),
        seeds_json: r.get("seeds_json"),
        result_json: r.get("result_json"),
        status: r.get("status"),
        error_message: r.get("error_message"),
        duration_ms: r.get("duration_ms"),
        created_at: r.get::<Option<String>, _>("created_at").unwrap_or_default(),
        created_by: r.get("created_by"),
        notes: r.get("notes"),
        outstanding_json: r.get("outstanding_json"),
        profiles_json: r.get("profiles_json"),
        rates_json: r.get("rates_json"),
    }
}

pub async fn list_executions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Execution>>, StatusCode> {
    let rows = sqlx::query(
        r#"SELECT id, label, method, portfolio_id, curve_ids_json,
                  parameters_json, seeds_json, result_json, status,
                  error_message, duration_ms, created_at::TEXT, created_by, notes,
                  outstanding_json, profiles_json, rates_json
           FROM executions ORDER BY created_at DESC LIMIT 200"#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.iter().map(row_to_execution).collect()))
}

pub async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Execution>, StatusCode> {
    let row = sqlx::query(
        r#"SELECT id, label, method, portfolio_id, curve_ids_json,
                  parameters_json, seeds_json, result_json, status,
                  error_message, duration_ms, created_at::TEXT, created_by, notes,
                  outstanding_json, profiles_json, rates_json
           FROM executions WHERE id = $1"#,
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(row_to_execution(&row)))
}

/// GET /api/executions/:id/inputs
/// Returns the raw input matrices stored for an execution so the UI can replay it.
#[derive(Serialize)]
pub struct ExecutionInputs {
    pub execution_id: String,
    pub method: String,
    pub portfolio_id: String,
    pub label: Option<String>,
    pub outstanding_json: Option<String>,
    pub profiles_json: Option<String>,
    pub rates_json: Option<String>,
}

/// GET /api/executions/diff?a=<id>&b=<id>
/// Compare KPIs and weighted FTP rates of two executions.
#[derive(Deserialize)]
pub struct DiffQuery {
    pub a: String,
    pub b: String,
}

#[derive(Serialize)]
pub struct ExecSummary {
    pub id: String,
    pub label: Option<String>,
    pub method: String,
    pub created_at: String,
    pub kpis: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct DiffResponse {
    pub a: ExecSummary,
    pub b: ExecSummary,
    /// delta_weighted_ftp_rate = b.weighted_ftp_rate - a.weighted_ftp_rate (if both have kpis)
    pub delta_weighted_ftp_rate: Option<f64>,
    pub delta_total_outstanding: Option<f64>,
    pub delta_ftp_int_monthly: Option<f64>,
}

fn extract_kpis(result_json: Option<&str>) -> Option<serde_json::Value> {
    let json = result_json?;
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v.get("kpis").cloned()
}

fn kpi_f64(kpis: &serde_json::Value, key: &str) -> Option<f64> {
    kpis.get(key)?.as_f64()
}

pub async fn diff_executions(
    State(state): State<AppState>,
    Query(q): Query<DiffQuery>,
) -> Result<Json<DiffResponse>, StatusCode> {
    let fetch = |id: String| {
        let pool = state.pool.clone();
        async move {
            sqlx::query(
                r#"SELECT id, label, method, result_json, created_at::TEXT
                   FROM executions WHERE id = $1"#,
            )
            .bind(&id)
            .fetch_optional(&pool)
            .await
        }
    };

    let (ra, rb) = tokio::try_join!(fetch(q.a), fetch(q.b))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ra = ra.ok_or(StatusCode::NOT_FOUND)?;
    let rb = rb.ok_or(StatusCode::NOT_FOUND)?;

    let result_a: Option<String> = ra.get("result_json");
    let result_b: Option<String> = rb.get("result_json");

    let kpis_a = extract_kpis(result_a.as_deref());
    let kpis_b = extract_kpis(result_b.as_deref());

    let delta_weighted_ftp_rate = kpis_a.as_ref().zip(kpis_b.as_ref()).and_then(|(a, b)| {
        Some(kpi_f64(b, "weighted_ftp_rate")? - kpi_f64(a, "weighted_ftp_rate")?)
    });
    let delta_total_outstanding = kpis_a.as_ref().zip(kpis_b.as_ref()).and_then(|(a, b)| {
        Some(kpi_f64(b, "total_outstanding")? - kpi_f64(a, "total_outstanding")?)
    });
    let delta_ftp_int_monthly = kpis_a.as_ref().zip(kpis_b.as_ref()).and_then(|(a, b)| {
        Some(kpi_f64(b, "total_ftp_int_monthly")? - kpi_f64(a, "total_ftp_int_monthly")?)
    });

    Ok(Json(DiffResponse {
        a: ExecSummary {
            id:         ra.get("id"),
            label:      ra.get("label"),
            method:     ra.get("method"),
            created_at: ra.get::<Option<String>, _>("created_at").unwrap_or_default(),
            kpis: kpis_a,
        },
        b: ExecSummary {
            id:         rb.get("id"),
            label:      rb.get("label"),
            method:     rb.get("method"),
            created_at: rb.get::<Option<String>, _>("created_at").unwrap_or_default(),
            kpis: kpis_b,
        },
        delta_weighted_ftp_rate,
        delta_total_outstanding,
        delta_ftp_int_monthly,
    }))
}

pub async fn get_execution_inputs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ExecutionInputs>, StatusCode> {
    let row = sqlx::query(
        r#"SELECT id, method, portfolio_id, label,
                  outstanding_json, profiles_json, rates_json
           FROM executions WHERE id = $1"#,
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ExecutionInputs {
        execution_id: row.get("id"),
        method:       row.get("method"),
        portfolio_id: row.get("portfolio_id"),
        label:        row.get("label"),
        outstanding_json: row.get("outstanding_json"),
        profiles_json:    row.get("profiles_json"),
        rates_json:       row.get("rates_json"),
    }))
}

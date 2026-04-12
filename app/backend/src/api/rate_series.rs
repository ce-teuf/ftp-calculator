use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashMap;

use crate::db::AppState;

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RateSeriesQuery {
    /// Comma-separated series names, e.g. "ESTR,EURIBOR,SOFR"
    pub series: Option<String>,
    /// ISO date lower bound (inclusive)
    pub from: Option<String>,
    /// ISO date upper bound (inclusive)
    pub to: Option<String>,
    /// Single tenor filter, e.g. "3M"
    pub tenor: Option<String>,
    /// Max rows returned (default 20 000)
    pub limit: Option<i64>,
}

// ── List rate series names ─────────────────────────────────────────────────────

pub async fn list_series_names(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let rows = sqlx::query(
        "SELECT DISTINCT series_name, component, currency FROM rate_series_data ORDER BY series_name"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let names: Vec<Value> = rows.iter().map(|r| json!({
        "name":      r.get::<String, _>("series_name"),
        "component": r.get::<String, _>("component"),
        "currency":  r.get::<String, _>("currency"),
    })).collect();

    Ok(Json(json!({ "series": names })))
}

// ── Query rate series data ────────────────────────────────────────────────────

pub async fn query_rate_series(
    State(state): State<AppState>,
    Query(q): Query<RateSeriesQuery>,
) -> Result<Json<Value>, StatusCode> {
    let limit = q.limit.unwrap_or(20_000).min(100_000);

    // Build WHERE clauses dynamically
    let mut conditions: Vec<String> = Vec::new();
    let mut bind_idx = 1i32;

    // Parse comma-separated series names
    let series_names: Vec<String> = q.series
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if !series_names.is_empty() {
        conditions.push(format!("series_name = ANY(${bind_idx})"));
        bind_idx += 1;
    }

    let from = q.from.as_deref().filter(|s| !s.is_empty());
    if from.is_some() {
        conditions.push(format!("obs_date >= ${}::DATE", bind_idx));
        bind_idx += 1;
    }

    let to = q.to.as_deref().filter(|s| !s.is_empty());
    if to.is_some() {
        conditions.push(format!("obs_date <= ${}::DATE", bind_idx));
        bind_idx += 1;
    }

    let tenor_filter = q.tenor.as_deref().filter(|s| !s.is_empty());
    if tenor_filter.is_some() {
        conditions.push(format!("tenor = ${}", bind_idx));
        // bind_idx += 1;  // last param, no need to increment
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT series_name, obs_date::TEXT AS date, tenor, rate \
         FROM rate_series_data {where_clause} \
         ORDER BY series_name, obs_date, tenor \
         LIMIT {limit}"
    );

    // Build query with dynamic bindings
    let mut query = sqlx::query(&sql);
    if !series_names.is_empty() {
        query = query.bind(series_names.clone());
    }
    if let Some(f) = from {
        query = query.bind(f.to_string());
    }
    if let Some(t) = to {
        query = query.bind(t.to_string());
    }
    if let Some(tn) = tenor_filter {
        query = query.bind(tn.to_string());
    }

    let rows = query
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("rate_series query failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Group by series_name → list of {date, tenor, rate}
    let mut grouped: HashMap<String, Vec<Value>> = HashMap::new();
    for r in &rows {
        let sn: String = r.get("series_name");
        let obs: String = r.get("date");
        let tn: Option<String> = r.get("tenor");
        let rate: f64 = r.get("rate");
        grouped.entry(sn).or_default().push(json!({
            "date":  obs,
            "tenor": tn,
            "rate":  rate,
        }));
    }

    Ok(Json(json!({
        "total_rows": rows.len(),
        "data": grouped,
    })))
}

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::Response,
    body::Body,
};
use serde_json::{json, Value};
use sqlx::Row;

use crate::db::AppState;

/// GET /api/export
/// Returns a complete JSON backup of all tables: rate_curves, portfolios,
/// portfolio_positions, runoff_models, executions.
/// The response is served as an attachment so browsers download it directly.
pub async fn export_backup(
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let pool = &state.pool;

    // Rate curves
    let curves: Vec<Value> = sqlx::query(
        "SELECT id, name, component, currency, version, status, valid_from::TEXT, valid_to::TEXT,
                tenors_json, values_json, source, notes, created_at::TEXT, created_by
         FROM rate_curves ORDER BY created_at"
    )
    .fetch_all(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .iter().map(|r| json!({
        "id": r.get::<String, _>("id"),
        "name": r.get::<String, _>("name"),
        "component": r.get::<String, _>("component"),
        "currency": r.get::<String, _>("currency"),
        "version": r.get::<i32, _>("version"),
        "status": r.get::<String, _>("status"),
        "valid_from": r.get::<Option<String>, _>("valid_from"),
        "valid_to": r.get::<Option<String>, _>("valid_to"),
        "tenors_json": r.get::<String, _>("tenors_json"),
        "values_json": r.get::<String, _>("values_json"),
        "source": r.get::<Option<String>, _>("source"),
        "notes": r.get::<Option<String>, _>("notes"),
        "created_at": r.get::<Option<String>, _>("created_at"),
        "created_by": r.get::<Option<String>, _>("created_by"),
    })).collect();

    // Portfolios
    let portfolios: Vec<Value> = sqlx::query(
        "SELECT id, name, description, version, status, as_of_date::TEXT, created_at::TEXT
         FROM portfolios ORDER BY created_at"
    )
    .fetch_all(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .iter().map(|r| json!({
        "id": r.get::<String, _>("id"),
        "name": r.get::<String, _>("name"),
        "description": r.get::<Option<String>, _>("description"),
        "version": r.get::<i32, _>("version"),
        "status": r.get::<String, _>("status"),
        "as_of_date": r.get::<Option<String>, _>("as_of_date"),
        "created_at": r.get::<Option<String>, _>("created_at"),
    })).collect();

    // Portfolio positions
    let positions: Vec<Value> = sqlx::query(
        "SELECT id, portfolio_id, position_ref, product_type, branch, seller, currency,
                outstanding, origination_date::TEXT, maturity_date::TEXT, client_rate,
                runoff_model_id, risk_weight, profiles_json, rates_json
         FROM portfolio_positions ORDER BY portfolio_id"
    )
    .fetch_all(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .iter().map(|r| json!({
        "id": r.get::<String, _>("id"),
        "portfolio_id": r.get::<String, _>("portfolio_id"),
        "position_ref": r.get::<Option<String>, _>("position_ref"),
        "product_type": r.get::<String, _>("product_type"),
        "branch": r.get::<Option<String>, _>("branch"),
        "seller": r.get::<Option<String>, _>("seller"),
        "currency": r.get::<String, _>("currency"),
        "outstanding": r.get::<f64, _>("outstanding"),
        "origination_date": r.get::<Option<String>, _>("origination_date"),
        "maturity_date": r.get::<Option<String>, _>("maturity_date"),
        "client_rate": r.get::<Option<f64>, _>("client_rate"),
        "runoff_model_id": r.get::<Option<String>, _>("runoff_model_id"),
        "risk_weight": r.get::<Option<f64>, _>("risk_weight"),
        "profiles_json": r.get::<Option<String>, _>("profiles_json"),
        "rates_json": r.get::<Option<String>, _>("rates_json"),
    })).collect();

    // Runoff models
    let runoff_models: Vec<Value> = sqlx::query(
        "SELECT id, name, product_type, category, version, status, method,
                profile_json, parameters_json, created_at::TEXT
         FROM runoff_models ORDER BY created_at"
    )
    .fetch_all(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .iter().map(|r| json!({
        "id": r.get::<String, _>("id"),
        "name": r.get::<String, _>("name"),
        "product_type": r.get::<String, _>("product_type"),
        "category": r.get::<Option<String>, _>("category"),
        "version": r.get::<i32, _>("version"),
        "status": r.get::<String, _>("status"),
        "method": r.get::<String, _>("method"),
        "profile_json": r.get::<String, _>("profile_json"),
        "parameters_json": r.get::<Option<String>, _>("parameters_json"),
        "created_at": r.get::<Option<String>, _>("created_at"),
    })).collect();

    // Executions (without result matrices — too large; store separately)
    let executions: Vec<Value> = sqlx::query(
        "SELECT id, label, method, portfolio_id, curve_ids_json, parameters_json,
                status, error_message, duration_ms, created_at::TEXT, created_by, notes,
                outstanding_json, profiles_json, rates_json
         FROM executions ORDER BY created_at"
    )
    .fetch_all(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .iter().map(|r| json!({
        "id": r.get::<String, _>("id"),
        "label": r.get::<Option<String>, _>("label"),
        "method": r.get::<String, _>("method"),
        "portfolio_id": r.get::<String, _>("portfolio_id"),
        "curve_ids_json": r.get::<String, _>("curve_ids_json"),
        "parameters_json": r.get::<String, _>("parameters_json"),
        "status": r.get::<String, _>("status"),
        "error_message": r.get::<Option<String>, _>("error_message"),
        "duration_ms": r.get::<Option<i64>, _>("duration_ms"),
        "created_at": r.get::<Option<String>, _>("created_at"),
        "created_by": r.get::<Option<String>, _>("created_by"),
        "notes": r.get::<Option<String>, _>("notes"),
        "outstanding_json": r.get::<Option<String>, _>("outstanding_json"),
        "profiles_json": r.get::<Option<String>, _>("profiles_json"),
        "rates_json": r.get::<Option<String>, _>("rates_json"),
    })).collect();

    let backup = json!({
        "export_version": 1,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "rate_curves": curves,
        "portfolios": portfolios,
        "portfolio_positions": positions,
        "runoff_models": runoff_models,
        "executions": executions,
    });

    let body = serde_json::to_vec_pretty(&backup)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let filename = format!(
        "ftp-simulator-backup-{}.json",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(body))
        .unwrap())
}

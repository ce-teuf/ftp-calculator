use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub status: String,
    pub as_of_date: String,
    pub position_count: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub id: String,
    pub portfolio_id: String,
    pub position_ref: Option<String>,
    pub product_type: String,
    pub branch: Option<String>,
    pub seller: Option<String>,
    pub currency: String,
    pub outstanding: f64,
    pub origination_date: Option<String>,
    pub maturity_date: Option<String>,
    pub client_rate: Option<f64>,
    pub runoff_model_id: Option<String>,
    pub risk_weight: f64,
    pub profiles_json: Option<String>,
    pub rates_json: Option<String>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePortfolioRequest {
    pub name: String,
    pub description: Option<String>,
    pub as_of_date: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePositionRequest {
    pub position_ref: Option<String>,
    pub product_type: String,
    pub branch: Option<String>,
    pub seller: Option<String>,
    pub currency: Option<String>,
    pub outstanding: f64,
    pub origination_date: Option<String>,
    pub maturity_date: Option<String>,
    pub client_rate: Option<f64>,
    pub runoff_model_id: Option<String>,
    pub risk_weight: Option<f64>,
    pub profiles_json: Option<String>,
    pub rates_json: Option<String>,
    pub metadata_json: Option<String>,
}

fn row_to_portfolio(r: &sqlx::postgres::PgRow) -> Portfolio {
    Portfolio {
        id: r.get("id"),
        name: r.get("name"),
        description: r.get("description"),
        version: r.get("version"),
        status: r.get("status"),
        as_of_date: r.get::<Option<String>, _>("as_of_date").unwrap_or_default(),
        position_count: r.try_get("position_count").ok(),
        created_at: r.get::<Option<String>, _>("created_at").unwrap_or_default(),
    }
}

fn row_to_position(r: &sqlx::postgres::PgRow) -> PortfolioPosition {
    PortfolioPosition {
        id: r.get("id"),
        portfolio_id: r.get("portfolio_id"),
        position_ref: r.get("position_ref"),
        product_type: r.get("product_type"),
        branch: r.get("branch"),
        seller: r.get("seller"),
        currency: r.get("currency"),
        outstanding: r.get("outstanding"),
        origination_date: r.get("origination_date"),
        maturity_date: r.get("maturity_date"),
        client_rate: r.get("client_rate"),
        runoff_model_id: r.get("runoff_model_id"),
        risk_weight: r.get::<Option<f64>, _>("risk_weight").unwrap_or(1.0),
        profiles_json: r.get("profiles_json"),
        rates_json: r.get("rates_json"),
        metadata_json: r.get("metadata_json"),
    }
}

// ── Portfolios ────────────────────────────────────────────────────────────────

pub async fn list_portfolios(
    State(state): State<AppState>,
) -> Result<Json<Vec<Portfolio>>, StatusCode> {
    let rows = sqlx::query(
        r#"SELECT p.id, p.name, p.description, p.version, p.status,
                  p.as_of_date::TEXT, p.created_at::TEXT,
                  COUNT(pp.id) AS position_count
           FROM portfolios p
           LEFT JOIN portfolio_positions pp ON pp.portfolio_id = p.id
           GROUP BY p.id ORDER BY p.created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.iter().map(row_to_portfolio).collect()))
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Portfolio>, StatusCode> {
    let row = sqlx::query(
        r#"SELECT p.id, p.name, p.description, p.version, p.status,
                  p.as_of_date::TEXT, p.created_at::TEXT,
                  COUNT(pp.id) AS position_count
           FROM portfolios p
           LEFT JOIN portfolio_positions pp ON pp.portfolio_id = p.id
           WHERE p.id = $1 GROUP BY p.id"#,
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(row_to_portfolio(&row)))
}

pub async fn create_portfolio(
    State(state): State<AppState>,
    Json(payload): Json<CreatePortfolioRequest>,
) -> Result<(StatusCode, Json<Portfolio>), StatusCode> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO portfolios (id, name, description, as_of_date) VALUES ($1, $2, $3, $4::DATE)",
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.as_of_date)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let p = get_portfolio(State(state), Path(id)).await?;
    Ok((StatusCode::CREATED, p))
}

pub async fn delete_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    match sqlx::query("DELETE FROM portfolios WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
    {
        Ok(r) if r.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ── Positions ─────────────────────────────────────────────────────────────────

pub async fn list_positions(
    State(state): State<AppState>,
    Path(portfolio_id): Path<String>,
) -> Result<Json<Vec<PortfolioPosition>>, StatusCode> {
    let rows = sqlx::query(
        r#"SELECT id, portfolio_id, position_ref, product_type,
                  branch, seller, currency, outstanding,
                  origination_date::TEXT, maturity_date::TEXT,
                  client_rate, runoff_model_id, risk_weight,
                  profiles_json, rates_json, metadata_json
           FROM portfolio_positions WHERE portfolio_id = $1
           ORDER BY position_ref NULLS LAST"#,
    )
    .bind(&portfolio_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.iter().map(row_to_position).collect()))
}

async fn insert_position(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    portfolio_id: &str,
    p: &CreatePositionRequest,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let currency = p.currency.clone().unwrap_or_else(|| "EUR".to_string());
    let risk_weight = p.risk_weight.unwrap_or(1.0);

    sqlx::query(
        r#"INSERT INTO portfolio_positions
           (id, portfolio_id, position_ref, product_type, branch, seller,
            currency, outstanding, origination_date, maturity_date,
            client_rate, runoff_model_id, risk_weight,
            profiles_json, rates_json, metadata_json)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::DATE, $10::DATE,
                   $11, $12, $13, $14, $15, $16)"#,
    )
    .bind(&id)
    .bind(portfolio_id)
    .bind(&p.position_ref)
    .bind(&p.product_type)
    .bind(&p.branch)
    .bind(&p.seller)
    .bind(&currency)
    .bind(p.outstanding)
    .bind(&p.origination_date)
    .bind(&p.maturity_date)
    .bind(p.client_rate)
    .bind(&p.runoff_model_id)
    .bind(risk_weight)
    .bind(&p.profiles_json)
    .bind(&p.rates_json)
    .bind(&p.metadata_json)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn add_position(
    State(state): State<AppState>,
    Path(portfolio_id): Path<String>,
    Json(payload): Json<CreatePositionRequest>,
) -> Result<(StatusCode, Json<PortfolioPosition>), StatusCode> {
    let id = Uuid::new_v4().to_string();
    let currency = payload.currency.clone().unwrap_or_else(|| "EUR".to_string());
    let risk_weight = payload.risk_weight.unwrap_or(1.0);

    sqlx::query(
        r#"INSERT INTO portfolio_positions
           (id, portfolio_id, position_ref, product_type, branch, seller,
            currency, outstanding, origination_date, maturity_date,
            client_rate, runoff_model_id, risk_weight,
            profiles_json, rates_json, metadata_json)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::DATE, $10::DATE,
                   $11, $12, $13, $14, $15, $16)"#,
    )
    .bind(&id)
    .bind(&portfolio_id)
    .bind(&payload.position_ref)
    .bind(&payload.product_type)
    .bind(&payload.branch)
    .bind(&payload.seller)
    .bind(&currency)
    .bind(payload.outstanding)
    .bind(&payload.origination_date)
    .bind(&payload.maturity_date)
    .bind(payload.client_rate)
    .bind(&payload.runoff_model_id)
    .bind(risk_weight)
    .bind(&payload.profiles_json)
    .bind(&payload.rates_json)
    .bind(&payload.metadata_json)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        r#"SELECT id, portfolio_id, position_ref, product_type,
                  branch, seller, currency, outstanding,
                  origination_date::TEXT, maturity_date::TEXT,
                  client_rate, runoff_model_id, risk_weight,
                  profiles_json, rates_json, metadata_json
           FROM portfolio_positions WHERE id = $1"#,
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(row_to_position(&row))))
}

/// Replace all positions of a portfolio atomically.
pub async fn bulk_import_positions(
    State(state): State<AppState>,
    Path(portfolio_id): Path<String>,
    Json(positions): Json<Vec<CreatePositionRequest>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let n = positions.len();
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("DELETE FROM portfolio_positions WHERE portfolio_id = $1")
        .bind(&portfolio_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for p in &positions {
        insert_position(&mut tx, &portfolio_id, p)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "imported": n })))
}

pub async fn delete_position(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    match sqlx::query("DELETE FROM portfolio_positions WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
    {
        Ok(r) if r.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

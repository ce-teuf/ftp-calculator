use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::AppState;

// ── FTP standard tenors (months) ──────────────────────────────────────────────
pub const FTP_TENORS: &[&str] = &[
    "1M", "3M", "6M", "12M", "24M", "36M", "60M", "84M", "120M", "180M", "240M", "360M",
];
pub const FTP_TENOR_COLS: &[&str] = &[
    "m1", "m3", "m6", "m12", "m24", "m36", "m60", "m84", "m120", "m180", "m240", "m360",
];

// ── DB rows ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
struct PortfolioRow {
    id: String,
    name: String,
    description: Option<String>,
    schedule_type: String,
    created_at: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a schedule CSV into a JSON array.
/// Expected columns: date, m1, m3, m6, m12, m24, m36, m60, m84, m120, m180, m240, m360
/// Returns (rows_json, row_count, error_option)
fn parse_schedule_csv(content: &str) -> Result<(Value, usize), String> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| format!("CSV header error: {e}"))?
        .iter()
        .map(|h| h.to_lowercase())
        .collect();

    let date_col = headers.iter().position(|h| h == "date")
        .ok_or("Colonne 'date' manquante")?;

    let tenor_positions: Vec<Option<usize>> = FTP_TENOR_COLS
        .iter()
        .map(|t| headers.iter().position(|h| h == *t))
        .collect();

    let mut rows = vec![];
    for result in rdr.records() {
        let rec = result.map_err(|e| format!("CSV parse error: {e}"))?;
        let date = rec.get(date_col).unwrap_or("").trim().to_string();
        if date.is_empty() { continue; }

        let buckets: Vec<f64> = tenor_positions
            .iter()
            .map(|pos| {
                pos.and_then(|p| rec.get(p))
                    .and_then(|v| v.trim().parse::<f64>().ok())
                    .unwrap_or(0.0)
            })
            .collect();

        rows.push(json!({ "date": date, "buckets": buckets }));
    }

    let n = rows.len();
    Ok((json!(rows), n))
}

/// Parse an outstanding CSV into a JSON array.
/// Expected columns: date, outstanding
fn parse_outstanding_csv(content: &str) -> Result<(Value, usize), String> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| format!("CSV header error: {e}"))?
        .iter()
        .map(|h| h.to_lowercase())
        .collect();

    let date_col = headers.iter().position(|h| h == "date")
        .ok_or("Colonne 'date' manquante")?;
    let val_col = headers
        .iter()
        .position(|h| h == "outstanding" || h == "amount" || h == "encours")
        .ok_or("Colonne 'outstanding' manquante")?;

    let mut rows = vec![];
    for result in rdr.records() {
        let rec = result.map_err(|e| format!("CSV parse error: {e}"))?;
        let date = rec.get(date_col).unwrap_or("").trim().to_string();
        if date.is_empty() { continue; }
        let val: f64 = rec.get(val_col)
            .and_then(|v| v.trim().replace(',', ".").parse().ok())
            .unwrap_or(0.0);
        rows.push(json!({ "date": date, "outstanding": val }));
    }

    let n = rows.len();
    Ok((json!(rows), n))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_portfolios(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let rows = sqlx::query_as::<_, PortfolioRow>(
        "SELECT id, name, description, schedule_type, created_at::TEXT
         FROM portfolios_v3 ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Attach row count per portfolio
    let mut result = vec![];
    for p in rows {
        let row_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM portfolio_rows WHERE portfolio_id = $1",
        )
        .bind(&p.id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

        result.push(json!({
            "id": p.id,
            "name": p.name,
            "description": p.description,
            "schedule_type": p.schedule_type,
            "created_at": p.created_at,
            "row_count": row_count,
        }));
    }

    Ok(Json(json!(result)))
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let p = sqlx::query_as::<_, PortfolioRow>(
        "SELECT id, name, description, schedule_type, created_at::TEXT
         FROM portfolios_v3 WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let rows: Vec<(String, Option<String>, String, String, i32)> = sqlx::query_as(
        "SELECT id, label, schedule_json, outstanding_json, row_order
         FROM portfolio_rows WHERE portfolio_id = $1 ORDER BY row_order",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let portfolio_rows: Vec<Value> = rows
        .into_iter()
        .map(|(rid, label, sched, out, order)| {
            json!({
                "id": rid,
                "label": label,
                "schedule_json": sched,
                "outstanding_json": out,
                "row_order": order,
            })
        })
        .collect();

    Ok(Json(json!({
        "id": p.id,
        "name": p.name,
        "description": p.description,
        "schedule_type": p.schedule_type,
        "created_at": p.created_at,
        "rows": portfolio_rows,
    })))
}

#[derive(Debug, Deserialize)]
pub struct CreatePortfolioRequest {
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: Option<String>,
}

pub async fn create_portfolio(
    State(state): State<AppState>,
    Json(body): Json<CreatePortfolioRequest>,
) -> Result<Json<Value>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let schedule_type = body.schedule_type.unwrap_or_else(|| "stock_amort".to_string());

    sqlx::query(
        "INSERT INTO portfolios_v3 (id, name, description, schedule_type) VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(&schedule_type)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "id": id,
        "name": body.name,
        "description": body.description,
        "schedule_type": schedule_type,
        "row_count": 0,
    })))
}

pub async fn delete_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM portfolios_v3 WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/portfolios-v3/:id/rows/upload
/// Multipart with fields: schedule (CSV file), outstanding (CSV file), label (optional text)
pub async fn upload_portfolio_row(
    State(state): State<AppState>,
    Path(portfolio_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    let mut schedule_csv: Option<String> = None;
    let mut outstanding_csv: Option<String> = None;
    let mut label: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "schedule" => {
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                schedule_csv = Some(String::from_utf8_lossy(&bytes).to_string());
            }
            "outstanding" => {
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                outstanding_csv = Some(String::from_utf8_lossy(&bytes).to_string());
            }
            "label" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                if !text.is_empty() { label = Some(text); }
            }
            _ => {}
        }
    }

    let schedule_content = schedule_csv.ok_or_else(|| StatusCode::BAD_REQUEST)?;
    let outstanding_content = outstanding_csv.ok_or_else(|| StatusCode::BAD_REQUEST)?;

    let (schedule_json, sched_rows) = parse_schedule_csv(&schedule_content)
        .map_err(|e| { tracing::warn!("Schedule CSV error: {}", e); StatusCode::UNPROCESSABLE_ENTITY })?;

    let (outstanding_json, out_rows) = parse_outstanding_csv(&outstanding_content)
        .map_err(|e| { tracing::warn!("Outstanding CSV error: {}", e); StatusCode::UNPROCESSABLE_ENTITY })?;

    if sched_rows != out_rows {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Get next row_order
    let max_order: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(row_order) FROM portfolio_rows WHERE portfolio_id = $1",
    )
    .bind(&portfolio_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(None);

    let next_order = max_order.map(|m| m + 1).unwrap_or(0);
    let row_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO portfolio_rows (id, portfolio_id, label, schedule_json, outstanding_json, row_order)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&row_id)
    .bind(&portfolio_id)
    .bind(&label)
    .bind(schedule_json.to_string())
    .bind(outstanding_json.to_string())
    .bind(next_order)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "id": row_id,
        "portfolio_id": portfolio_id,
        "label": label,
        "row_order": next_order,
        "date_count": sched_rows,
    })))
}

pub async fn delete_portfolio_row(
    State(state): State<AppState>,
    Path(row_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM portfolio_rows WHERE id = $1")
        .bind(&row_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/portfolios-v3/:id/rows/:row_id — returns parsed data for preview
pub async fn get_portfolio_row(
    State(state): State<AppState>,
    Path((portfolio_id, row_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    let row: (String, Option<String>, String, String, i32) = sqlx::query_as(
        "SELECT id, label, schedule_json, outstanding_json, row_order
         FROM portfolio_rows WHERE id = $1 AND portfolio_id = $2",
    )
    .bind(&row_id)
    .bind(&portfolio_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let sched: Value = serde_json::from_str(&row.2).unwrap_or(json!([]));
    let out: Value = serde_json::from_str(&row.3).unwrap_or(json!([]));

    Ok(Json(json!({
        "id": row.0,
        "label": row.1,
        "schedule": sched,
        "outstanding": out,
        "row_order": row.4,
        "tenors": FTP_TENORS,
    })))
}

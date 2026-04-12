use axum::{extract::State, http::StatusCode, Json};
use ftp_calculator_core::{ComputeMethod, FtpResult};
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::AppState;

#[derive(Debug, Deserialize)]
pub struct ComputeRequest {
    /// FTP method: "stock" | "flux" | "duration" | "pool" | "refinancing" |
    ///             "floating" | "behavioral" | "replicating" | "ldi"
    pub method: String,
    pub portfolio_id: String,
    pub label: Option<String>,
    pub curve_ids: Option<Vec<String>>,
    /// N×1 flat array of outstanding amounts
    pub outstanding_json: String,
    /// N×P matrix (rows = positions, cols = tenors)
    pub profiles_json: String,
    /// N×(P-1) matrix of rates (same row count, one fewer column than profiles)
    pub rates_json: String,
    pub parameters_json: Option<String>,
    pub seeds_json: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ComputeResponse {
    pub execution_id: String,
    pub status: String,
    pub method: String,
    pub duration_ms: i64,
    /// FTP rate matrix N×P (JSON-serialised Vec<Vec<f64>>)
    pub ftp_rate: Option<Vec<Vec<f64>>>,
    /// FTP interest matrix N×P
    pub ftp_int: Option<Vec<Vec<f64>>>,
    /// Market rate matrix N×P
    pub market_rate: Option<Vec<Vec<f64>>>,
    /// Stock amortisation matrix N×P
    pub stock_amort: Option<Vec<Vec<f64>>>,
    /// KPIs (computed from result)
    pub total_outstanding: f64,
    pub weighted_ftp_rate: f64,
    pub total_ftp_int_monthly: f64,
    pub error: Option<String>,
}

fn mat_to_vec(m: &Array2<f64>) -> Vec<Vec<f64>> {
    m.rows().into_iter().map(|r| r.to_vec()).collect()
}

fn parse_matrix(json: &str) -> Result<Vec<Vec<f64>>, String> {
    serde_json::from_str(json).map_err(|e| format!("Invalid matrix JSON: {e}"))
}

fn parse_flat(json: &str) -> Result<Vec<f64>, String> {
    serde_json::from_str(json).map_err(|e| format!("Invalid array JSON: {e}"))
}

pub async fn run_calculation(
    State(state): State<AppState>,
    Json(req): Json<ComputeRequest>,
) -> (StatusCode, Json<ComputeResponse>) {
    let start = std::time::Instant::now();
    let exec_id = Uuid::new_v4().to_string();
    let method_str = req.method.to_lowercase();

    // ── Parse inputs ─────────────────────────────────────────────────────────

    let outstanding_vec = match parse_flat(&req.outstanding_json) {
        Ok(v) => v,
        Err(e) => return error_response(exec_id, method_str, 0, e),
    };
    let profiles = match parse_matrix(&req.profiles_json) {
        Ok(v) => v,
        Err(e) => return error_response(exec_id, method_str, 0, e),
    };
    let rates = match parse_matrix(&req.rates_json) {
        Ok(v) => v,
        Err(e) => return error_response(exec_id, method_str, 0, e),
    };

    let nrows = outstanding_vec.len();
    let ncols = profiles.first().map(|r| r.len()).unwrap_or(0);
    let rate_cols = rates.first().map(|r| r.len()).unwrap_or(0);

    if nrows == 0 || ncols < 2 || rate_cols != ncols - 1 {
        return error_response(
            exec_id,
            method_str,
            0,
            format!("Dimension error: {nrows} positions, {ncols} profile cols, {rate_cols} rate cols (expected {ncols}-1)"),
        );
    }

    // ── Build ndarray matrices ────────────────────────────────────────────────

    let mut out_arr = Array2::<f64>::zeros((nrows, 1));
    for (i, v) in outstanding_vec.iter().enumerate() {
        out_arr[[i, 0]] = *v;
    }

    let mut prof_arr = Array2::<f64>::zeros((nrows, ncols));
    for (i, row) in profiles.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            if i < nrows && j < ncols {
                prof_arr[[i, j]] = *v;
            }
        }
    }

    let mut rate_arr = Array2::<f64>::zeros((nrows, rate_cols));
    for (i, row) in rates.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            if i < nrows && j < rate_cols {
                rate_arr[[i, j]] = *v;
            }
        }
    }

    // ── Map method string ─────────────────────────────────────────────────────

    let method = match method_str.as_str() {
        "stock"       => ComputeMethod::Stock,
        "flux"        => ComputeMethod::Flux,
        "duration"    => ComputeMethod::Duration,
        "pool"        => ComputeMethod::Pool,
        "refinancing" => ComputeMethod::Refinancing,
        "floating"    => ComputeMethod::Floating,
        "behavioral"  => ComputeMethod::Behavioral,
        "replicating" => ComputeMethod::Replicating,
        "ldi"         => ComputeMethod::Ldi,
        other => {
            let msg = format!("Unknown method: {other}");
            return error_response(exec_id, method_str, 0, msg);
        }
    };

    // ── Compute ───────────────────────────────────────────────────────────────

    let mut result = FtpResult::new(out_arr.clone(), prof_arr, rate_arr);
    match result.compute(method) {
        Err(e) => return error_response(exec_id, method_str, 0, e.to_string()),
        Ok(()) => {}
    }

    let duration_ms = start.elapsed().as_millis() as i64;

    let ftp_rate   = result.ftp_rate().map(mat_to_vec);
    let ftp_int    = result.ftp_int().map(mat_to_vec);
    let market_rate = result.market_rate().map(mat_to_vec);
    let stock_amort = result.stock_amort().map(mat_to_vec);

    // ── KPIs ──────────────────────────────────────────────────────────────────

    let total_outstanding: f64 = outstanding_vec.iter().sum();

    let weighted_ftp_rate = if let Some(fr) = &ftp_rate {
        let num: f64 = fr
            .iter()
            .zip(outstanding_vec.iter())
            .map(|(row, &o)| row.first().copied().unwrap_or(0.0) * o)
            .sum();
        if total_outstanding > 0.0 { num / total_outstanding } else { 0.0 }
    } else {
        0.0
    };

    let total_ftp_int_monthly: f64 = ftp_int
        .as_ref()
        .map(|m| m.iter().map(|row| row.first().copied().unwrap_or(0.0)).sum())
        .unwrap_or(0.0);

    // ── Ensure portfolio exists (pricer / ad-hoc calls) ──────────────────────

    let _ = sqlx::query(
        r#"INSERT INTO portfolios (id, name, as_of_date)
           VALUES ($1, $1, CURRENT_DATE)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(&req.portfolio_id)
    .execute(&state.pool)
    .await;

    // ── Persist execution ─────────────────────────────────────────────────────

    let result_payload = serde_json::json!({
        "ftp_rate":    ftp_rate,
        "ftp_int":     ftp_int,
        "market_rate": market_rate,
        "stock_amort": stock_amort,
        "kpis": {
            "total_outstanding":     total_outstanding,
            "weighted_ftp_rate":     weighted_ftp_rate,
            "total_ftp_int_monthly": total_ftp_int_monthly,
        }
    });

    let curve_ids_json = serde_json::to_string(
        &req.curve_ids.unwrap_or_default()
    ).unwrap_or_else(|_| "[]".to_string());

    let _ = sqlx::query(
        r#"INSERT INTO executions
           (id, label, method, portfolio_id, curve_ids_json, parameters_json,
            seeds_json, result_json, status, duration_ms,
            outstanding_json, profiles_json, rates_json)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'completed', $9, $10, $11, $12)"#,
    )
    .bind(&exec_id)
    .bind(&req.label)
    .bind(&req.method)
    .bind(&req.portfolio_id)
    .bind(&curve_ids_json)
    .bind(req.parameters_json.as_deref().unwrap_or("{}"))
    .bind(&req.seeds_json)
    .bind(result_payload.to_string())
    .bind(duration_ms)
    .bind(&req.outstanding_json)
    .bind(&req.profiles_json)
    .bind(&req.rates_json)
    .execute(&state.pool)
    .await;   // best-effort: don't fail compute if DB insert fails

    // ── Re-extract for response ───────────────────────────────────────────────

    let ftp_rate_out   = result.ftp_rate().map(mat_to_vec);
    let ftp_int_out    = result.ftp_int().map(mat_to_vec);
    let market_rate_out = result.market_rate().map(mat_to_vec);
    let stock_amort_out = result.stock_amort().map(mat_to_vec);

    (
        StatusCode::OK,
        Json(ComputeResponse {
            execution_id: exec_id,
            status: "completed".to_string(),
            method: req.method,
            duration_ms,
            ftp_rate: ftp_rate_out,
            ftp_int: ftp_int_out,
            market_rate: market_rate_out,
            stock_amort: stock_amort_out,
            total_outstanding,
            weighted_ftp_rate,
            total_ftp_int_monthly,
            error: None,
        }),
    )
}

fn error_response(
    exec_id: String,
    method: String,
    duration_ms: i64,
    msg: String,
) -> (StatusCode, Json<ComputeResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ComputeResponse {
            execution_id: exec_id,
            status: "error".to_string(),
            method,
            duration_ms,
            ftp_rate: None,
            ftp_int: None,
            market_rate: None,
            stock_amort: None,
            total_outstanding: 0.0,
            weighted_ftp_rate: 0.0,
            total_ftp_int_monthly: 0.0,
            error: Some(msg),
        }),
    )
}

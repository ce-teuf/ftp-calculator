use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use ftp_calculator_core::{ComputeMethod, FtpResult};
use ndarray::Array2;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::compute::interpolate;
use crate::db::AppState;

// ── Request ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RunRequest {
    pub study_id: String,
    pub label: Option<String>,
    /// "stock" | "flux" | "duration" — default "stock"
    pub method: Option<String>,
}

// ── Data helpers ──────────────────────────────────────────────────────────────

/// Find the outstanding value closest to `target_date` in a JSON array
/// `[{date: "YYYY-MM-DD", outstanding: f64}]`
fn outstanding_at(arr: &[Value], target: &str) -> f64 {
    arr.iter()
        .min_by_key(|v| {
            let d = v["date"].as_str().unwrap_or("");
            date_dist(d, target)
        })
        .and_then(|v| v["outstanding"].as_f64())
        .unwrap_or(0.0)
}

/// Find the schedule buckets closest to `target_date`
/// `[{date: "YYYY-MM-DD", buckets: [f64;12]}]`
fn schedule_at(arr: &[Value], target: &str) -> Vec<f64> {
    let entry = arr.iter().min_by_key(|v| {
        let d = v["date"].as_str().unwrap_or("");
        date_dist(d, target)
    });
    match entry {
        Some(v) => v["buckets"]
            .as_array()
            .map(|b| b.iter().filter_map(|x| x.as_f64()).collect())
            .unwrap_or_default(),
        None => vec![0.0; 12],
    }
}

/// Simple YYYY-MM distance (in months) for nearest-date lookup.
fn date_dist(a: &str, b: &str) -> u32 {
    fn ym(s: &str) -> (i32, i32) {
        let p: Vec<&str> = s.splitn(3, '-').collect();
        let y = p.first().and_then(|v| v.parse().ok()).unwrap_or(0);
        let m = p.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        (y, m)
    }
    let (ay, am) = ym(a);
    let (by, bm) = ym(b);
    ((ay - by).abs() * 12 + (am - bm).abs()) as u32
}

/// Generate monthly date strings between start and end with given step.
fn analysis_dates(start: &str, end: &str, step: i32) -> Vec<String> {
    fn ym(s: &str) -> (i32, i32) {
        let p: Vec<&str> = s.splitn(3, '-').collect();
        let y = p.first().and_then(|v| v.parse().ok()).unwrap_or(0);
        let m = p.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        (y, m)
    }
    let (sy, sm) = ym(start);
    let (ey, em) = ym(end);
    let total_months = (ey - sy) * 12 + (em - sm);
    if total_months < 0 || step <= 0 {
        return vec![start.to_string()];
    }
    let n = (total_months / step) as usize + 1;
    (0..n)
        .map(|i| {
            let months = sm - 1 + (i as i32) * step;
            let y = sy + months / 12;
            let m = months % 12 + 1;
            format!("{y:04}-{m:02}-01")
        })
        .collect()
}

fn method_from_str(s: &str) -> ComputeMethod {
    match s {
        "flux"        => ComputeMethod::Flux,
        "duration"    => ComputeMethod::Duration,
        "pool"        => ComputeMethod::Pool,
        "refinancing" => ComputeMethod::Refinancing,
        "floating"    => ComputeMethod::Floating,
        _             => ComputeMethod::Stock,
    }
}

// ── Core computation for one (linker, analysis_time) ─────────────────────────

struct LinkerAtTime {
    outstanding: Vec<f64>,      // one per portfolio row
    profiles:    Vec<Vec<f64>>, // one per portfolio row, 13 cols (1.0 prepended)
    rates:       [f64; 12],     // 12 values (summed stack components, interpolated)
}

fn run_ftp(data: LinkerAtTime, method: ComputeMethod) -> Result<Value, String> {
    let n = data.outstanding.len();
    if n == 0 { return Err("No portfolio rows".into()); }

    let ncols = data.profiles.first().map(|r| r.len()).unwrap_or(0);
    if ncols < 2 {
        return Err(format!("Profile has {ncols} columns, need ≥ 2"));
    }
    let rate_cols = ncols - 1;

    let mut out_arr = Array2::<f64>::zeros((n, 1));
    let mut prof_arr = Array2::<f64>::zeros((n, ncols));
    let mut rate_arr = Array2::<f64>::zeros((n, rate_cols));

    for i in 0..n {
        out_arr[[i, 0]] = data.outstanding[i];
        for (j, &v) in data.profiles[i].iter().enumerate() {
            if j < ncols { prof_arr[[i, j]] = v; }
        }
        // Same rates for all rows
        for (j, &v) in data.rates.iter().enumerate() {
            if j < rate_cols { rate_arr[[i, j]] = v; }
        }
    }

    let mut result = FtpResult::new(out_arr.clone(), prof_arr, rate_arr);
    result.compute(method).map_err(|e| e.to_string())?;

    let total_out: f64 = data.outstanding.iter().sum();
    let ftp_rate = result.ftp_rate();
    let ftp_int  = result.ftp_int();

    let weighted_ftp = if let Some(fr) = ftp_rate {
        let num: f64 = fr.rows().into_iter()
            .zip(data.outstanding.iter())
            .map(|(row, &o)| row.first().copied().unwrap_or(0.0) * o)
            .sum();
        if total_out > 0.0 { num / total_out } else { 0.0 }
    } else { 0.0 };

    let total_ftp_int: f64 = ftp_int
        .map(|m| m.rows().into_iter().map(|r| r.first().copied().unwrap_or(0.0)).sum())
        .unwrap_or(0.0);

    Ok(json!({
        "total_outstanding":     total_out,
        "weighted_ftp_rate":     weighted_ftp,
        "total_ftp_int_monthly": total_ftp_int,
    }))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn run_execution(
    State(state): State<AppState>,
    Json(body): Json<RunRequest>,
) -> Result<Json<Value>, StatusCode> {
    let exec_id = Uuid::new_v4().to_string();
    let method_str = body.method.clone().unwrap_or_else(|| "stock".into());
    let start = std::time::Instant::now();

    // ── Persist pending ───────────────────────────────────────────────────────
    sqlx::query(
        "INSERT INTO executions_v3 (id, study_id, label, method, status) VALUES ($1,$2,$3,$4,'running')",
    )
    .bind(&exec_id)
    .bind(&body.study_id)
    .bind(&body.label)
    .bind(&method_str)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // ── Load study linkers ────────────────────────────────────────────────────
    let linker_rows: Vec<(String, Option<String>, String, String, Option<String>)> =
        sqlx::query_as(
            r#"SELECT sl.linker_id, sl.label,
                      lk.portfolio_id, lk.cube_id, lk.start_date::TEXT
               FROM study_linkers sl
               JOIN linkers lk ON lk.id = sl.linker_id
               WHERE sl.study_id = $1
               ORDER BY sl.position"#,
        )
        .bind(&body.study_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let method = method_from_str(&method_str);
    let mut linker_results = vec![];
    let mut global_error: Option<String> = None;

    'linker: for (linker_id, linker_label, portfolio_id, cube_id, _start_date) in &linker_rows {
        // ── Load portfolio rows ───────────────────────────────────────────────
        let pf_rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT schedule_json, outstanding_json FROM portfolio_rows WHERE portfolio_id = $1 ORDER BY row_order",
        )
        .bind(portfolio_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        if pf_rows.is_empty() {
            linker_results.push(json!({
                "linker_id": linker_id,
                "label": linker_label,
                "error": "Portfolio has no rows",
                "analysis_times": [],
            }));
            continue;
        }

        // Parse portfolio rows once
        let parsed_rows: Vec<(Vec<Value>, Vec<Value>)> = pf_rows
            .iter()
            .map(|(sched, out)| {
                let s: Vec<Value> = serde_json::from_str(sched).unwrap_or_default();
                let o: Vec<Value> = serde_json::from_str(out).unwrap_or_default();
                (s, o)
            })
            .collect();

        // ── Load cube + stack ─────────────────────────────────────────────────
        let cube: Option<(String, String, i32, String)> = sqlx::query_as(
            "SELECT stack_id, analysis_start::TEXT, step_months, analysis_end::TEXT FROM curve_cubes WHERE id = $1",
        )
        .bind(cube_id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

        let (stack_id, ana_start, step, ana_end) = match cube {
            Some(c) => c,
            None => {
                linker_results.push(json!({ "linker_id": linker_id, "error": "Cube not found", "analysis_times": [] }));
                continue;
            }
        };

        // ── Load stack components (summed rates with interpolation) ──────────
        let components: Vec<(String, f32, String)> = sqlx::query_as(
            "SELECT curve_id, weight, interp_method
             FROM curve_stack_components WHERE stack_id = $1 ORDER BY position",
        )
        .bind(&stack_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        // Sum component curves → 12 rate values (interpolated to FTP tenors)
        let mut summed_rates = [0.0f64; 12];
        for (curve_id, weight, interp_method) in &components {
            let curve_data: Option<(String, String)> = sqlx::query_as(
                "SELECT tenors_json, values_json FROM rate_curves WHERE id = $1",
            )
            .bind(curve_id)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None);

            if let Some((tenors_json, values_json)) = curve_data {
                match interpolate::interpolate_to_ftp(&tenors_json, &values_json, interp_method) {
                    Ok(vals) => {
                        for i in 0..12 {
                            summed_rates[i] += vals[i] * (*weight as f64);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Interpolation error for curve {curve_id}: {e}");
                        // Fallback: try parsing values_json directly as 12 floats
                        if let Ok(vals) = serde_json::from_str::<Vec<f64>>(&values_json) {
                            for (i, v) in vals.iter().enumerate() {
                                if i < 12 { summed_rates[i] += v * (*weight as f64); }
                            }
                        }
                    }
                }
            }
        }

        // ── Iterate analysis times ────────────────────────────────────────────
        let dates = analysis_dates(&ana_start, &ana_end, step);
        let mut time_results = vec![];

        for date in &dates {
            // Build outstanding vector (one value per portfolio row)
            let outstanding: Vec<f64> = parsed_rows
                .iter()
                .map(|(_, out_arr)| outstanding_at(out_arr, date))
                .collect();

            // Build profile matrix (prepend 1.0 to each 12-bucket row → 13 cols)
            let profiles: Vec<Vec<f64>> = parsed_rows
                .iter()
                .map(|(sched_arr, _)| {
                    let mut p = vec![1.0f64];
                    p.extend(schedule_at(sched_arr, date));
                    p
                })
                .collect();

            let data = LinkerAtTime {
                outstanding,
                profiles,
                rates: summed_rates,
            };

            match run_ftp(data, method) {
                Ok(kpis) => time_results.push(json!({ "date": date, "kpis": kpis })),
                Err(e) => {
                    global_error = Some(e.clone());
                    time_results.push(json!({ "date": date, "error": e }));
                    break 'linker; // stop on first hard error
                }
            }
        }

        linker_results.push(json!({
            "linker_id": linker_id,
            "label": linker_label,
            "analysis_times": time_results,
        }));
    }

    let duration_ms = start.elapsed().as_millis() as i64;
    let status = if global_error.is_some() { "error" } else { "completed" };
    let result = json!({ "linkers": linker_results });

    sqlx::query(
        "UPDATE executions_v3 SET status=$1, result_json=$2, error_message=$3, duration_ms=$4 WHERE id=$5",
    )
    .bind(status)
    .bind(result.to_string())
    .bind(&global_error)
    .bind(duration_ms)
    .bind(&exec_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "id": exec_id,
        "status": status,
        "duration_ms": duration_ms,
        "error": global_error,
        "result": result,
    })))
}

pub async fn list_executions(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let rows: Vec<(String, Option<String>, Option<String>, String, String, Option<i64>, String)> =
        sqlx::query_as(
            r#"SELECT e.id, e.label, s.name AS study_name, e.method, e.status,
                      e.duration_ms, e.created_at::TEXT
               FROM executions_v3 e
               LEFT JOIN studies s ON s.id = e.study_id
               ORDER BY e.created_at DESC"#,
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<Value> = rows
        .into_iter()
        .map(|(id, label, study_name, method, status, dur, created_at)| {
            json!({
                "id": id,
                "label": label,
                "study_name": study_name,
                "method": method,
                "status": status,
                "duration_ms": dur,
                "created_at": created_at,
            })
        })
        .collect();

    Ok(Json(json!(result)))
}

pub async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let row: Option<(String, Option<String>, Option<String>, String, String, Option<String>, Option<String>, Option<i64>, String)> =
        sqlx::query_as(
            r#"SELECT e.id, e.label, s.name AS study_name, e.method, e.status,
                      e.result_json, e.error_message, e.duration_ms, e.created_at::TEXT
               FROM executions_v3 e
               LEFT JOIN studies s ON s.id = e.study_id
               WHERE e.id = $1"#,
        )
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (id, label, study_name, method, status, result_json, error_msg, dur, created_at) =
        row.ok_or(StatusCode::NOT_FOUND)?;

    let result: Value = result_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(json!(null));

    Ok(Json(json!({
        "id": id,
        "label": label,
        "study_name": study_name,
        "method": method,
        "status": status,
        "duration_ms": dur,
        "created_at": created_at,
        "error": error_msg,
        "result": result,
    })))
}

pub async fn delete_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM executions_v3 WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

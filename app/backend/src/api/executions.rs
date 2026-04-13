use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use ftp_calculator_core::{ComputeMethod, FtpResult};
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

use crate::compute::interpolate::{interp_at_tenors, tenor_to_months};
use crate::db::AppState;

// ── Error helper ──────────────────────────────────────────────────────────────

fn err(status: StatusCode, msg: impl ToString) -> (StatusCode, String) {
    (status, msg.to_string())
}

// ── Modèles publics ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ExecutionSummary {
    pub id: String,
    pub study_id: Option<String>,
    pub study_name: Option<String>,
    pub label: Option<String>,
    pub method: String,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionDetail {
    pub id: String,
    pub study_id: Option<String>,
    pub study_name: Option<String>,
    pub label: Option<String>,
    pub method: String,
    pub status: String,
    pub result: Option<JsonValue>,
    pub duration_ms: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Requêtes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateExecutionRequest {
    pub study_id: String,
    pub label: Option<String>,
}

// ── Structs sqlx ─────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct ExecutionRow {
    id: String,
    study_id: Option<String>,
    study_name: Option<String>,
    label: Option<String>,
    method: String,
    status: String,
    result_json: Option<String>,
    duration_ms: Option<i64>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_executions(
    State(state): State<AppState>,
) -> Result<Json<Vec<ExecutionSummary>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, ExecutionRow>(
        "SELECT id, study_id, study_name, label, method, status,
                NULL::text AS result_json, duration_ms, error_message, created_at
         FROM executions ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(
        rows.into_iter()
            .map(|r| ExecutionSummary {
                id: r.id,
                study_id: r.study_id,
                study_name: r.study_name,
                label: r.label,
                method: r.method,
                status: r.status,
                duration_ms: r.duration_ms,
                error_message: r.error_message,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

pub async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ExecutionDetail>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, ExecutionRow>(
        "SELECT id, study_id, study_name, label, method, status,
                result_json, duration_ms, error_message, created_at
         FROM executions WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Exécution introuvable"))?;

    let result = row
        .result_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

    Ok(Json(ExecutionDetail {
        id: row.id,
        study_id: row.study_id,
        study_name: row.study_name,
        label: row.label,
        method: row.method,
        status: row.status,
        result,
        duration_ms: row.duration_ms,
        error_message: row.error_message,
        created_at: row.created_at,
    }))
}

pub async fn create_execution(
    State(state): State<AppState>,
    Json(body): Json<CreateExecutionRequest>,
) -> Result<Json<ExecutionDetail>, (StatusCode, String)> {
    if body.study_id.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "study_id est requis"));
    }

    // Snapshot du nom de l'étude
    let study_name: Option<String> =
        sqlx::query_scalar("SELECT name FROM studies WHERE id = $1")
            .bind(&body.study_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if study_name.is_none() {
        return Err(err(StatusCode::BAD_REQUEST, "Étude introuvable"));
    }

    let exec_id = Uuid::new_v4().to_string();
    let started_at = Utc::now();

    // Persister avec status "running"
    sqlx::query(
        "INSERT INTO executions (id, study_id, study_name, label, status)
         VALUES ($1, $2, $3, $4, 'running')",
    )
    .bind(&exec_id)
    .bind(&body.study_id)
    .bind(&study_name)
    .bind(&body.label)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // ── Lancer le calcul ──────────────────────────────────────────────────────
    let engine_result = run_engine(&state, &body.study_id).await;
    let elapsed_ms = (Utc::now() - started_at).num_milliseconds();

    match engine_result {
        Ok((result_value, method_used)) => {
            let result_text = serde_json::to_string(&result_value).unwrap_or_default();
            sqlx::query(
                "UPDATE executions
                 SET status = 'completed', result_json = $1, duration_ms = $2, method = $3
                 WHERE id = $4",
            )
            .bind(&result_text)
            .bind(elapsed_ms)
            .bind(&method_used)
            .bind(&exec_id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

            Ok(Json(ExecutionDetail {
                id: exec_id,
                study_id: Some(body.study_id),
                study_name,
                label: body.label,
                method: method_used,
                status: "completed".to_string(),
                result: Some(result_value),
                duration_ms: Some(elapsed_ms),
                error_message: None,
                created_at: started_at,
            }))
        }
        Err(msg) => {
            sqlx::query(
                "UPDATE executions
                 SET status = 'error', error_message = $1, duration_ms = $2
                 WHERE id = $3",
            )
            .bind(&msg)
            .bind(elapsed_ms)
            .bind(&exec_id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

            Ok(Json(ExecutionDetail {
                id: exec_id,
                study_id: Some(body.study_id),
                study_name,
                label: body.label,
                method: "error".to_string(),
                status: "error".to_string(),
                result: None,
                duration_ms: Some(elapsed_ms),
                error_message: Some(msg),
                created_at: started_at,
            }))
        }
    }
}

pub async fn delete_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM executions WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if res.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Exécution introuvable"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ═════════════════════════════════════════════════════════════════════════════
// Moteur de calcul — Maturity Matching
// ═════════════════════════════════════════════════════════════════════════════

// ── Structures de travail ─────────────────────────────────────────────────────

struct VectorMap {
    name: String,
    /// date "YYYY-MM" → encours
    dates: HashMap<String, f64>,
    /// date "YYYY-MM" → "observed" | "projected" | "contrafactual"
    period_types: HashMap<String, String>,
}

struct ScheduleMap {
    name: String,
    bucket_labels: Vec<String>,
    bucket_months: Vec<f64>,
    /// date "YYYY-MM" → vecteur de poids (même longueur que bucket_labels)
    dates: HashMap<String, Vec<f64>>,
}

struct MatrixData {
    /// tenors en mois, triés
    tenor_months: Vec<f64>,
    interp_method: String,
    /// date "YYYY-MM" → valeurs (même longueur que tenor_months)
    dates: HashMap<String, Vec<f64>>,
}

struct AssignmentData {
    id: String,
    pair_id: String,
    pair_label: Option<String>,
    vector_id: String,
    schedule_id: String,
    combination_matrix_ids: Vec<String>,
    is_existing_stock: bool,
    /// tenor_label → taux initial (profil FTP pour t=t₀)
    initial_ftp_profile: HashMap<String, f64>,
    /// méthodes de calcul sélectionnées (e.g. ["Stock"] ou ["Flux"])
    methods: Vec<String>,
}

// ── Chargements depuis la DB ──────────────────────────────────────────────────

async fn load_vector(pool: &sqlx::PgPool, id: &str) -> Result<VectorMap, String> {
    #[derive(sqlx::FromRow)]
    struct Row { name: String, rows_json: String }

    let row = sqlx::query_as::<_, Row>(
        "SELECT name, rows_json FROM outstanding_vectors WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load_vector DB error: {e}"))?
    .ok_or_else(|| format!("Vecteur introuvable : {id}"))?;

    let raw: Vec<JsonValue> = serde_json::from_str(&row.rows_json)
        .map_err(|e| format!("parse vector rows_json: {e}"))?;

    let mut dates        = HashMap::new();
    let mut period_types = HashMap::new();
    for item in &raw {
        if let (Some(date), Some(val)) = (
            item["date"].as_str(),
            item["value"].as_f64(),
        ) {
            dates.insert(date.to_string(), val);
            let pt = item["period_type"].as_str().unwrap_or("observed").to_string();
            period_types.insert(date.to_string(), pt);
        }
    }

    Ok(VectorMap { name: row.name, dates, period_types })
}

async fn load_schedule(pool: &sqlx::PgPool, id: &str) -> Result<ScheduleMap, String> {
    #[derive(sqlx::FromRow)]
    struct Row { name: String, bucket_labels_json: String, rows_json: String }

    let row = sqlx::query_as::<_, Row>(
        "SELECT name, bucket_labels_json, rows_json FROM amort_schedules WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load_schedule DB error: {e}"))?
    .ok_or_else(|| format!("Schedule introuvable : {id}"))?;

    let bucket_labels: Vec<String> = serde_json::from_str(&row.bucket_labels_json)
        .map_err(|e| format!("parse bucket_labels_json: {e}"))?;

    let bucket_months: Vec<f64> = bucket_labels
        .iter()
        .map(|l| tenor_to_months(l).unwrap_or(0.0))
        .collect();

    let raw: Vec<JsonValue> = serde_json::from_str(&row.rows_json)
        .map_err(|e| format!("parse schedule rows_json: {e}"))?;

    let mut dates = HashMap::new();
    for item in &raw {
        if let Some(date) = item["date"].as_str() {
            if let Some(arr) = item["buckets"].as_array() {
                let buckets: Vec<f64> = arr.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                dates.insert(date.to_string(), buckets);
            }
        }
    }

    Ok(ScheduleMap { name: row.name, bucket_labels, bucket_months, dates })
}

async fn load_matrix(pool: &sqlx::PgPool, id: &str) -> Result<MatrixData, String> {
    #[derive(sqlx::FromRow)]
    struct Row { tenors_json: String, rows_json: String, interp_method: String }

    let row = sqlx::query_as::<_, Row>(
        "SELECT tenors_json, rows_json, interp_method FROM rate_matrices WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load_matrix DB error: {e}"))?
    .ok_or_else(|| format!("Matrice introuvable : {id}"))?;

    let tenor_strs: Vec<String> = serde_json::from_str(&row.tenors_json)
        .map_err(|e| format!("parse tenors_json: {e}"))?;

    // Paires (mois, index) triées par mois croissants
    let mut pairs: Vec<(f64, usize)> = tenor_strs
        .iter()
        .enumerate()
        .filter_map(|(i, s)| tenor_to_months(s).map(|m| (m, i)))
        .collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let tenor_months: Vec<f64> = pairs.iter().map(|p| p.0).collect();
    let sorted_indices: Vec<usize> = pairs.iter().map(|p| p.1).collect();

    let raw: Vec<JsonValue> = serde_json::from_str(&row.rows_json)
        .map_err(|e| format!("parse matrix rows_json: {e}"))?;

    let mut dates = HashMap::new();
    for item in &raw {
        if let Some(date) = item["date"].as_str() {
            if let Some(arr) = item["values"].as_array() {
                let all_vals: Vec<f64> = arr.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                // Re-order values according to sorted tenor indices
                let sorted_vals: Vec<f64> = sorted_indices
                    .iter()
                    .filter_map(|&i| all_vals.get(i).copied())
                    .collect();
                if sorted_vals.len() == tenor_months.len() {
                    dates.insert(date.to_string(), sorted_vals);
                }
            }
        }
    }

    Ok(MatrixData { tenor_months, interp_method: row.interp_method, dates })
}

// ── Génération des pas de temps mensuels ────────────────────────────────────

fn monthly_steps(start: NaiveDate, end: NaiveDate) -> Vec<String> {
    let mut steps = Vec::new();
    let end_ym = end.format("%Y-%m").to_string();
    let mut y = start.year();
    let mut m = start.month();

    loop {
        let ym = format!("{y:04}-{m:02}");
        steps.push(ym.clone());
        if ym >= end_ym {
            break;
        }
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }
    steps
}

// ── Moteur principal ──────────────────────────────────────────────────────────

async fn run_engine(state: &AppState, study_id: &str) -> Result<(JsonValue, String), String> {
    // ── 1. Lire les study units de l'étude ───────────────────────────────────

    #[derive(sqlx::FromRow)]
    struct StudyItemRow { study_unit_id: String }

    let items = sqlx::query_as::<_, StudyItemRow>(
        "SELECT study_unit_id FROM study_items WHERE study_id = $1 ORDER BY position",
    )
    .bind(study_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("load study items: {e}"))?;

    if items.is_empty() {
        return Err("L'étude ne contient aucune study unit".to_string());
    }

    let mut study_unit_results: Vec<JsonValue> = Vec::new();

    // ── 2. Pour chaque study unit ─────────────────────────────────────────────

    for item in &items {
        let su_id = &item.study_unit_id;

        // Charger les métadonnées de la study unit
        #[derive(sqlx::FromRow)]
        struct SuRow {
            name: String,
            hypercube_id: String,
            start_date: NaiveDate,
        }

        let su = sqlx::query_as::<_, SuRow>(
            "SELECT name, hypercube_id, start_date FROM study_units WHERE id = $1",
        )
        .bind(su_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| format!("load study unit {su_id}: {e}"))?
        .ok_or_else(|| format!("Study unit introuvable : {su_id}"))?;

        let start_ym = su.start_date.format("%Y-%m").to_string();

        // Charger le hypercube
        #[derive(sqlx::FromRow)]
        struct HcRow {
            start_date: NaiveDate,
            end_date: NaiveDate,
            proj_end_date: Option<NaiveDate>,
        }

        let hc = sqlx::query_as::<_, HcRow>(
            "SELECT start_date, end_date, proj_end_date FROM hypercubes WHERE id = $1",
        )
        .bind(&su.hypercube_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| format!("load hypercube {}: {e}", su.hypercube_id))?
        .ok_or_else(|| format!("Hypercube introuvable : {}", su.hypercube_id))?;

        let horizon = hc.proj_end_date.unwrap_or(hc.end_date);
        let time_steps = monthly_steps(hc.start_date, horizon);

        // Charger les assignments
        #[derive(sqlx::FromRow)]
        struct AssignRow {
            id: String,
            pair_id: String,
            pair_label: Option<String>,
            vector_id: String,
            schedule_id: String,
            combination_matrix_ids: String,
            is_existing_stock: bool,
            initial_ftp_profile_json: Option<String>,
            methods_json: String,
        }

        let assign_rows = sqlx::query_as::<_, AssignRow>(
            "SELECT sua.id, sua.pair_id, pp.label AS pair_label,
                    pp.vector_id, pp.schedule_id,
                    sua.combination_matrix_ids, sua.is_existing_stock,
                    sua.initial_ftp_profile_json, sua.methods_json
             FROM study_unit_assignments sua
             JOIN portfolio_pairs pp ON pp.id = sua.pair_id
             WHERE sua.study_unit_id = $1
             ORDER BY sua.created_at",
        )
        .bind(su_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| format!("load assignments for {su_id}: {e}"))?;

        if assign_rows.is_empty() {
            return Err(format!(
                "Study unit '{}' n'a aucun assignment",
                su.name
            ));
        }

        // Convertir les AssignRow → AssignmentData
        let mut assignments: Vec<AssignmentData> = Vec::new();
        for ar in assign_rows {
            let combo_ids: Vec<String> =
                serde_json::from_str(&ar.combination_matrix_ids).unwrap_or_default();

            let initial_ftp: HashMap<String, f64> = ar
                .initial_ftp_profile_json
                .as_deref()
                .and_then(|s| serde_json::from_str::<Vec<JsonValue>>(s).ok())
                .map(|arr| {
                    arr.into_iter()
                        .filter_map(|e| {
                            let tenor = e["tenor"].as_str()?.to_string();
                            let rate  = e["rate"].as_f64()?;
                            Some((tenor, rate))
                        })
                        .collect()
                })
                .unwrap_or_default();

            let methods: Vec<String> =
                serde_json::from_str(&ar.methods_json).unwrap_or_else(|_| vec!["Stock".to_string()]);

            assignments.push(AssignmentData {
                id: ar.id,
                pair_id: ar.pair_id,
                pair_label: ar.pair_label,
                vector_id: ar.vector_id,
                schedule_id: ar.schedule_id,
                combination_matrix_ids: combo_ids,
                is_existing_stock: ar.is_existing_stock,
                initial_ftp_profile: initial_ftp,
                methods,
            });
        }

        // ── 3. Pour chaque assignment ─────────────────────────────────────────

        let mut assignment_results: Vec<JsonValue> = Vec::new();

        for assign in &assignments {
            // Charger vecteur et schedule
            let vector   = load_vector(&state.pool, &assign.vector_id).await
                .map_err(|e| format!("assignment {}: {e}", assign.id))?;
            let schedule = load_schedule(&state.pool, &assign.schedule_id).await
                .map_err(|e| format!("assignment {}: {e}", assign.id))?;

            // Charger toutes les matrices de la combinaison
            let mut matrices: Vec<MatrixData> = Vec::new();
            for mid in &assign.combination_matrix_ids {
                let m = load_matrix(&state.pool, mid).await
                    .map_err(|e| format!("assignment {} matrice {mid}: {e}", assign.id))?;
                matrices.push(m);
            }

            // ── 4. Sélectionner la méthode de calcul ──────────────────────────

            // Seules Stock et Flux sont supportées pour l'instant
            let compute_method = match assign.methods.first().map(|s| s.as_str()) {
                Some("Flux") => ComputeMethod::Flux,
                _            => ComputeMethod::Stock,
            };
            let method_label = match compute_method {
                ComputeMethod::Flux  => "Matched Maturity (Flux)",
                _                   => "Matched Maturity (Stock)",
            };

            // ── 5. Construire les matrices d'entrée ───────────────────────────

            // Pas de temps alignés (données présentes pour vecteur ET schedule)
            let aligned_dates: Vec<String> = time_steps
                .iter()
                .filter(|d| {
                    vector.dates.contains_key(*d) && schedule.dates.contains_key(*d)
                })
                .cloned()
                .collect();

            if aligned_dates.is_empty() {
                continue;
            }

            let n_time    = aligned_dates.len();
            let n_buckets = schedule.bucket_labels.len();

            if n_buckets == 0 {
                continue;
            }

            // outstanding_arr : T × 1
            let mut outstanding_arr = Array2::<f64>::zeros((n_time, 1));
            // profiles_arr    : T × (B+1)  — profils de balance résiduelle décroissants
            let mut profiles_arr    = Array2::<f64>::zeros((n_time, n_buckets + 1));
            // rates_arr       : T × B      — taux FTP interpolés par tenor bucket
            let mut rates_arr       = Array2::<f64>::zeros((n_time, n_buckets));

            for (i, date) in aligned_dates.iter().enumerate() {
                outstanding_arr[[i, 0]] = *vector.dates.get(date).unwrap();

                // Conversion poids d'amortissement → profil balance résiduelle
                // w[j] = fraction qui s'amortit dans bucket j (Σw ≈ 1)
                // P[0] = 1.0, P[j] = P[j-1] - w[j-1]
                let weights = schedule.dates.get(date).unwrap();
                let mut remaining = 1.0_f64;
                profiles_arr[[i, 0]] = remaining;
                for j in 0..n_buckets {
                    remaining -= weights.get(j).copied().unwrap_or(0.0);
                    profiles_arr[[i, j + 1]] = remaining.max(0.0);
                }

                // Taux : profil FTP initial à t0 (stock existant) ou interpolation matrice
                let is_t0 = date == &start_ym;
                if is_t0 && assign.is_existing_stock && !assign.initial_ftp_profile.is_empty() {
                    for (j, label) in schedule.bucket_labels.iter().enumerate() {
                        rates_arr[[i, j]] =
                            assign.initial_ftp_profile.get(label).copied().unwrap_or(0.0);
                    }
                } else {
                    let mut combined = vec![0.0_f64; n_buckets];
                    for matrix in &matrices {
                        if let Some(mat_values) = matrix.dates.get(date) {
                            let interp_vals = interp_at_tenors(
                                &matrix.tenor_months,
                                mat_values,
                                &schedule.bucket_months,
                                &matrix.interp_method,
                            );
                            for (j, v) in interp_vals.iter().enumerate() {
                                combined[j] += v;
                            }
                        }
                    }
                    for j in 0..n_buckets {
                        rates_arr[[i, j]] = combined[j];
                    }
                }
            }

            // ── 6. Calcul FTP via ftp-calculator-core ─────────────────────────

            let mut ftp_result = FtpResult::new(
                outstanding_arr.clone(),
                profiles_arr,
                rates_arr.clone(),
            );
            ftp_result.compute(compute_method)
                .map_err(|e| format!("assignment {} FTP compute error: {e}", assign.id))?;

            let ftp_rate_mat      = ftp_result.ftp_rate().unwrap();
            let ftp_int_mat       = ftp_result.ftp_int().unwrap();
            let stock_amort_mat   = ftp_result.stock_amort().unwrap();
            let stock_instal_mat  = ftp_result.stock_instal().unwrap();
            let varstock_amort_mat  = ftp_result.varstock_amort().unwrap();
            let varstock_instal_mat = ftp_result.varstock_instal().unwrap();
            let market_rate_mat   = ftp_result.market_rate().unwrap();

            // Helper : extrait la ligne i d'une matrice Array2 en Vec<f64>
            let row_f64 = |mat: &ndarray::Array2<f64>, i: usize| -> Vec<f64> {
                mat.row(i).iter().map(|&v| (v * 1e8).round() / 1e8).collect()
            };

            // ── 7. Sérialiser les résultats par pas de temps ──────────────────

            let mut time_step_results: Vec<JsonValue> = Vec::new();

            for (i, date) in aligned_dates.iter().enumerate() {
                let outstanding  = outstanding_arr[[i, 0]];
                let ftp_rate     = ftp_rate_mat[[i, 0]];
                let ftp_interest = ftp_int_mat[[i, 0]];

                // period_type depuis le vecteur d'encours
                let period_type = vector.period_types
                    .get(date)
                    .map(|s| s.as_str())
                    .unwrap_or("observed");

                // ftp_by_tenor : taux interpolés par tenor bucket
                let ftp_obj: serde_json::Map<String, JsonValue> = schedule
                    .bucket_labels
                    .iter()
                    .enumerate()
                    .map(|(j, l)| {
                        (l.clone(), JsonValue::from((rates_arr[[i, j]] * 1e8).round() / 1e8))
                    })
                    .collect();

                // profile : poids d'amortissement du schedule à cette date
                let profile_vec: Vec<f64> = schedule
                    .dates
                    .get(date)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| (v * 1e8).round() / 1e8)
                    .collect();

                time_step_results.push(serde_json::json!({
                    "date":        date,
                    "period_type": period_type,
                    "kpis": {
                        "total_outstanding":     (outstanding  * 100.0).round() / 100.0,
                        "weighted_ftp_rate":     (ftp_rate     * 1e8).round()   / 1e8,
                        "ftp_interest_periodic": (ftp_interest * 100.0).round() / 100.0,
                    },
                    "ftp_by_tenor": JsonValue::Object(ftp_obj),
                    "profile":      profile_vec,
                    "matrices": {
                        "stock_amort":      row_f64(stock_amort_mat,    i),
                        "stock_instal":     row_f64(stock_instal_mat,   i),
                        "varstock_amort":   row_f64(varstock_amort_mat,  i),
                        "varstock_instal":  row_f64(varstock_instal_mat, i),
                        "ftp_rate":         row_f64(ftp_rate_mat,       i),
                        "ftp_int":          row_f64(ftp_int_mat,        i),
                        "market_rate":      row_f64(market_rate_mat,    i),
                    },
                }));
            }

            assignment_results.push(serde_json::json!({
                "assignment_id":          assign.id,
                "pair_id":                assign.pair_id,
                "pair_label":             assign.pair_label,
                "method":                 method_label,
                "vector_name":            vector.name,
                "schedule_name":          schedule.name,
                "bucket_labels":          schedule.bucket_labels,
                "combination_matrix_ids": assign.combination_matrix_ids,
                "time_step_count":        time_step_results.len(),
                "time_steps":             time_step_results,
            }));
        }

        study_unit_results.push(serde_json::json!({
            "study_unit_id":   su_id,
            "study_unit_name": su.name,
            "hypercube_id":    su.hypercube_id,
            "time_step_range": {
                "start": time_steps.first(),
                "end":   time_steps.last(),
                "count": time_steps.len(),
            },
            "assignments": assignment_results,
        }));
    }

    // Collecter les méthodes distinctes utilisées dans tous les assignments
    let mut methods_used: Vec<String> = study_unit_results
        .iter()
        .flat_map(|su| {
            su["assignments"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|a| a["method"].as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .collect();
    methods_used.sort();
    methods_used.dedup();
    let method_summary = if methods_used.is_empty() {
        "Stock".to_string()
    } else {
        methods_used.join("+")
    };

    Ok((serde_json::json!({ "study_units": study_unit_results }), method_summary))
}

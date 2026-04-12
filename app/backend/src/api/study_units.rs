use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::AppState;

// ── Error helper ──────────────────────────────────────────────────────────────

fn err(status: StatusCode, msg: impl ToString) -> (StatusCode, String) {
    (status, msg.to_string())
}

// ── Modèles publics ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct StudyUnitSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub hypercube_id: String,
    pub hypercube_name: String,
    pub portfolio_id: String,
    pub portfolio_name: String,
    pub start_date: NaiveDate,
    pub granularity_rule: String,
    pub is_valid: bool,
    pub assignment_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AssignmentInfo {
    pub id: String,
    pub pair_id: String,
    pub pair_label: Option<String>,
    pub vector_id: String,
    pub vector_name: String,
    pub schedule_id: String,
    pub schedule_name: String,
    pub combination_matrix_ids: Vec<String>,
    pub label: Option<String>,
    pub is_existing_stock: bool,
    pub initial_ftp_profile_json: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StudyUnitDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub hypercube_id: String,
    pub portfolio_id: String,
    pub start_date: NaiveDate,
    pub granularity_rule: String,
    pub is_valid: bool,
    pub validation_log: Option<String>,
    pub assignments: Vec<AssignmentInfo>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ValidationCheck {
    pub check: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub checks: Vec<ValidationCheck>,
}

// ── Requêtes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateStudyUnitRequest {
    pub name: String,
    pub description: Option<String>,
    pub hypercube_id: String,
    pub portfolio_id: String,
    pub start_date: NaiveDate,
    pub granularity_rule: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStudyUnitRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub granularity_rule: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    pub pair_id: String,
    pub combination_matrix_ids: Vec<String>,
    pub label: Option<String>,
    pub is_existing_stock: Option<bool>,
    pub initial_ftp_profile_json: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignmentRequest {
    pub combination_matrix_ids: Option<Vec<String>>,
    pub label: Option<String>,
    pub is_existing_stock: Option<bool>,
    pub initial_ftp_profile_json: Option<JsonValue>,
}

// ── Structs sqlx ─────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct StudyUnitRow {
    id: String,
    name: String,
    description: Option<String>,
    hypercube_id: String,
    portfolio_id: String,
    start_date: NaiveDate,
    granularity_rule: String,
    is_valid: bool,
    validation_log: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct AssignmentRow {
    id: String,
    pair_id: String,
    pair_label: Option<String>,
    vector_id: String,
    vector_name: String,
    schedule_id: String,
    schedule_name: String,
    combination_matrix_ids: String,
    label: Option<String>,
    is_existing_stock: bool,
    initial_ftp_profile_json: Option<String>,
    created_at: DateTime<Utc>,
}

// ── Helpers internes ──────────────────────────────────────────────────────────

async fn fetch_detail(
    state: &AppState,
    id: &str,
) -> Result<Json<StudyUnitDetail>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, StudyUnitRow>(
        "SELECT id, name, description, hypercube_id, portfolio_id, start_date,
                granularity_rule, is_valid, validation_log, created_at
         FROM study_units WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Study unit introuvable"))?;

    let assignments = load_assignments(state, id).await?;

    Ok(Json(StudyUnitDetail {
        id: row.id,
        name: row.name,
        description: row.description,
        hypercube_id: row.hypercube_id,
        portfolio_id: row.portfolio_id,
        start_date: row.start_date,
        granularity_rule: row.granularity_rule,
        is_valid: row.is_valid,
        validation_log: row.validation_log,
        assignments,
        created_at: row.created_at,
    }))
}

async fn load_assignments(
    state: &AppState,
    study_unit_id: &str,
) -> Result<Vec<AssignmentInfo>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AssignmentRow>(
        "SELECT sua.id, sua.pair_id, pp.label AS pair_label,
                pp.vector_id, ov.name AS vector_name,
                pp.schedule_id, ams.name AS schedule_name,
                sua.combination_matrix_ids, sua.label,
                sua.is_existing_stock, sua.initial_ftp_profile_json,
                sua.created_at
         FROM study_unit_assignments sua
         JOIN portfolio_pairs pp       ON pp.id  = sua.pair_id
         JOIN outstanding_vectors ov   ON ov.id  = pp.vector_id
         JOIN amort_schedules ams       ON ams.id = pp.schedule_id
         WHERE sua.study_unit_id = $1
         ORDER BY sua.created_at",
    )
    .bind(study_unit_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut result = Vec::new();
    for r in rows {
        let combo_ids: Vec<String> =
            serde_json::from_str(&r.combination_matrix_ids).unwrap_or_default();
        let ftp_profile: Option<JsonValue> = r
            .initial_ftp_profile_json
            .and_then(|s| serde_json::from_str(&s).ok());
        result.push(AssignmentInfo {
            id: r.id,
            pair_id: r.pair_id,
            pair_label: r.pair_label,
            vector_id: r.vector_id,
            vector_name: r.vector_name,
            schedule_id: r.schedule_id,
            schedule_name: r.schedule_name,
            combination_matrix_ids: combo_ids,
            label: r.label,
            is_existing_stock: r.is_existing_stock,
            initial_ftp_profile_json: ftp_profile,
            created_at: r.created_at,
        });
    }
    Ok(result)
}

async fn reset_validity(state: &AppState, id: &str) -> Result<(), (StatusCode, String)> {
    sqlx::query(
        "UPDATE study_units SET is_valid = false, validation_log = NULL WHERE id = $1",
    )
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(())
}

// ── Handlers — Study units ────────────────────────────────────────────────────

pub async fn list_study_units(
    State(state): State<AppState>,
) -> Result<Json<Vec<StudyUnitSummary>>, (StatusCode, String)> {
    #[derive(sqlx::FromRow)]
    struct SummaryRow {
        id: String,
        name: String,
        description: Option<String>,
        hypercube_id: String,
        hypercube_name: String,
        portfolio_id: String,
        portfolio_name: String,
        start_date: NaiveDate,
        granularity_rule: String,
        is_valid: bool,
        assignment_count: i64,
        created_at: DateTime<Utc>,
    }

    let rows = sqlx::query_as::<_, SummaryRow>(
        r#"
        SELECT su.id, su.name, su.description,
               su.hypercube_id, h.name  AS hypercube_name,
               su.portfolio_id, p.name  AS portfolio_name,
               su.start_date, su.granularity_rule, su.is_valid,
               COUNT(sua.id)            AS assignment_count,
               su.created_at
        FROM study_units su
        JOIN hypercubes h ON h.id = su.hypercube_id
        JOIN portfolios p ON p.id = su.portfolio_id
        LEFT JOIN study_unit_assignments sua ON sua.study_unit_id = su.id
        GROUP BY su.id, h.name, p.name
        ORDER BY su.created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(
        rows.into_iter()
            .map(|r| StudyUnitSummary {
                id: r.id,
                name: r.name,
                description: r.description,
                hypercube_id: r.hypercube_id,
                hypercube_name: r.hypercube_name,
                portfolio_id: r.portfolio_id,
                portfolio_name: r.portfolio_name,
                start_date: r.start_date,
                granularity_rule: r.granularity_rule,
                is_valid: r.is_valid,
                assignment_count: r.assignment_count,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

pub async fn create_study_unit(
    State(state): State<AppState>,
    Json(body): Json<CreateStudyUnitRequest>,
) -> Result<Json<StudyUnitDetail>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Le nom est requis"));
    }
    if body.hypercube_id.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "hypercube_id est requis"));
    }
    if body.portfolio_id.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "portfolio_id est requis"));
    }

    let hc: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hypercubes WHERE id = $1")
        .bind(&body.hypercube_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if hc == 0 {
        return Err(err(StatusCode::BAD_REQUEST, "Hypercube introuvable"));
    }

    let p: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portfolios WHERE id = $1")
        .bind(&body.portfolio_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if p == 0 {
        return Err(err(StatusCode::BAD_REQUEST, "Portfolio introuvable"));
    }

    let id = Uuid::new_v4().to_string();
    let rule = body.granularity_rule.as_deref().unwrap_or("none");

    sqlx::query(
        "INSERT INTO study_units
             (id, name, description, hypercube_id, portfolio_id, start_date, granularity_rule)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&id)
    .bind(body.name.trim())
    .bind(&body.description)
    .bind(&body.hypercube_id)
    .bind(&body.portfolio_id)
    .bind(body.start_date)
    .bind(rule)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    fetch_detail(&state, &id).await
}

pub async fn get_study_unit(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StudyUnitDetail>, (StatusCode, String)> {
    fetch_detail(&state, &id).await
}

pub async fn update_study_unit(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateStudyUnitRequest>,
) -> Result<Json<StudyUnitDetail>, (StatusCode, String)> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM study_units WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if count == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Study unit introuvable"));
    }

    if let Some(v) = &body.name {
        sqlx::query("UPDATE study_units SET name = $1 WHERE id = $2")
            .bind(v)
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.description {
        sqlx::query("UPDATE study_units SET description = $1 WHERE id = $2")
            .bind(v)
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = body.start_date {
        sqlx::query("UPDATE study_units SET start_date = $1 WHERE id = $2")
            .bind(v)
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.granularity_rule {
        sqlx::query("UPDATE study_units SET granularity_rule = $1 WHERE id = $2")
            .bind(v)
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    reset_validity(&state, &id).await?;
    fetch_detail(&state, &id).await
}

pub async fn delete_study_unit(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM study_units WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if res.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Study unit introuvable"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ── Handler — Validation ──────────────────────────────────────────────────────

pub async fn validate_study_unit(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ValidationReport>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, StudyUnitRow>(
        "SELECT id, name, description, hypercube_id, portfolio_id, start_date,
                granularity_rule, is_valid, validation_log, created_at
         FROM study_units WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Study unit introuvable"))?;

    let mut checks: Vec<ValidationCheck> = Vec::new();

    // ── Charger le hypercube ─────────────────────────────────────────────────

    #[derive(sqlx::FromRow)]
    struct HcRow {
        start_date: NaiveDate,
        end_date: NaiveDate,
        time_granularity: String,
    }

    let hc_opt = sqlx::query_as::<_, HcRow>(
        "SELECT start_date, end_date, time_granularity FROM hypercubes WHERE id = $1",
    )
    .bind(&row.hypercube_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let hc = match hc_opt {
        Some(h) => h,
        None => {
            checks.push(ValidationCheck {
                check: "Hypercube".to_string(),
                passed: false,
                message: "Hypercube introuvable ou supprimé".to_string(),
            });
            let report = ValidationReport { is_valid: false, checks };
            persist_validation(&state, &id, &report).await?;
            return Ok(Json(report));
        }
    };

    let hc_start_ym = hc.start_date.format("%Y-%m").to_string();
    let hc_end_ym   = hc.end_date.format("%Y-%m").to_string();

    // Matrices du hypercube
    let hc_matrix_ids: Vec<String> =
        sqlx::query_scalar("SELECT matrix_id FROM hypercube_matrices WHERE hypercube_id = $1")
            .bind(&row.hypercube_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    checks.push(ValidationCheck {
        check: "Hypercube".to_string(),
        passed: true,
        message: format!(
            "Hypercube valide ({} matrice(s), plage {}/{})",
            hc_matrix_ids.len(),
            hc_start_ym,
            hc_end_ym
        ),
    });

    // ── Check 1 : Portfolio + paires ─────────────────────────────────────────

    let pair_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM portfolio_pairs WHERE portfolio_id = $1")
            .bind(&row.portfolio_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    checks.push(ValidationCheck {
        check: "Portfolio".to_string(),
        passed: pair_count > 0,
        message: if pair_count > 0 {
            format!("Portfolio valide avec {} paire(s)", pair_count)
        } else {
            "Portfolio sans paires définies".to_string()
        },
    });

    // ── Check 2 : Couverture temporelle — vecteurs ───────────────────────────

    #[derive(sqlx::FromRow)]
    struct DateRow {
        name: String,
        date_from: Option<String>,
        date_to: Option<String>,
    }

    let vectors = sqlx::query_as::<_, DateRow>(
        r#"SELECT ov.name,
                  ov.rows_json::json -> 0               ->> 'date' AS date_from,
                  ov.rows_json::json ->
                      (json_array_length(ov.rows_json::json) - 1) ->> 'date' AS date_to
           FROM outstanding_vectors ov
           JOIN portfolio_vectors pv ON pv.vector_id = ov.id
           WHERE pv.portfolio_id = $1"#,
    )
    .bind(&row.portfolio_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut vec_issues: Vec<String> = Vec::new();
    for v in &vectors {
        let from = v.date_from.as_deref().unwrap_or("");
        let to   = v.date_to.as_deref().unwrap_or("");
        if from.is_empty() || to.is_empty() || from > hc_start_ym.as_str() || to < hc_end_ym.as_str() {
            vec_issues.push(format!(
                "{}: {}/{}  (requis: {}/{})",
                v.name, from, to, hc_start_ym, hc_end_ym
            ));
        }
    }
    checks.push(ValidationCheck {
        check: "Couverture temporelle (vecteurs)".to_string(),
        passed: vec_issues.is_empty(),
        message: if vec_issues.is_empty() {
            format!("Les {} vecteur(s) couvrent la plage du hypercube", vectors.len())
        } else {
            format!("Couverture insuffisante : {}", vec_issues.join("; "))
        },
    });

    // ── Check 3 : Couverture temporelle — schedules ──────────────────────────

    let schedules = sqlx::query_as::<_, DateRow>(
        r#"SELECT ams.name,
                  ams.rows_json::json -> 0               ->> 'date' AS date_from,
                  ams.rows_json::json ->
                      (json_array_length(ams.rows_json::json) - 1) ->> 'date' AS date_to
           FROM amort_schedules ams
           JOIN portfolio_schedules ps ON ps.schedule_id = ams.id
           WHERE ps.portfolio_id = $1"#,
    )
    .bind(&row.portfolio_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut sched_issues: Vec<String> = Vec::new();
    for s in &schedules {
        let from = s.date_from.as_deref().unwrap_or("");
        let to   = s.date_to.as_deref().unwrap_or("");
        if from.is_empty() || to.is_empty() || from > hc_start_ym.as_str() || to < hc_end_ym.as_str() {
            sched_issues.push(format!(
                "{}: {}/{}  (requis: {}/{})",
                s.name, from, to, hc_start_ym, hc_end_ym
            ));
        }
    }
    checks.push(ValidationCheck {
        check: "Couverture temporelle (schedules)".to_string(),
        passed: sched_issues.is_empty(),
        message: if sched_issues.is_empty() {
            format!("Les {} schedule(s) couvrent la plage du hypercube", schedules.len())
        } else {
            format!("Couverture insuffisante : {}", sched_issues.join("; "))
        },
    });

    // ── Check 4 : Validité des combinaisons ──────────────────────────────────

    #[derive(sqlx::FromRow)]
    struct AssignRow {
        id: String,
        pair_id: String,
        combination_matrix_ids: String,
        label: Option<String>,
    }

    let assignments = sqlx::query_as::<_, AssignRow>(
        "SELECT id, pair_id, combination_matrix_ids, label
         FROM study_unit_assignments WHERE study_unit_id = $1",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut combo_issues: Vec<String> = Vec::new();
    for a in &assignments {
        let combo_ids: Vec<String> =
            serde_json::from_str(&a.combination_matrix_ids).unwrap_or_default();
        for mid in &combo_ids {
            if !hc_matrix_ids.contains(mid) {
                combo_issues.push(format!(
                    "Assignment '{}': matrice {} absente du hypercube",
                    a.label.as_deref().unwrap_or(&a.id),
                    mid
                ));
            }
        }
    }
    checks.push(ValidationCheck {
        check: "Validité des combinaisons".to_string(),
        passed: combo_issues.is_empty(),
        message: if combo_issues.is_empty() {
            if assignments.is_empty() {
                "Aucun assignment défini".to_string()
            } else {
                format!("{} assignment(s) avec des combinaisons valides", assignments.len())
            }
        } else {
            combo_issues.join("; ")
        },
    });

    // ── Check 5 : Paires sans assignment ─────────────────────────────────────

    let unassigned: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM portfolio_pairs pp
         WHERE pp.portfolio_id = $1
         AND pp.id NOT IN (
             SELECT pair_id FROM study_unit_assignments WHERE study_unit_id = $2
         )",
    )
    .bind(&row.portfolio_id)
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    checks.push(ValidationCheck {
        check: "Paires assignées".to_string(),
        passed: unassigned == 0,
        message: if unassigned == 0 {
            if pair_count == 0 {
                "Aucune paire dans ce portfolio".to_string()
            } else {
                format!("Toutes les {} paire(s) ont au moins un assignment", pair_count)
            }
        } else {
            format!("{}/{} paire(s) sans assignment", unassigned, pair_count)
        },
    });

    // ── Persister et retourner ────────────────────────────────────────────────

    let is_valid = checks.iter().all(|c| c.passed);
    let report = ValidationReport { is_valid, checks };
    persist_validation(&state, &id, &report).await?;
    Ok(Json(report))
}

async fn persist_validation(
    state: &AppState,
    id: &str,
    report: &ValidationReport,
) -> Result<(), (StatusCode, String)> {
    let log = serde_json::to_string(&report.checks).unwrap_or_default();
    sqlx::query(
        "UPDATE study_units SET is_valid = $1, validation_log = $2 WHERE id = $3",
    )
    .bind(report.is_valid)
    .bind(&log)
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(())
}

// ── Handlers — Assignments ────────────────────────────────────────────────────

pub async fn create_assignment(
    State(state): State<AppState>,
    Path(study_unit_id): Path<String>,
    Json(body): Json<CreateAssignmentRequest>,
) -> Result<Json<AssignmentInfo>, (StatusCode, String)> {
    // Vérifier la study unit
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM study_units WHERE id = $1")
        .bind(&study_unit_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if count == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Study unit introuvable"));
    }

    if body.combination_matrix_ids.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "La combinaison ne peut pas être vide"));
    }

    // Vérifier que la paire appartient au portfolio de la study unit
    let ok: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM study_units su
         JOIN portfolio_pairs pp ON pp.portfolio_id = su.portfolio_id
         WHERE su.id = $1 AND pp.id = $2",
    )
    .bind(&study_unit_id)
    .bind(&body.pair_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if ok == 0 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "La paire n'appartient pas au portfolio de cette study unit",
        ));
    }

    let id = Uuid::new_v4().to_string();
    let combo_json = serde_json::to_string(&body.combination_matrix_ids)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let ftp_json = body
        .initial_ftp_profile_json
        .map(|v| serde_json::to_string(&v).unwrap_or_default());
    let is_existing = body.is_existing_stock.unwrap_or(false);

    sqlx::query(
        "INSERT INTO study_unit_assignments
             (id, study_unit_id, pair_id, combination_matrix_ids, label,
              is_existing_stock, initial_ftp_profile_json)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&id)
    .bind(&study_unit_id)
    .bind(&body.pair_id)
    .bind(&combo_json)
    .bind(&body.label)
    .bind(is_existing)
    .bind(&ftp_json)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    reset_validity(&state, &study_unit_id).await?;

    let assignments = load_assignments(&state, &study_unit_id).await?;
    assignments
        .into_iter()
        .find(|a| a.id == id)
        .map(Json)
        .ok_or_else(|| err(StatusCode::INTERNAL_SERVER_ERROR, "Assignment introuvable après création"))
}

pub async fn update_assignment(
    State(state): State<AppState>,
    Path((study_unit_id, assignment_id)): Path<(String, String)>,
    Json(body): Json<UpdateAssignmentRequest>,
) -> Result<Json<AssignmentInfo>, (StatusCode, String)> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM study_unit_assignments WHERE id = $1 AND study_unit_id = $2",
    )
    .bind(&assignment_id)
    .bind(&study_unit_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if count == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Assignment introuvable"));
    }

    if let Some(combo) = &body.combination_matrix_ids {
        if combo.is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, "La combinaison ne peut pas être vide"));
        }
        let json = serde_json::to_string(combo)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        sqlx::query(
            "UPDATE study_unit_assignments SET combination_matrix_ids = $1 WHERE id = $2",
        )
        .bind(&json)
        .bind(&assignment_id)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.label {
        sqlx::query("UPDATE study_unit_assignments SET label = $1 WHERE id = $2")
            .bind(v)
            .bind(&assignment_id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = body.is_existing_stock {
        sqlx::query("UPDATE study_unit_assignments SET is_existing_stock = $1 WHERE id = $2")
            .bind(v)
            .bind(&assignment_id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.initial_ftp_profile_json {
        let json = serde_json::to_string(v)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        sqlx::query(
            "UPDATE study_unit_assignments SET initial_ftp_profile_json = $1 WHERE id = $2",
        )
        .bind(&json)
        .bind(&assignment_id)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    reset_validity(&state, &study_unit_id).await?;

    let assignments = load_assignments(&state, &study_unit_id).await?;
    assignments
        .into_iter()
        .find(|a| a.id == assignment_id)
        .map(Json)
        .ok_or_else(|| err(StatusCode::INTERNAL_SERVER_ERROR, "Assignment introuvable après mise à jour"))
}

pub async fn delete_assignment(
    State(state): State<AppState>,
    Path((study_unit_id, assignment_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query(
        "DELETE FROM study_unit_assignments WHERE id = $1 AND study_unit_id = $2",
    )
    .bind(&assignment_id)
    .bind(&study_unit_id)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if res.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Assignment introuvable"));
    }

    reset_validity(&state, &study_unit_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

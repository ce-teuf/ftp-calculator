use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::AppState;

// ── Error helper ──────────────────────────────────────────────────────────────

fn err(status: StatusCode, msg: impl ToString) -> (StatusCode, String) {
    (status, msg.to_string())
}

// ── Modèles publics ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct StudySummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub unit_count: i64,
    pub valid_unit_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StudyUnitRef {
    pub study_unit_id: String,
    pub name: String,
    pub hypercube_name: String,
    pub portfolio_name: String,
    pub is_valid: bool,
    pub assignment_count: i64,
    pub label: Option<String>,
    pub position: i32,
}

#[derive(Debug, Serialize)]
pub struct StudyDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub units: Vec<StudyUnitRef>,
    pub created_at: DateTime<Utc>,
}

// ── Requêtes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateStudyRequest {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStudyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddUnitRequest {
    pub study_unit_id: String,
    pub label: Option<String>,
}

// ── Structs sqlx ─────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct StudyRow {
    id: String,
    name: String,
    description: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct UnitRefRow {
    study_unit_id: String,
    name: String,
    hypercube_name: String,
    portfolio_name: String,
    is_valid: bool,
    assignment_count: i64,
    label: Option<String>,
    position: i32,
}

// ── Helper interne ────────────────────────────────────────────────────────────

async fn load_units(
    state: &AppState,
    study_id: &str,
) -> Result<Vec<StudyUnitRef>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, UnitRefRow>(
        r#"
        SELECT si.study_unit_id, su.name,
               h.name AS hypercube_name,
               p.name AS portfolio_name,
               su.is_valid,
               COUNT(sua.id) AS assignment_count,
               si.label, si.position
        FROM study_items si
        JOIN study_units su ON su.id = si.study_unit_id
        JOIN hypercubes  h  ON h.id  = su.hypercube_id
        JOIN portfolios  p  ON p.id  = su.portfolio_id
        LEFT JOIN study_unit_assignments sua ON sua.study_unit_id = su.id
        WHERE si.study_id = $1
        GROUP BY si.study_unit_id, su.name, h.name, p.name, su.is_valid, si.label, si.position
        ORDER BY si.position, si.study_unit_id
        "#,
    )
    .bind(study_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(rows
        .into_iter()
        .map(|r| StudyUnitRef {
            study_unit_id: r.study_unit_id,
            name: r.name,
            hypercube_name: r.hypercube_name,
            portfolio_name: r.portfolio_name,
            is_valid: r.is_valid,
            assignment_count: r.assignment_count,
            label: r.label,
            position: r.position,
        })
        .collect())
}

async fn fetch_detail(
    state: &AppState,
    id: &str,
) -> Result<Json<StudyDetail>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, StudyRow>(
        "SELECT id, name, description, status, created_at FROM studies WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Étude introuvable"))?;

    let units = load_units(state, id).await?;

    Ok(Json(StudyDetail {
        id: row.id,
        name: row.name,
        description: row.description,
        status: row.status,
        units,
        created_at: row.created_at,
    }))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_studies(
    State(state): State<AppState>,
) -> Result<Json<Vec<StudySummary>>, (StatusCode, String)> {
    #[derive(sqlx::FromRow)]
    struct SummaryRow {
        id: String,
        name: String,
        description: Option<String>,
        status: String,
        unit_count: i64,
        valid_unit_count: i64,
        created_at: DateTime<Utc>,
    }

    let rows = sqlx::query_as::<_, SummaryRow>(
        r#"
        SELECT s.id, s.name, s.description, s.status,
               COUNT(si.study_unit_id)                                   AS unit_count,
               COUNT(si.study_unit_id) FILTER (WHERE su.is_valid = true) AS valid_unit_count,
               s.created_at
        FROM studies s
        LEFT JOIN study_items si ON si.study_id = s.id
        LEFT JOIN study_units su ON su.id = si.study_unit_id
        GROUP BY s.id
        ORDER BY s.created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(
        rows.into_iter()
            .map(|r| StudySummary {
                id: r.id,
                name: r.name,
                description: r.description,
                status: r.status,
                unit_count: r.unit_count,
                valid_unit_count: r.valid_unit_count,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

pub async fn create_study(
    State(state): State<AppState>,
    Json(body): Json<CreateStudyRequest>,
) -> Result<Json<StudyDetail>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Le nom est requis"));
    }
    let valid_statuses = ["draft", "ready", "archived"];
    let status = body.status.as_deref().unwrap_or("draft");
    if !valid_statuses.contains(&status) {
        return Err(err(StatusCode::BAD_REQUEST, "Statut invalide (draft | ready | archived)"));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO studies (id, name, description, status) VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(body.name.trim())
    .bind(&body.description)
    .bind(status)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    fetch_detail(&state, &id).await
}

pub async fn get_study(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StudyDetail>, (StatusCode, String)> {
    fetch_detail(&state, &id).await
}

pub async fn update_study(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateStudyRequest>,
) -> Result<Json<StudyDetail>, (StatusCode, String)> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM studies WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if count == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Étude introuvable"));
    }

    if let Some(v) = &body.name {
        if v.trim().is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, "Le nom ne peut pas être vide"));
        }
        sqlx::query("UPDATE studies SET name = $1 WHERE id = $2")
            .bind(v.trim())
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.description {
        sqlx::query("UPDATE studies SET description = $1 WHERE id = $2")
            .bind(v)
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.status {
        let valid_statuses = ["draft", "ready", "archived"];
        if !valid_statuses.contains(&v.as_str()) {
            return Err(err(StatusCode::BAD_REQUEST, "Statut invalide (draft | ready | archived)"));
        }
        sqlx::query("UPDATE studies SET status = $1 WHERE id = $2")
            .bind(v)
            .bind(&id)
            .execute(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    fetch_detail(&state, &id).await
}

pub async fn delete_study(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM studies WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if res.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Étude introuvable"));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_unit(
    State(state): State<AppState>,
    Path(study_id): Path<String>,
    Json(body): Json<AddUnitRequest>,
) -> Result<Json<StudyDetail>, (StatusCode, String)> {
    // Vérifier l'étude
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM studies WHERE id = $1")
        .bind(&study_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if count == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Étude introuvable"));
    }

    // Vérifier la study unit
    let su_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM study_units WHERE id = $1")
        .bind(&body.study_unit_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if su_count == 0 {
        return Err(err(StatusCode::BAD_REQUEST, "Study unit introuvable"));
    }

    // Position = max existant + 1
    let max_pos: Option<i32> =
        sqlx::query_scalar("SELECT MAX(position) FROM study_items WHERE study_id = $1")
            .bind(&study_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let position = max_pos.unwrap_or(-1) + 1;

    sqlx::query(
        "INSERT INTO study_items (study_id, study_unit_id, label, position)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (study_id, study_unit_id) DO NOTHING",
    )
    .bind(&study_id)
    .bind(&body.study_unit_id)
    .bind(&body.label)
    .bind(position)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    fetch_detail(&state, &study_id).await
}

pub async fn remove_unit(
    State(state): State<AppState>,
    Path((study_id, unit_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query(
        "DELETE FROM study_items WHERE study_id = $1 AND study_unit_id = $2",
    )
    .bind(&study_id)
    .bind(&unit_id)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if res.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Study unit non trouvée dans cette étude"));
    }
    Ok(StatusCode::NO_CONTENT)
}

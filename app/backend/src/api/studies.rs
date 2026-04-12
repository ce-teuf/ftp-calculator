use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
struct StudyRow {
    id: String,
    name: String,
    description: Option<String>,
    notes: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct StudyLinkerRow {
    linker_id: String,
    linker_name: Option<String>,
    portfolio_name: Option<String>,
    cube_name: Option<String>,
    start_date: Option<String>,
    label: Option<String>,
    position: i32,
}

async fn fetch_study(pool: &sqlx::PgPool, id: &str) -> Result<Value, StatusCode> {
    let s = sqlx::query_as::<_, StudyRow>(
        "SELECT id, name, description, notes, created_at::TEXT FROM studies WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let linker_rows = sqlx::query_as::<_, StudyLinkerRow>(
        r#"SELECT sl.linker_id,
                  lk.name  AS linker_name,
                  p.name   AS portfolio_name,
                  c.name   AS cube_name,
                  lk.start_date::TEXT,
                  sl.label,
                  sl.position
           FROM study_linkers sl
           JOIN linkers      lk ON lk.id = sl.linker_id
           LEFT JOIN portfolios_v3 p ON p.id = lk.portfolio_id
           LEFT JOIN curve_cubes   c ON c.id = lk.cube_id
           WHERE sl.study_id = $1
           ORDER BY sl.position"#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let linkers: Vec<Value> = linker_rows
        .into_iter()
        .map(|r| json!({
            "linker_id": r.linker_id,
            "linker_name": r.linker_name,
            "portfolio_name": r.portfolio_name,
            "cube_name": r.cube_name,
            "start_date": r.start_date,
            "label": r.label,
            "position": r.position,
        }))
        .collect();

    Ok(json!({
        "id": s.id,
        "name": s.name,
        "description": s.description,
        "notes": s.notes,
        "created_at": s.created_at,
        "linkers": linkers,
    }))
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateStudyRequest {
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateStudyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct AddLinkerRequest {
    pub linker_id: String,
    pub label: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_studies(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let rows = sqlx::query_as::<_, StudyRow>(
        "SELECT id, name, description, notes, created_at::TEXT FROM studies ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut result = vec![];
    for s in rows {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM study_linkers WHERE study_id = $1",
        )
        .bind(&s.id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

        result.push(json!({
            "id": s.id,
            "name": s.name,
            "description": s.description,
            "notes": s.notes,
            "created_at": s.created_at,
            "linker_count": count,
        }));
    }

    Ok(Json(json!(result)))
}

pub async fn get_study(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(fetch_study(&state.pool, &id).await?))
}

pub async fn create_study(
    State(state): State<AppState>,
    Json(body): Json<CreateStudyRequest>,
) -> Result<Json<Value>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO studies (id, name, description, notes) VALUES ($1,$2,$3,$4)",
    )
    .bind(&id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.notes)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(fetch_study(&state.pool, &id).await?))
}

pub async fn update_study(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateStudyRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Some(name) = &body.name {
        sqlx::query("UPDATE studies SET name=$1 WHERE id=$2")
            .bind(name).bind(&id)
            .execute(&state.pool).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(desc) = &body.description {
        sqlx::query("UPDATE studies SET description=$1 WHERE id=$2")
            .bind(desc).bind(&id)
            .execute(&state.pool).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    // notes can be cleared (always update if key present)
    sqlx::query("UPDATE studies SET notes=$1 WHERE id=$2")
        .bind(&body.notes).bind(&id)
        .execute(&state.pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(fetch_study(&state.pool, &id).await?))
}

pub async fn delete_study(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM studies WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_linker_to_study(
    State(state): State<AppState>,
    Path(study_id): Path<String>,
    Json(body): Json<AddLinkerRequest>,
) -> Result<Json<Value>, StatusCode> {
    let pos: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM study_linkers WHERE study_id = $1",
    )
    .bind(&study_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    sqlx::query(
        "INSERT INTO study_linkers (study_id, linker_id, label, position)
         VALUES ($1,$2,$3,$4)
         ON CONFLICT (study_id, linker_id) DO UPDATE SET label=$3",
    )
    .bind(&study_id)
    .bind(&body.linker_id)
    .bind(&body.label)
    .bind(pos as i32)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(fetch_study(&state.pool, &study_id).await?))
}

pub async fn remove_linker_from_study(
    State(state): State<AppState>,
    Path((study_id, linker_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    sqlx::query(
        "DELETE FROM study_linkers WHERE study_id=$1 AND linker_id=$2",
    )
    .bind(&study_id)
    .bind(&linker_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(fetch_study(&state.pool, &study_id).await?))
}

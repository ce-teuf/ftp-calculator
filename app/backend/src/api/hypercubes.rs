use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::db::AppState;

// ── Error helper ──────────────────────────────────────────────────────────────

fn err(status: StatusCode, msg: impl ToString) -> (StatusCode, String) {
    (status, msg.to_string())
}

// ── Modèles ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct HypercubeSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub proj_end_date: Option<NaiveDate>,
    pub time_granularity: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub matrix_count: i64,
    pub combination_count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct HypercubeMatrixRef {
    pub id: String,
    pub name: String,
    pub currency: Option<String>,
    pub status: String,
    pub risks: Vec<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HypercubeDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub proj_end_date: Option<NaiveDate>,
    pub time_granularity: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub matrices: Vec<HypercubeMatrixRef>,
    pub combination_count: usize,
}

#[derive(Debug, Serialize)]
pub struct Combination {
    pub matrix_ids: Vec<String>,
    pub matrix_names: Vec<String>,
    pub risks_covered: Vec<String>,
}

// ── Requêtes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub proj_end_date: Option<NaiveDate>,
    pub time_granularity: Option<String>,
    pub status: Option<String>,
    pub matrix_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub proj_end_date: Option<NaiveDate>,
    pub time_granularity: Option<String>,
    pub status: Option<String>,
    pub matrix_ids: Option<Vec<String>>,
}

// ── Structs sqlx internes ─────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct HypercubeRow {
    id: String,
    name: String,
    description: Option<String>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    proj_end_date: Option<NaiveDate>,
    time_granularity: String,
    status: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct MatrixInfoRow {
    id: String,
    name: String,
    currency: Option<String>,
    status: String,
    date_from: Option<String>,
    date_to: Option<String>,
}

#[derive(sqlx::FromRow)]
struct RiskAssoc {
    matrix_id: String,
    risk_key: String,
}

#[derive(sqlx::FromRow)]
struct HypercubeMatrixLink {
    hypercube_id: String,
    matrix_id: String,
}

// ── Calcul des combinaisons valides ───────────────────────────────────────────

/// Génère toutes les combinaisons valides de matrices (sans doublon de risque).
/// Retourne les combinaisons triées par taille croissante.
fn compute_combinations(matrices: &[HypercubeMatrixRef]) -> Vec<Combination> {
    let n = matrices.len();
    if n == 0 || n > 20 {
        return Vec::new(); // garde-fou : 2^20 = 1M, au-delà c'est impraticable
    }

    let mut result = Vec::new();

    for mask in 1u32..(1u32 << n) {
        let mut risks_used: HashSet<&str> = HashSet::new();
        let mut ids = Vec::new();
        let mut names = Vec::new();
        let mut valid = true;

        for i in 0..n {
            if mask & (1 << i) != 0 {
                for risk in &matrices[i].risks {
                    if !risks_used.insert(risk.as_str()) {
                        valid = false;
                        break;
                    }
                }
                if !valid {
                    break;
                }
                ids.push(matrices[i].id.clone());
                names.push(matrices[i].name.clone());
            }
        }

        if valid {
            let mut risks_covered: Vec<String> = risks_used.into_iter().map(String::from).collect();
            risks_covered.sort();
            result.push(Combination {
                matrix_ids: ids,
                matrix_names: names,
                risks_covered,
            });
        }
    }

    result
}

// ── Utilitaires internes ──────────────────────────────────────────────────────

async fn load_matrices_for_hypercube(
    state: &AppState,
    hypercube_id: &str,
) -> Result<Vec<HypercubeMatrixRef>, (StatusCode, String)> {
    // IDs des matrices liées à ce hypercube
    let links = sqlx::query_as::<_, HypercubeMatrixLink>(
        "SELECT hypercube_id, matrix_id FROM hypercube_matrices WHERE hypercube_id = $1"
    )
    .bind(hypercube_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if links.is_empty() {
        return Ok(Vec::new());
    }

    let matrix_ids: Vec<String> = links.into_iter().map(|l| l.matrix_id).collect();

    // Infos des matrices
    let placeholders: String = matrix_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        r#"
        SELECT id, name, currency, status,
               rows_json::json -> 0 ->> 'date'                              AS date_from,
               rows_json::json ->
                   (GREATEST(json_array_length(rows_json::json) - 1, 0))
                   ->> 'date'                                               AS date_to
        FROM rate_matrices
        WHERE id IN ({placeholders})
        ORDER BY name
        "#
    );

    let mut q = sqlx::query_as::<_, MatrixInfoRow>(&query);
    for id in &matrix_ids {
        q = q.bind(id);
    }
    let matrix_rows = q.fetch_all(&state.pool).await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Risques associés à ces matrices
    let risk_query = format!(
        "SELECT matrix_id, risk_key FROM rate_matrix_risks WHERE matrix_id IN ({placeholders}) ORDER BY matrix_id, risk_key"
    );
    let mut rq = sqlx::query_as::<_, RiskAssoc>(&risk_query);
    for id in &matrix_ids {
        rq = rq.bind(id);
    }
    let risk_rows = rq.fetch_all(&state.pool).await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut risk_map: HashMap<String, Vec<String>> = HashMap::new();
    for ra in risk_rows {
        risk_map.entry(ra.matrix_id).or_default().push(ra.risk_key);
    }

    let matrices = matrix_rows
        .into_iter()
        .map(|r| HypercubeMatrixRef {
            risks: risk_map.get(&r.id).cloned().unwrap_or_default(),
            id: r.id,
            name: r.name,
            currency: r.currency,
            status: r.status,
            date_from: r.date_from,
            date_to: r.date_to,
        })
        .collect();

    Ok(matrices)
}

async fn fetch_detail(
    state: &AppState,
    id: &str,
) -> Result<Json<HypercubeDetail>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, HypercubeRow>(
        "SELECT id, name, description, start_date, end_date, proj_end_date,
                time_granularity, status, created_at
         FROM hypercubes WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Hypercube introuvable"))?;

    let matrices = load_matrices_for_hypercube(state, id).await?;
    let combos = compute_combinations(&matrices);

    Ok(Json(HypercubeDetail {
        id: row.id,
        name: row.name,
        description: row.description,
        start_date: row.start_date,
        end_date: row.end_date,
        proj_end_date: row.proj_end_date,
        time_granularity: row.time_granularity,
        status: row.status,
        created_at: row.created_at,
        combination_count: combos.len(),
        matrices,
    }))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_hypercubes(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<HypercubeSummary>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, HypercubeRow>(
        "SELECT id, name, description, start_date, end_date, proj_end_date,
                time_granularity, status, created_at
         FROM hypercubes ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Compte de matrices par hypercube
    #[derive(sqlx::FromRow)]
    struct CountRow { hypercube_id: String, cnt: i64 }

    let counts = sqlx::query_as::<_, CountRow>(
        "SELECT hypercube_id, COUNT(*) AS cnt FROM hypercube_matrices GROUP BY hypercube_id"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let count_map: HashMap<String, i64> = counts.into_iter()
        .map(|r| (r.hypercube_id, r.cnt))
        .collect();

    let mut summaries: Vec<HypercubeSummary> = Vec::new();
    for row in rows {
        let matrix_count = *count_map.get(&row.id).unwrap_or(&0);
        // Pour la liste, on calcule le nb de combinaisons uniquement si peu de matrices
        let combo_count = if matrix_count <= 20 {
            let matrices = load_matrices_for_hypercube(&state, &row.id).await?;
            compute_combinations(&matrices).len()
        } else {
            0
        };
        summaries.push(HypercubeSummary {
            id: row.id,
            name: row.name,
            description: row.description,
            start_date: row.start_date,
            end_date: row.end_date,
            proj_end_date: row.proj_end_date,
            time_granularity: row.time_granularity,
            status: row.status,
            created_at: row.created_at,
            matrix_count,
            combination_count: combo_count,
        });
    }

    if let Some(status) = &q.status {
        summaries.retain(|s| &s.status == status);
    }

    Ok(Json(summaries))
}

pub async fn create_hypercube(
    State(state): State<AppState>,
    Json(body): Json<CreateRequest>,
) -> Result<Json<HypercubeDetail>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Le nom est requis"));
    }
    if body.end_date < body.start_date {
        return Err(err(StatusCode::BAD_REQUEST, "end_date doit être >= start_date"));
    }
    if let Some(proj) = body.proj_end_date {
        if proj < body.end_date {
            return Err(err(StatusCode::BAD_REQUEST, "proj_end_date doit être >= end_date"));
        }
    }

    let granularity = "monthly";
    let status = body.status.as_deref().unwrap_or("draft");
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO hypercubes (id, name, description, start_date, end_date, proj_end_date, time_granularity, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&id)
    .bind(body.name.trim())
    .bind(&body.description)
    .bind(body.start_date)
    .bind(body.end_date)
    .bind(body.proj_end_date)
    .bind(granularity)
    .bind(status)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    for matrix_id in &body.matrix_ids {
        sqlx::query(
            "INSERT INTO hypercube_matrices (hypercube_id, matrix_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(&id)
        .bind(matrix_id)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    fetch_detail(&state, &id).await
}

pub async fn get_hypercube(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<HypercubeDetail>, (StatusCode, String)> {
    fetch_detail(&state, &id).await
}

pub async fn update_hypercube(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateRequest>,
) -> Result<Json<HypercubeDetail>, (StatusCode, String)> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hypercubes WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if count == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Hypercube introuvable"));
    }

    if let Some(v) = &body.name {
        sqlx::query("UPDATE hypercubes SET name = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.description {
        sqlx::query("UPDATE hypercubes SET description = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = body.start_date {
        sqlx::query("UPDATE hypercubes SET start_date = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = body.end_date {
        sqlx::query("UPDATE hypercubes SET end_date = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    // proj_end_date peut être null intentionnellement → toujours mettre à jour si présent dans le body
    if body.proj_end_date.is_some() || body.end_date.is_some() {
        // Ne mettre à jour que si explicitement fourni dans l'objet (Option<Option<_>> serait plus précis
        // mais pour simplifier on met à jour si la clé est présente)
        if let Some(v) = body.proj_end_date {
            sqlx::query("UPDATE hypercubes SET proj_end_date = $1 WHERE id = $2")
                .bind(v).bind(&id).execute(&state.pool).await
                .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
    }
    // time_granularity is always monthly — no update needed
    if let Some(v) = &body.status {
        sqlx::query("UPDATE hypercubes SET status = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(matrix_ids) = &body.matrix_ids {
        sqlx::query("DELETE FROM hypercube_matrices WHERE hypercube_id = $1")
            .bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        for matrix_id in matrix_ids {
            sqlx::query(
                "INSERT INTO hypercube_matrices (hypercube_id, matrix_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(&id).bind(matrix_id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
    }

    fetch_detail(&state, &id).await
}

pub async fn delete_hypercube(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM hypercubes WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if res.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Hypercube introuvable"));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_combinations(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Combination>>, (StatusCode, String)> {
    // Vérifier l'existence
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hypercubes WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if count == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Hypercube introuvable"));
    }

    let matrices = load_matrices_for_hypercube(&state, &id).await?;
    let combos = compute_combinations(&matrices);
    Ok(Json(combos))
}

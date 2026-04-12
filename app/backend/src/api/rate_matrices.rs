use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;

use crate::db::AppState;

// ── Error helper ──────────────────────────────────────────────────────────────

fn err(status: StatusCode, msg: impl ToString) -> (StatusCode, String) {
    (status, msg.to_string())
}

// ── Types de risque ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RiskType {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
}

// ── Modèles de réponse ────────────────────────────────────────────────────────

/// Ligne résumée pour la vue liste (sans rows_json)
#[derive(Debug, Serialize)]
pub struct RateMatrixSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub status: String,
    pub interp_method: String,
    pub tenors: Vec<String>,
    pub row_count: i64,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub risks: Vec<String>,
}

/// Vue complète incluant les données brutes
#[derive(Debug, Serialize)]
pub struct RateMatrixDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub status: String,
    pub interp_method: String,
    pub tenors: Vec<String>,
    pub rows: Vec<MatrixRow>,
    pub created_at: DateTime<Utc>,
    pub risks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatrixRow {
    pub date: String,
    pub period_type: String,
    pub values: Vec<f64>,
}

// ── Requêtes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub currency: Option<String>,
    pub risk_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub status: Option<String>,
    pub interp_method: Option<String>,
    pub risks: Option<Vec<String>>,
}

// ── Structs sqlx internes ─────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct MatrixListRow {
    id: String,
    name: String,
    description: Option<String>,
    currency: Option<String>,
    status: String,
    interp_method: String,
    tenors_json: String,
    row_count: i64,
    date_from: Option<String>,
    date_to: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct MatrixDetailRow {
    id: String,
    name: String,
    description: Option<String>,
    currency: Option<String>,
    status: String,
    interp_method: String,
    tenors_json: String,
    rows_json: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct RiskAssoc {
    matrix_id: String,
    risk_key: String,
}

// ── Parsing du fichier Excel / ODS ────────────────────────────────────────────

fn cell_to_string(cell: &Data) -> Option<String> {
    match cell {
        Data::String(s) => {
            let s = s.trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        }
        Data::Float(f)  => Some(f.to_string()),
        Data::Int(i)    => Some(i.to_string()),
        Data::Bool(b)   => Some(b.to_string()),
        Data::DateTimeIso(s) => Some(s.clone()),
        _ => None,
    }
}

/// Convertit une cellule en "YYYY-MM"
fn cell_to_ym(cell: &Data) -> Option<String> {
    match cell {
        Data::String(s) => {
            let s = s.trim();
            // Accepte "2024-01", "2024-01-01", "Jan-24", etc.
            // On ne gère que le format YYYY-MM[...] pour l'instant
            if s.len() >= 7 && s.chars().nth(4) == Some('-') {
                Some(s[..7].to_string())
            } else {
                None
            }
        }
        Data::DateTimeIso(s) => {
            if s.len() >= 7 { Some(s[..7].to_string()) } else { None }
        }
        Data::DateTime(dt)     => excel_serial_to_ym(dt.as_f64()),
        Data::Float(f)         => excel_serial_to_ym(*f),
        _ => None,
    }
}

fn excel_serial_to_ym(serial: f64) -> Option<String> {
    // L'epoch Excel est le 30 déc. 1899.
    // Excel croit à tort que 1900 est une année bissextile → on compense à partir du serial 61.
    let days = if serial >= 61.0 { serial as i64 - 2 } else { serial as i64 - 1 };
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let date  = epoch.checked_add_signed(Duration::days(days))?;
    Some(date.format("%Y-%m").to_string())
}

fn cell_to_f64(cell: &Data) -> Option<f64> {
    match cell {
        Data::Float(f) => Some(*f),
        Data::Int(i)   => Some(*i as f64),
        Data::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn parse_rate_matrix_file(bytes: Vec<u8>) -> Result<(Vec<String>, Vec<MatrixRow>), String> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;

    let sheet = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| "Le fichier ne contient aucune feuille".to_string())?
        .map_err(|e| format!("Erreur de lecture de la feuille : {e}"))?;

    let mut rows_iter = sheet.rows();

    // ── Ligne d'entête ────────────────────────────────────────────────────────
    let header = rows_iter.next()
        .ok_or_else(|| "Fichier vide".to_string())?;

    if header.len() < 3 {
        return Err("Au moins 3 colonnes requises : date_month, period_type, puis les tenors".into());
    }

    let h0 = cell_to_string(&header[0]).unwrap_or_default().to_lowercase();
    let h1 = cell_to_string(&header[1]).unwrap_or_default().to_lowercase();

    if h0 != "date_month" {
        return Err(format!("Colonne A doit être 'date_month', trouvé '{h0}'"));
    }
    if h1 != "period_type" {
        return Err(format!("Colonne B doit être 'period_type', trouvé '{h1}'"));
    }

    let tenors: Vec<String> = header[2..]
        .iter()
        .filter_map(|c| cell_to_string(c))
        .collect();

    if tenors.is_empty() {
        return Err("Aucun tenor trouvé (colonnes à partir de C)".into());
    }
    let n_tenors = tenors.len();

    // ── Lignes de données ─────────────────────────────────────────────────────
    let mut parsed: Vec<MatrixRow> = Vec::new();
    let mut seen_projected = false;
    let mut base_type: Option<String> = None; // "observed" ou "contrafactual"

    for (i, row) in rows_iter.enumerate() {
        // Ignorer les lignes vides
        if row.iter().all(|c| matches!(c, Data::Empty)) {
            continue;
        }

        let row_num = i + 2; // numéro de ligne dans le fichier (1-indexed, +1 pour l'entête)

        let date = cell_to_ym(&row[0])
            .ok_or_else(|| format!("Ligne {row_num} : date invalide dans la colonne A"))?;

        let pt_raw = row.get(1)
            .and_then(|c| cell_to_string(c))
            .ok_or_else(|| format!("Ligne {row_num} : period_type manquant dans la colonne B"))?;
        let pt = pt_raw.to_lowercase();

        match pt.as_str() {
            "observed" | "contrafactual" | "projected" => {}
            other => return Err(format!(
                "Ligne {row_num} : period_type invalide '{other}'. \
                 Valeurs acceptées : observed, contrafactual, projected"
            )),
        }

        if pt == "projected" {
            seen_projected = true;
        } else {
            if seen_projected {
                return Err(format!(
                    "Ligne {row_num} : '{pt}' après 'projected' — \
                     les lignes projected doivent être les dernières"
                ));
            }
            match &base_type {
                None => base_type = Some(pt.clone()),
                Some(base) if base != &pt => return Err(format!(
                    "Ligne {row_num} : mélange de '{base}' et '{pt}' \
                     dans les lignes non-projected"
                )),
                _ => {}
            }
        }

        let mut values: Vec<f64> = row
            .get(2..)
            .map(|cells| {
                cells.iter().take(n_tenors)
                    .map(|c| cell_to_f64(c).unwrap_or(0.0))
                    .collect()
            })
            .unwrap_or_default();

        // Compléter si la ligne a moins de colonnes que de tenors
        while values.len() < n_tenors { values.push(0.0); }

        parsed.push(MatrixRow { date, period_type: pt, values });
    }

    if parsed.is_empty() {
        return Err("Aucune ligne de données trouvée".into());
    }

    Ok((tenors, parsed))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_risk_types(
    State(state): State<AppState>,
) -> Result<Json<Vec<RiskType>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, RiskType>(
        "SELECT key, label, description FROM risk_types ORDER BY key"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(rows))
}

pub async fn list_rate_matrices(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<RateMatrixSummary>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, MatrixListRow>(
        r#"
        SELECT
            id, name, description, currency, status, interp_method, tenors_json,
            COALESCE(json_array_length(rows_json::json), 0)::BIGINT    AS row_count,
            rows_json::json -> 0 ->> 'date'                             AS date_from,
            rows_json::json ->
                (GREATEST(json_array_length(rows_json::json) - 1, 0))
                ->> 'date'                                              AS date_to,
            created_at
        FROM rate_matrices
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Récupérer toutes les associations de risques en une seule requête
    let risk_assocs = sqlx::query_as::<_, RiskAssoc>(
        "SELECT matrix_id, risk_key FROM rate_matrix_risks ORDER BY matrix_id, risk_key"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut risk_map: HashMap<String, Vec<String>> = HashMap::new();
    for ra in risk_assocs {
        risk_map.entry(ra.matrix_id).or_default().push(ra.risk_key);
    }

    let mut result: Vec<RateMatrixSummary> = rows
        .into_iter()
        .map(|r| {
            let tenors: Vec<String> = serde_json::from_str(&r.tenors_json).unwrap_or_default();
            let risks = risk_map.get(&r.id).cloned().unwrap_or_default();
            RateMatrixSummary {
                id: r.id.clone(),
                name: r.name,
                description: r.description,
                currency: r.currency,
                status: r.status,
                interp_method: r.interp_method,
                tenors,
                row_count: r.row_count,
                date_from: r.date_from,
                date_to: r.date_to,
                created_at: r.created_at,
                risks,
            }
        })
        .collect();

    // Filtres optionnels (sur le petit ensemble en mémoire)
    if let Some(status) = &q.status {
        result.retain(|r| &r.status == status);
    }
    if let Some(currency) = &q.currency {
        result.retain(|r| r.currency.as_deref() == Some(currency.as_str()));
    }
    if let Some(risk_key) = &q.risk_key {
        result.retain(|r| r.risks.contains(risk_key));
    }

    Ok(Json(result))
}

pub async fn create_rate_matrix(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<RateMatrixDetail>, (StatusCode, String)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut name          = String::new();
    let mut description:   Option<String> = None;
    let mut currency:      Option<String> = None;
    let mut status        = "draft".to_string();
    let mut interp_method = "linear".to_string();
    let mut risk_keys:     Vec<String> = Vec::new();

    while let Some(field) = multipart.next_field().await
        .map_err(|e| err(StatusCode::BAD_REQUEST, e))?
    {
        match field.name() {
            Some("file") => {
                file_bytes = Some(
                    field.bytes().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?.to_vec()
                );
            }
            Some("name") => {
                name = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?;
            }
            Some("description") => {
                let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?;
                if !v.is_empty() { description = Some(v); }
            }
            Some("currency") => {
                let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?;
                if !v.is_empty() { currency = Some(v); }
            }
            Some("status") => {
                let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?;
                if !v.is_empty() { status = v; }
            }
            Some("interp_method") => {
                let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?;
                if !v.is_empty() { interp_method = v; }
            }
            Some("risk_key") => {
                let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?;
                if !v.is_empty() { risk_keys.push(v); }
            }
            _ => { let _ = field.bytes().await; }
        }
    }

    if name.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Le nom est requis"));
    }

    let bytes = file_bytes.ok_or_else(|| err(StatusCode::BAD_REQUEST, "Aucun fichier fourni"))?;
    let (tenors, rows) = parse_rate_matrix_file(bytes)
        .map_err(|e| err(StatusCode::UNPROCESSABLE_ENTITY, e))?;

    let tenors_json = serde_json::to_string(&tenors)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let rows_json = serde_json::to_string(&rows)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO rate_matrices (id, name, description, currency, status, interp_method, tenors_json, rows_json)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&id)
    .bind(&name)
    .bind(&description)
    .bind(&currency)
    .bind(&status)
    .bind(&interp_method)
    .bind(&tenors_json)
    .bind(&rows_json)
    .execute(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    for risk_key in &risk_keys {
        sqlx::query(
            "INSERT INTO rate_matrix_risks (matrix_id, risk_key) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(&id)
        .bind(risk_key)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    fetch_detail(&state, &id).await
}

pub async fn get_rate_matrix(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RateMatrixDetail>, (StatusCode, String)> {
    fetch_detail(&state, &id).await
}

pub async fn update_rate_matrix(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateRequest>,
) -> Result<Json<RateMatrixDetail>, (StatusCode, String)> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rate_matrices WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if count == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Matrice introuvable"));
    }

    if let Some(v) = &body.name {
        sqlx::query("UPDATE rate_matrices SET name = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.description {
        sqlx::query("UPDATE rate_matrices SET description = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.currency {
        sqlx::query("UPDATE rate_matrices SET currency = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.status {
        sqlx::query("UPDATE rate_matrices SET status = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.interp_method {
        sqlx::query("UPDATE rate_matrices SET interp_method = $1 WHERE id = $2")
            .bind(v).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(risks) = &body.risks {
        sqlx::query("DELETE FROM rate_matrix_risks WHERE matrix_id = $1")
            .bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        for risk_key in risks {
            sqlx::query(
                "INSERT INTO rate_matrix_risks (matrix_id, risk_key) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(&id).bind(risk_key).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
    }

    fetch_detail(&state, &id).await
}

pub async fn delete_rate_matrix(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM rate_matrices WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if res.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Matrice introuvable"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ── Utilitaire interne ────────────────────────────────────────────────────────

async fn fetch_detail(
    state: &AppState,
    id: &str,
) -> Result<Json<RateMatrixDetail>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, MatrixDetailRow>(
        "SELECT id, name, description, currency, status, interp_method,
                tenors_json, rows_json, created_at
         FROM rate_matrices WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Matrice introuvable"))?;

    let risks: Vec<String> = sqlx::query_scalar(
        "SELECT risk_key FROM rate_matrix_risks WHERE matrix_id = $1 ORDER BY risk_key"
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let tenors: Vec<String> = serde_json::from_str(&row.tenors_json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let rows: Vec<MatrixRow> = serde_json::from_str(&row.rows_json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(RateMatrixDetail {
        id: row.id,
        name: row.name,
        description: row.description,
        currency: row.currency,
        status: row.status,
        interp_method: row.interp_method,
        tenors,
        rows,
        created_at: row.created_at,
        risks,
    }))
}

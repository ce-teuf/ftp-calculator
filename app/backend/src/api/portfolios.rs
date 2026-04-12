use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use uuid::Uuid;

use crate::db::AppState;

// ── Error helper ──────────────────────────────────────────────────────────────

fn err(status: StatusCode, msg: impl ToString) -> (StatusCode, String) {
    (status, msg.to_string())
}

// ── Parseurs de cellules Excel (partagés) ─────────────────────────────────────

fn cell_str(cell: &Data) -> Option<String> {
    match cell {
        Data::String(s) => { let s = s.trim().to_string(); if s.is_empty() { None } else { Some(s) } }
        Data::Float(f)  => Some(f.to_string()),
        Data::Int(i)    => Some(i.to_string()),
        Data::Bool(b)   => Some(b.to_string()),
        Data::DateTimeIso(s) => Some(s.clone()),
        _ => None,
    }
}

fn cell_ym(cell: &Data) -> Option<String> {
    match cell {
        Data::String(s) => {
            let s = s.trim();
            if s.len() >= 7 && s.chars().nth(4) == Some('-') { Some(s[..7].to_string()) } else { None }
        }
        Data::DateTimeIso(s) => if s.len() >= 7 { Some(s[..7].to_string()) } else { None },
        Data::DateTime(dt) => excel_ym(dt.as_f64()),
        Data::Float(f)     => excel_ym(*f),
        _ => None,
    }
}

fn excel_ym(serial: f64) -> Option<String> {
    let days  = if serial >= 61.0 { serial as i64 - 2 } else { serial as i64 - 1 };
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let date  = epoch.checked_add_signed(Duration::days(days))?;
    Some(date.format("%Y-%m").to_string())
}

fn cell_f64(cell: &Data) -> Option<f64> {
    match cell {
        Data::Float(f)  => Some(*f),
        Data::Int(i)    => Some(*i as f64),
        Data::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn validate_period_type(pt: &str, seen_projected: bool, base_type: &mut Option<String>, row_num: usize)
    -> Result<(), String>
{
    match pt {
        "observed" | "contrafactual" | "projected" => {}
        other => return Err(format!("Ligne {row_num}: period_type invalide '{other}'")),
    }
    if pt == "projected" { return Ok(()); }
    if seen_projected {
        return Err(format!("Ligne {row_num}: '{pt}' après 'projected'"));
    }
    match base_type {
        None => *base_type = Some(pt.to_string()),
        Some(b) if b != pt => return Err(format!("Ligne {row_num}: mélange '{b}' et '{pt}'")),
        _ => {}
    }
    Ok(())
}

// ── Modèles domaine ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorRow {
    pub date: String,
    pub period_type: String,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleRow {
    pub date: String,
    pub period_type: String,
    pub buckets: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct VectorSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub row_count: i64,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct VectorDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rows: Vec<VectorRow>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ScheduleSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub bucket_labels: Vec<String>,
    pub row_count: i64,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ScheduleDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub bucket_labels: Vec<String>,
    pub rows: Vec<ScheduleRow>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PairInfo {
    pub id: String,
    pub vector_id: String,
    pub vector_name: String,
    pub schedule_id: String,
    pub schedule_name: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PortfolioSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub vector_count: i64,
    pub schedule_count: i64,
    pub pair_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PortfolioDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub vectors: Vec<VectorSummary>,
    pub schedules: Vec<ScheduleSummary>,
    pub pairs: Vec<PairInfo>,
}

// ── Requêtes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreatePortfolioReq {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePortfolioReq {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssociateReq {
    pub id: String,  // vector_id ou schedule_id
}

#[derive(Debug, Deserialize)]
pub struct CreatePairReq {
    pub vector_id: String,
    pub schedule_id: String,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNameDescReq {
    pub name: Option<String>,
    pub description: Option<String>,
}

// ── Structs sqlx internes ─────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct PortfolioRow {
    id: String,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct VectorRow_ {
    id: String,
    name: String,
    description: Option<String>,
    rows_json: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct ScheduleRow_ {
    id: String,
    name: String,
    description: Option<String>,
    bucket_labels_json: String,
    rows_json: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct PairRow {
    id: String,
    vector_id: String,
    vector_name: String,
    schedule_id: String,
    schedule_name: String,
    label: Option<String>,
}

// ── Parseurs de fichiers ──────────────────────────────────────────────────────

fn parse_vector_file(bytes: Vec<u8>) -> Result<Vec<VectorRow>, String> {
    let cursor = Cursor::new(bytes);
    let mut wb  = open_workbook_auto_from_rs(cursor)
        .map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;
    let sheet = wb.worksheet_range_at(0)
        .ok_or("Le fichier ne contient aucune feuille")?
        .map_err(|e| format!("Erreur lecture feuille : {e}"))?;

    let mut rows_iter = sheet.rows();
    let header = rows_iter.next().ok_or("Fichier vide")?;
    if header.len() < 3 {
        return Err("3 colonnes requises : date_month, period_type, value".into());
    }
    let h0 = cell_str(&header[0]).unwrap_or_default().to_lowercase();
    let h1 = cell_str(&header[1]).unwrap_or_default().to_lowercase();
    if h0 != "date_month" { return Err(format!("Colonne A doit être 'date_month', trouvé '{h0}'")); }
    if h1 != "period_type" { return Err(format!("Colonne B doit être 'period_type', trouvé '{h1}'")); }

    let mut parsed = Vec::new();
    let mut seen_projected = false;
    let mut base_type: Option<String> = None;

    for (i, row) in rows_iter.enumerate() {
        if row.iter().all(|c| matches!(c, Data::Empty)) { continue; }
        let row_num = i + 2;

        let date = cell_ym(&row[0])
            .ok_or_else(|| format!("Ligne {row_num}: date invalide"))?;
        let pt = row.get(1).and_then(|c| cell_str(c))
            .ok_or_else(|| format!("Ligne {row_num}: period_type manquant"))?
            .to_lowercase();
        validate_period_type(&pt, seen_projected, &mut base_type, row_num)?;
        if pt == "projected" { seen_projected = true; }

        let value = row.get(2).and_then(|c| cell_f64(c)).unwrap_or(0.0);
        parsed.push(VectorRow { date, period_type: pt, value });
    }

    if parsed.is_empty() { return Err("Aucune ligne de données".into()); }
    Ok(parsed)
}

fn parse_schedule_file(bytes: Vec<u8>) -> Result<(Vec<String>, Vec<ScheduleRow>), String> {
    let cursor = Cursor::new(bytes);
    let mut wb  = open_workbook_auto_from_rs(cursor)
        .map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;
    let sheet = wb.worksheet_range_at(0)
        .ok_or("Le fichier ne contient aucune feuille")?
        .map_err(|e| format!("Erreur lecture feuille : {e}"))?;

    let mut rows_iter = sheet.rows();
    let header = rows_iter.next().ok_or("Fichier vide")?;
    if header.len() < 3 {
        return Err("Au moins 3 colonnes requises : date_month, period_type, puis les buckets".into());
    }
    let h0 = cell_str(&header[0]).unwrap_or_default().to_lowercase();
    let h1 = cell_str(&header[1]).unwrap_or_default().to_lowercase();
    if h0 != "date_month" { return Err(format!("Colonne A doit être 'date_month', trouvé '{h0}'")); }
    if h1 != "period_type" { return Err(format!("Colonne B doit être 'period_type', trouvé '{h1}'")); }

    let bucket_labels: Vec<String> = header[2..].iter().filter_map(|c| cell_str(c)).collect();
    if bucket_labels.is_empty() { return Err("Aucun bucket trouvé (colonnes à partir de C)".into()); }
    let n = bucket_labels.len();

    let mut parsed = Vec::new();
    let mut seen_projected = false;
    let mut base_type: Option<String> = None;

    for (i, row) in rows_iter.enumerate() {
        if row.iter().all(|c| matches!(c, Data::Empty)) { continue; }
        let row_num = i + 2;

        let date = cell_ym(&row[0])
            .ok_or_else(|| format!("Ligne {row_num}: date invalide"))?;
        let pt = row.get(1).and_then(|c| cell_str(c))
            .ok_or_else(|| format!("Ligne {row_num}: period_type manquant"))?
            .to_lowercase();
        validate_period_type(&pt, seen_projected, &mut base_type, row_num)?;
        if pt == "projected" { seen_projected = true; }

        let mut buckets: Vec<f64> = row.get(2..)
            .map(|cells| cells.iter().take(n).map(|c| cell_f64(c).unwrap_or(0.0)).collect())
            .unwrap_or_default();
        while buckets.len() < n { buckets.push(0.0); }

        parsed.push(ScheduleRow { date, period_type: pt, buckets });
    }

    if parsed.is_empty() { return Err("Aucune ligne de données".into()); }
    Ok((bucket_labels, parsed))
}

// ── Utilitaires internes ──────────────────────────────────────────────────────

fn rows_summary(rows_json: &str) -> (i64, Option<String>, Option<String>) {
    let rows: Vec<serde_json::Value> = serde_json::from_str(rows_json).unwrap_or_default();
    let n = rows.len() as i64;
    let from = rows.first().and_then(|r| r.get("date")).and_then(|v| v.as_str()).map(String::from);
    let to   = rows.last() .and_then(|r| r.get("date")).and_then(|v| v.as_str()).map(String::from);
    (n, from, to)
}

async fn fetch_portfolio_detail(
    state: &AppState,
    id: &str,
) -> Result<Json<PortfolioDetail>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, PortfolioRow>(
        "SELECT id, name, description, created_at FROM portfolios WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool).await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Portfolio introuvable"))?;

    // Vecteurs associés
    let v_rows = sqlx::query_as::<_, VectorRow_>(
        r#"SELECT v.id, v.name, v.description, v.rows_json, v.created_at
           FROM outstanding_vectors v
           JOIN portfolio_vectors pv ON pv.vector_id = v.id
           WHERE pv.portfolio_id = $1
           ORDER BY v.name"#
    )
    .bind(id).fetch_all(&state.pool).await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let vectors = v_rows.into_iter().map(|r| {
        let (row_count, date_from, date_to) = rows_summary(&r.rows_json);
        VectorSummary { id: r.id, name: r.name, description: r.description, row_count, date_from, date_to, created_at: r.created_at }
    }).collect();

    // Schedules associés
    let s_rows = sqlx::query_as::<_, ScheduleRow_>(
        r#"SELECT s.id, s.name, s.description, s.bucket_labels_json, s.rows_json, s.created_at
           FROM amort_schedules s
           JOIN portfolio_schedules ps ON ps.schedule_id = s.id
           WHERE ps.portfolio_id = $1
           ORDER BY s.name"#
    )
    .bind(id).fetch_all(&state.pool).await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let schedules = s_rows.into_iter().map(|r| {
        let (row_count, date_from, date_to) = rows_summary(&r.rows_json);
        let bucket_labels: Vec<String> = serde_json::from_str(&r.bucket_labels_json).unwrap_or_default();
        ScheduleSummary { id: r.id, name: r.name, description: r.description, bucket_labels, row_count, date_from, date_to, created_at: r.created_at }
    }).collect();

    // Paires
    let pairs = sqlx::query_as::<_, PairRow>(
        r#"SELECT p.id, p.vector_id, v.name AS vector_name, p.schedule_id, s.name AS schedule_name, p.label
           FROM portfolio_pairs p
           JOIN outstanding_vectors v ON v.id = p.vector_id
           JOIN amort_schedules     s ON s.id = p.schedule_id
           WHERE p.portfolio_id = $1
           ORDER BY v.name, s.name"#
    )
    .bind(id).fetch_all(&state.pool).await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let pairs = pairs.into_iter().map(|r| PairInfo {
        id: r.id, vector_id: r.vector_id, vector_name: r.vector_name,
        schedule_id: r.schedule_id, schedule_name: r.schedule_name, label: r.label,
    }).collect();

    Ok(Json(PortfolioDetail {
        id: row.id, name: row.name, description: row.description,
        created_at: row.created_at, vectors, schedules, pairs,
    }))
}

// ── Handlers — Portfolios ─────────────────────────────────────────────────────

pub async fn list_portfolios(
    State(state): State<AppState>,
) -> Result<Json<Vec<PortfolioSummary>>, (StatusCode, String)> {
    #[derive(sqlx::FromRow)]
    struct Row { id: String, name: String, description: Option<String>, created_at: DateTime<Utc>,
                 vector_count: i64, schedule_count: i64, pair_count: i64 }

    let rows = sqlx::query_as::<_, Row>(
        r#"SELECT p.id, p.name, p.description, p.created_at,
                  COUNT(DISTINCT pv.vector_id)   AS vector_count,
                  COUNT(DISTINCT ps.schedule_id) AS schedule_count,
                  COUNT(DISTINCT pp.id)          AS pair_count
           FROM portfolios p
           LEFT JOIN portfolio_vectors  pv ON pv.portfolio_id = p.id
           LEFT JOIN portfolio_schedules ps ON ps.portfolio_id = p.id
           LEFT JOIN portfolio_pairs    pp ON pp.portfolio_id = p.id
           GROUP BY p.id, p.name, p.description, p.created_at
           ORDER BY p.created_at DESC"#
    )
    .fetch_all(&state.pool).await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(rows.into_iter().map(|r| PortfolioSummary {
        id: r.id, name: r.name, description: r.description, created_at: r.created_at,
        vector_count: r.vector_count, schedule_count: r.schedule_count, pair_count: r.pair_count,
    }).collect()))
}

pub async fn create_portfolio(
    State(state): State<AppState>,
    Json(body): Json<CreatePortfolioReq>,
) -> Result<Json<PortfolioDetail>, (StatusCode, String)> {
    if body.name.trim().is_empty() { return Err(err(StatusCode::BAD_REQUEST, "Le nom est requis")); }
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO portfolios (id, name, description) VALUES ($1, $2, $3)")
        .bind(&id).bind(body.name.trim()).bind(&body.description)
        .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    fetch_portfolio_detail(&state, &id).await
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PortfolioDetail>, (StatusCode, String)> {
    fetch_portfolio_detail(&state, &id).await
}

pub async fn update_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdatePortfolioReq>,
) -> Result<Json<PortfolioDetail>, (StatusCode, String)> {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portfolios WHERE id = $1")
        .bind(&id).fetch_one(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if n == 0 { return Err(err(StatusCode::NOT_FOUND, "Portfolio introuvable")); }

    if let Some(v) = &body.name {
        sqlx::query("UPDATE portfolios SET name = $1 WHERE id = $2").bind(v).bind(&id)
            .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.description {
        sqlx::query("UPDATE portfolios SET description = $1 WHERE id = $2").bind(v).bind(&id)
            .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    fetch_portfolio_detail(&state, &id).await
}

pub async fn delete_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM portfolios WHERE id = $1")
        .bind(&id).execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if res.rows_affected() == 0 { return Err(err(StatusCode::NOT_FOUND, "Portfolio introuvable")); }
    Ok(StatusCode::NO_CONTENT)
}

// ── Handlers — Associations vecteurs ─────────────────────────────────────────

pub async fn add_vector_to_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AssociateReq>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query(
        "INSERT INTO portfolio_vectors (portfolio_id, vector_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
    .bind(&id).bind(&body.id)
    .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_vector_from_portfolio(
    State(state): State<AppState>,
    Path((portfolio_id, vector_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM portfolio_vectors WHERE portfolio_id = $1 AND vector_id = $2")
        .bind(&portfolio_id).bind(&vector_id)
        .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Handlers — Associations schedules ────────────────────────────────────────

pub async fn add_schedule_to_portfolio(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AssociateReq>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query(
        "INSERT INTO portfolio_schedules (portfolio_id, schedule_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
    .bind(&id).bind(&body.id)
    .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_schedule_from_portfolio(
    State(state): State<AppState>,
    Path((portfolio_id, schedule_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM portfolio_schedules WHERE portfolio_id = $1 AND schedule_id = $2")
        .bind(&portfolio_id).bind(&schedule_id)
        .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Handlers — Paires ────────────────────────────────────────────────────────

pub async fn create_pair(
    State(state): State<AppState>,
    Path(portfolio_id): Path<String>,
    Json(body): Json<CreatePairReq>,
) -> Result<Json<PairInfo>, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO portfolio_pairs (id, portfolio_id, vector_id, schedule_id, label)
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&id).bind(&portfolio_id).bind(&body.vector_id).bind(&body.schedule_id).bind(&body.label)
    .execute(&state.pool).await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique") { err(StatusCode::CONFLICT, "Cette paire existe déjà dans le portfolio") }
        else { err(StatusCode::INTERNAL_SERVER_ERROR, e) }
    })?;

    let row = sqlx::query_as::<_, PairRow>(
        r#"SELECT p.id, p.vector_id, v.name AS vector_name, p.schedule_id, s.name AS schedule_name, p.label
           FROM portfolio_pairs p
           JOIN outstanding_vectors v ON v.id = p.vector_id
           JOIN amort_schedules     s ON s.id = p.schedule_id
           WHERE p.id = $1"#
    )
    .bind(&id).fetch_one(&state.pool).await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(PairInfo {
        id: row.id, vector_id: row.vector_id, vector_name: row.vector_name,
        schedule_id: row.schedule_id, schedule_name: row.schedule_name, label: row.label,
    }))
}

pub async fn delete_pair(
    State(state): State<AppState>,
    Path((_portfolio_id, pair_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM portfolio_pairs WHERE id = $1")
        .bind(&pair_id).execute(&state.pool).await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Handlers — Outstanding vectors ───────────────────────────────────────────

pub async fn list_vectors(
    State(state): State<AppState>,
) -> Result<Json<Vec<VectorSummary>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, VectorRow_>(
        "SELECT id, name, description, rows_json, created_at FROM outstanding_vectors ORDER BY name"
    )
    .fetch_all(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(rows.into_iter().map(|r| {
        let (row_count, date_from, date_to) = rows_summary(&r.rows_json);
        VectorSummary { id: r.id, name: r.name, description: r.description, row_count, date_from, date_to, created_at: r.created_at }
    }).collect()))
}

pub async fn create_vector(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<VectorDetail>, (StatusCode, String)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut name = String::new();
    let mut description: Option<String> = None;
    let mut portfolio_id: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))? {
        match field.name() {
            Some("file")         => { file_bytes = Some(field.bytes().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?.to_vec()); }
            Some("name")         => { name = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?; }
            Some("description")  => { let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?; if !v.is_empty() { description = Some(v); } }
            Some("portfolio_id") => { let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?; if !v.is_empty() { portfolio_id = Some(v); } }
            _ => { let _ = field.bytes().await; }
        }
    }

    if name.trim().is_empty() { return Err(err(StatusCode::BAD_REQUEST, "Le nom est requis")); }
    let bytes = file_bytes.ok_or_else(|| err(StatusCode::BAD_REQUEST, "Aucun fichier fourni"))?;
    let rows  = parse_vector_file(bytes).map_err(|e| err(StatusCode::UNPROCESSABLE_ENTITY, e))?;
    let rows_json = serde_json::to_string(&rows).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO outstanding_vectors (id, name, description, rows_json) VALUES ($1, $2, $3, $4)")
        .bind(&id).bind(name.trim()).bind(&description).bind(&rows_json)
        .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if let Some(pid) = portfolio_id {
        sqlx::query("INSERT INTO portfolio_vectors (portfolio_id, vector_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(&pid).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    let r = sqlx::query_as::<_, VectorRow_>(
        "SELECT id, name, description, rows_json, created_at FROM outstanding_vectors WHERE id = $1"
    ).bind(&id).fetch_one(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let rows: Vec<VectorRow> = serde_json::from_str(&r.rows_json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(VectorDetail { id: r.id, name: r.name, description: r.description, rows, created_at: r.created_at }))
}

pub async fn get_vector(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<VectorDetail>, (StatusCode, String)> {
    let r = sqlx::query_as::<_, VectorRow_>(
        "SELECT id, name, description, rows_json, created_at FROM outstanding_vectors WHERE id = $1"
    ).bind(&id).fetch_optional(&state.pool).await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Vecteur introuvable"))?;

    let rows: Vec<VectorRow> = serde_json::from_str(&r.rows_json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(VectorDetail { id: r.id, name: r.name, description: r.description, rows, created_at: r.created_at }))
}

pub async fn update_vector(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateNameDescReq>,
) -> Result<Json<VectorDetail>, (StatusCode, String)> {
    if let Some(v) = &body.name {
        sqlx::query("UPDATE outstanding_vectors SET name = $1 WHERE id = $2").bind(v).bind(&id)
            .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.description {
        sqlx::query("UPDATE outstanding_vectors SET description = $1 WHERE id = $2").bind(v).bind(&id)
            .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    get_vector(State(state), Path(id)).await
}

pub async fn delete_vector(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM outstanding_vectors WHERE id = $1")
        .bind(&id).execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if res.rows_affected() == 0 { return Err(err(StatusCode::NOT_FOUND, "Vecteur introuvable")); }
    Ok(StatusCode::NO_CONTENT)
}

// ── Handlers — Amortization schedules ────────────────────────────────────────

pub async fn list_schedules(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScheduleSummary>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, ScheduleRow_>(
        "SELECT id, name, description, bucket_labels_json, rows_json, created_at FROM amort_schedules ORDER BY name"
    )
    .fetch_all(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(rows.into_iter().map(|r| {
        let (row_count, date_from, date_to) = rows_summary(&r.rows_json);
        let bucket_labels: Vec<String> = serde_json::from_str(&r.bucket_labels_json).unwrap_or_default();
        ScheduleSummary { id: r.id, name: r.name, description: r.description, bucket_labels, row_count, date_from, date_to, created_at: r.created_at }
    }).collect()))
}

pub async fn create_schedule(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ScheduleDetail>, (StatusCode, String)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut name = String::new();
    let mut description: Option<String> = None;
    let mut portfolio_id: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))? {
        match field.name() {
            Some("file")         => { file_bytes = Some(field.bytes().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?.to_vec()); }
            Some("name")         => { name = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?; }
            Some("description")  => { let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?; if !v.is_empty() { description = Some(v); } }
            Some("portfolio_id") => { let v = field.text().await.map_err(|e| err(StatusCode::BAD_REQUEST, e))?; if !v.is_empty() { portfolio_id = Some(v); } }
            _ => { let _ = field.bytes().await; }
        }
    }

    if name.trim().is_empty() { return Err(err(StatusCode::BAD_REQUEST, "Le nom est requis")); }
    let bytes = file_bytes.ok_or_else(|| err(StatusCode::BAD_REQUEST, "Aucun fichier fourni"))?;
    let (bucket_labels, rows) = parse_schedule_file(bytes).map_err(|e| err(StatusCode::UNPROCESSABLE_ENTITY, e))?;
    let bucket_labels_json = serde_json::to_string(&bucket_labels).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let rows_json = serde_json::to_string(&rows).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO amort_schedules (id, name, description, bucket_labels_json, rows_json) VALUES ($1, $2, $3, $4, $5)")
        .bind(&id).bind(name.trim()).bind(&description).bind(&bucket_labels_json).bind(&rows_json)
        .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if let Some(pid) = portfolio_id {
        sqlx::query("INSERT INTO portfolio_schedules (portfolio_id, schedule_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(&pid).bind(&id).execute(&state.pool).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    let r = sqlx::query_as::<_, ScheduleRow_>(
        "SELECT id, name, description, bucket_labels_json, rows_json, created_at FROM amort_schedules WHERE id = $1"
    ).bind(&id).fetch_one(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let rows: Vec<ScheduleRow> = serde_json::from_str(&r.rows_json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let bucket_labels: Vec<String> = serde_json::from_str(&r.bucket_labels_json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(ScheduleDetail { id: r.id, name: r.name, description: r.description, bucket_labels, rows, created_at: r.created_at }))
}

pub async fn get_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ScheduleDetail>, (StatusCode, String)> {
    let r = sqlx::query_as::<_, ScheduleRow_>(
        "SELECT id, name, description, bucket_labels_json, rows_json, created_at FROM amort_schedules WHERE id = $1"
    ).bind(&id).fetch_optional(&state.pool).await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Schedule introuvable"))?;

    let rows: Vec<ScheduleRow> = serde_json::from_str(&r.rows_json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let bucket_labels: Vec<String> = serde_json::from_str(&r.bucket_labels_json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(ScheduleDetail { id: r.id, name: r.name, description: r.description, bucket_labels, rows, created_at: r.created_at }))
}

pub async fn update_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateNameDescReq>,
) -> Result<Json<ScheduleDetail>, (StatusCode, String)> {
    if let Some(v) = &body.name {
        sqlx::query("UPDATE amort_schedules SET name = $1 WHERE id = $2").bind(v).bind(&id)
            .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(v) = &body.description {
        sqlx::query("UPDATE amort_schedules SET description = $1 WHERE id = $2").bind(v).bind(&id)
            .execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    get_schedule(State(state), Path(id)).await
}

pub async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM amort_schedules WHERE id = $1")
        .bind(&id).execute(&state.pool).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if res.rows_affected() == 0 { return Err(err(StatusCode::NOT_FOUND, "Schedule introuvable")); }
    Ok(StatusCode::NO_CONTENT)
}

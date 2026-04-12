use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    pub id:          String,
    pub name:        String,
    pub description: Option<String>,
    pub status:      String,
    pub source:      String,
    pub as_of_date:  Option<String>,
    pub created_at:  String,
    // Aggregated counts (from dataset_items)
    pub count_contracts:    i64,
    pub count_rate_curves:  i64,
    pub count_runoff_models: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateDatasetRequest {
    pub name:        String,
    pub description: Option<String>,
    pub source:      Option<String>,
    pub as_of_date:  Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DatasetItemsQuery {
    pub entity_type: Option<String>,
    pub limit:       Option<i64>,
    pub offset:      Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Contract {
    pub id:                      String,
    pub contract_id:             String,
    pub contract_type:           String,
    pub side:                    String,
    pub seller_id:               Option<String>,
    pub branch_code:             Option<String>,
    pub currency:                String,
    pub rating:                  Option<String>,
    pub notional:                f64,
    pub rate_type:               Option<String>,
    pub interest_rate:           Option<f64>,
    pub settlement_date:         Option<String>,
    pub maturity_date:           Option<String>,
    pub tenor_months:            Option<i32>,
    pub payment_frequency:       Option<String>,
    pub day_count:               Option<String>,
    pub business_day_convention: Option<String>,
    pub amortization_type:       Option<String>,
    pub prepayment_allowed:      bool,
    pub prepayment_penalty:      f64,
    pub guarantee_type:          Option<String>,
    pub runoff_model_id:         Option<String>,
    pub profiles_json:           Option<String>,
    pub rates_json:              Option<String>,
    pub risk_weight:             f64,
    pub created_at:              String,
}

// ── List datasets ─────────────────────────────────────────────────────────────

pub async fn list_datasets(
    State(state): State<AppState>,
) -> Result<Json<Vec<Dataset>>, StatusCode> {
    let rows = sqlx::query(r#"
        SELECT
            d.id, d.name, d.description, d.status, d.source,
            d.as_of_date::TEXT, d.created_at::TEXT,
            COUNT(CASE WHEN di.entity_type = 'contract'      THEN 1 END) AS count_contracts,
            COUNT(CASE WHEN di.entity_type = 'rate_curve'    THEN 1 END) AS count_rate_curves,
            COUNT(CASE WHEN di.entity_type = 'runoff_model'  THEN 1 END) AS count_runoff_models
        FROM datasets d
        LEFT JOIN dataset_items di ON di.dataset_id = d.id
        GROUP BY d.id
        ORDER BY d.created_at DESC
        LIMIT 200
    "#)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let datasets = rows.iter().map(|r| Dataset {
        id:          r.get("id"),
        name:        r.get("name"),
        description: r.get("description"),
        status:      r.get("status"),
        source:      r.get("source"),
        as_of_date:  r.get("as_of_date"),
        created_at:  r.get::<Option<String>, _>("created_at").unwrap_or_default(),
        count_contracts:     r.get("count_contracts"),
        count_rate_curves:   r.get("count_rate_curves"),
        count_runoff_models: r.get("count_runoff_models"),
    }).collect();

    Ok(Json(datasets))
}

// ── Create dataset ────────────────────────────────────────────────────────────

pub async fn create_dataset(
    State(state): State<AppState>,
    Json(req): Json<CreateDatasetRequest>,
) -> Result<Json<Dataset>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(r#"
        INSERT INTO datasets (id, name, description, source, as_of_date)
        VALUES ($1, $2, $3, $4, $5::DATE)
    "#)
    .bind(&id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(req.source.as_deref().unwrap_or("manual"))
    .bind(&req.as_of_date)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Dataset {
        id,
        name:                req.name,
        description:         req.description,
        status:              "active".into(),
        source:              req.source.unwrap_or_else(|| "manual".into()),
        as_of_date:          req.as_of_date,
        created_at:          chrono::Utc::now().to_rfc3339(),
        count_contracts:     0,
        count_rate_curves:   0,
        count_runoff_models: 0,
    }))
}

// ── Delete dataset ────────────────────────────────────────────────────────────

pub async fn delete_dataset(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM datasets WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// ── List contracts in a dataset ───────────────────────────────────────────────

pub async fn list_dataset_contracts(
    State(state): State<AppState>,
    Path(dataset_id): Path<String>,
    Query(q): Query<DatasetItemsQuery>,
) -> Result<Json<Vec<Contract>>, StatusCode> {
    let limit  = q.limit.unwrap_or(200);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query(r#"
        SELECT c.*
        FROM contracts c
        JOIN dataset_items di ON di.entity_id = c.id AND di.entity_type = 'contract'
        WHERE di.dataset_id = $1
        ORDER BY c.created_at DESC
        LIMIT $2 OFFSET $3
    "#)
    .bind(&dataset_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let contracts = rows.iter().map(row_to_contract).collect();
    Ok(Json(contracts))
}

fn row_to_contract(r: &sqlx::postgres::PgRow) -> Contract {
    Contract {
        id:                      r.get("id"),
        contract_id:             r.get("contract_id"),
        contract_type:           r.get("contract_type"),
        side:                    r.get("side"),
        seller_id:               r.get("seller_id"),
        branch_code:             r.get("branch_code"),
        currency:                r.get("currency"),
        rating:                  r.get("rating"),
        notional:                r.get("notional"),
        rate_type:               r.get("rate_type"),
        interest_rate:           r.get("interest_rate"),
        settlement_date:         r.get::<Option<String>, _>("settlement_date"),
        maturity_date:           r.get::<Option<String>, _>("maturity_date"),
        tenor_months:            r.get("tenor_months"),
        payment_frequency:       r.get("payment_frequency"),
        day_count:               r.get("day_count"),
        business_day_convention: r.get("business_day_convention"),
        amortization_type:       r.get("amortization_type"),
        prepayment_allowed:      r.get::<bool, _>("prepayment_allowed"),
        prepayment_penalty:      r.get::<f64, _>("prepayment_penalty"),
        guarantee_type:          r.get("guarantee_type"),
        runoff_model_id:         r.get("runoff_model_id"),
        profiles_json:           r.get("profiles_json"),
        rates_json:              r.get("rates_json"),
        risk_weight:             r.get("risk_weight"),
        created_at:              r.get::<Option<String>, _>("created_at").unwrap_or_default(),
    }
}

// ── Upload CSV → contracts in a dataset ───────────────────────────────────────

pub async fn upload_contracts_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    let mut csv_bytes: Vec<u8> = Vec::new();
    let mut dataset_name = String::from("Upload");
    let mut dataset_id_param: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                csv_bytes = field.bytes().await.unwrap_or_default().to_vec();
            }
            "dataset_name" => {
                dataset_name = String::from_utf8(
                    field.bytes().await.unwrap_or_default().to_vec()
                ).unwrap_or_default();
            }
            "dataset_id" => {
                let v = String::from_utf8(
                    field.bytes().await.unwrap_or_default().to_vec()
                ).unwrap_or_default();
                if !v.is_empty() { dataset_id_param = Some(v); }
            }
            _ => {}
        }
    }

    if csv_bytes.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "no file received"})));
    }

    let csv_str = match String::from_utf8(csv_bytes) {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "file is not valid UTF-8"}))),
    };

    // Ensure dataset exists
    let dataset_id = match dataset_id_param {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            let _ = sqlx::query(
                "INSERT INTO datasets (id, name, source) VALUES ($1, $2, 'uploaded')"
            )
            .bind(&id)
            .bind(&dataset_name)
            .execute(&state.pool)
            .await;
            id
        }
    };

    // Parse CSV
    let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());
    let headers: Vec<String> = match rdr.headers() {
        Ok(h) => h.iter().map(|s| s.to_string()).collect(),
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("CSV header error: {e}")}))),
    };

    let mut imported = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(e) => { errors.push(e.to_string()); continue; }
        };

        // Build field map
        let row: std::collections::HashMap<&str, &str> = headers.iter()
            .zip(record.iter())
            .map(|(h, v)| (h.as_str(), v))
            .collect();

        let id = Uuid::new_v4().to_string();
        let contract_id = row.get("contract_id")
            .or_else(|| row.get("id"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("CNT-{}", &id[..8].to_uppercase()));

        let notional: f64 = row.get("notional").and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let interest_rate: Option<f64> = row.get("interest_rate").and_then(|v| v.parse().ok());
        let tenor: Option<i32> = row.get("tenor_months").and_then(|v| v.parse().ok());
        let prepay_allow: bool = row.get("prepayment_allowed")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        let prepay_pen: f64 = row.get("prepayment_penalty").and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let risk_weight: f64 = row.get("risk_weight").and_then(|v| v.parse().ok()).unwrap_or(1.0);

        // Helper: get &str from row, returning None if empty
        let opt = |key: &str| -> Option<&str> {
            row.get(key).copied().filter(|s| !s.is_empty())
        };

        let res = sqlx::query(r#"
            INSERT INTO contracts (
                id, contract_id, contract_type, side,
                seller_id, branch_code, currency, rating,
                notional, rate_type, interest_rate,
                settlement_date, maturity_date, tenor_months,
                payment_frequency, day_count, business_day_convention,
                amortization_type, prepayment_allowed, prepayment_penalty,
                guarantee_type, profiles_json, rates_json, risk_weight,
                source_dataset_id
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,
                $12::DATE,$13::DATE,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25
            )
            ON CONFLICT (contract_id) DO NOTHING
        "#)
        .bind(&id)
        .bind(&contract_id)
        .bind(row.get("contract_type").copied().unwrap_or("UNKNOWN"))
        .bind(row.get("side").copied().unwrap_or("ASSET"))
        .bind(opt("seller_id"))
        .bind(opt("branch_code"))
        .bind(row.get("currency").copied().unwrap_or("EUR"))
        .bind(opt("rating"))
        .bind(notional)
        .bind(opt("rate_type"))
        .bind(interest_rate)
        .bind(opt("settlement_date"))
        .bind(opt("maturity_date"))
        .bind(tenor)
        .bind(opt("payment_frequency"))
        .bind(opt("day_count"))
        .bind(opt("business_day_convention"))
        .bind(opt("amortization_type"))
        .bind(prepay_allow)
        .bind(prepay_pen)
        .bind(opt("guarantee_type"))
        .bind(opt("profiles_json"))
        .bind(opt("rates_json"))
        .bind(risk_weight)
        .bind(&dataset_id)
        .execute(&state.pool)
        .await;

        if let Ok(r) = res {
            if r.rows_affected() > 0 {
                // Link to dataset
                let _ = sqlx::query(
                    "INSERT INTO dataset_items (dataset_id, entity_type, entity_id)
                     VALUES ($1, 'contract', $2) ON CONFLICT DO NOTHING"
                )
                .bind(&dataset_id)
                .bind(&id)
                .execute(&state.pool)
                .await;
                imported += 1;
            }
        }
    }

    (StatusCode::OK, Json(json!({
        "dataset_id": dataset_id,
        "imported": imported,
        "errors": errors.len(),
        "error_sample": errors.into_iter().take(5).collect::<Vec<_>>(),
    })))
}

// ── Freeze: export dataset as CSV in original format ─────────────────────────

pub async fn freeze_dataset(
    State(state): State<AppState>,
    Path(dataset_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {

    // Contracts CSV
    let contracts = sqlx::query(r#"
        SELECT c.* FROM contracts c
        JOIN dataset_items di ON di.entity_id = c.id AND di.entity_type = 'contract'
        WHERE di.dataset_id = $1
        ORDER BY c.contract_id
    "#)
    .bind(&dataset_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut csv_rows: Vec<String> = Vec::new();
    csv_rows.push([
        "contract_id","contract_type","side","seller_id","branch_code","currency","notional",
        "rate_type","interest_rate","settlement_date","maturity_date","tenor_months",
        "payment_frequency","day_count","business_day_convention","amortization_type",
        "prepayment_allowed","prepayment_penalty","guarantee_type","rating",
        "profiles_json","rates_json","risk_weight",
    ].join(","));

    for r in &contracts {
        let line = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.get::<String, _>("contract_id"),
            r.get::<String, _>("contract_type"),
            r.get::<String, _>("side"),
            r.get::<Option<String>, _>("seller_id").unwrap_or_default(),
            r.get::<Option<String>, _>("branch_code").unwrap_or_default(),
            r.get::<String, _>("currency"),
            r.get::<f64, _>("notional"),
            r.get::<Option<String>, _>("rate_type").unwrap_or_default(),
            r.get::<Option<f64>, _>("interest_rate").map(|v| v.to_string()).unwrap_or_default(),
            r.get::<Option<String>, _>("settlement_date").unwrap_or_default(),
            r.get::<Option<String>, _>("maturity_date").unwrap_or_default(),
            r.get::<Option<i32>, _>("tenor_months").map(|v| v.to_string()).unwrap_or_default(),
            r.get::<Option<String>, _>("payment_frequency").unwrap_or_default(),
            r.get::<Option<String>, _>("day_count").unwrap_or_default(),
            r.get::<Option<String>, _>("business_day_convention").unwrap_or_default(),
            r.get::<Option<String>, _>("amortization_type").unwrap_or_default(),
            r.get::<bool, _>("prepayment_allowed"),
            r.get::<f64, _>("prepayment_penalty"),
            r.get::<Option<String>, _>("guarantee_type").unwrap_or_default(),
            r.get::<Option<String>, _>("rating").unwrap_or_default(),
            r.get::<Option<String>, _>("profiles_json").unwrap_or_default().replace(",", ";"),
            r.get::<Option<String>, _>("rates_json").unwrap_or_default().replace(",", ";"),
            r.get::<f64, _>("risk_weight"),
        );
        csv_rows.push(line);
    }

    // Rate curves CSV
    let curves = sqlx::query(r#"
        SELECT rc.* FROM rate_curves rc
        JOIN dataset_items di ON di.entity_id = rc.id AND di.entity_type = 'rate_curve'
        WHERE di.dataset_id = $1
    "#)
    .bind(&dataset_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut curves_csv: Vec<String> = Vec::new();
    curves_csv.push("id,name,component,currency,status,tenors_json,values_json,source".into());
    for r in &curves {
        curves_csv.push(format!(
            "{},{},{},{},{},{},{},{}",
            r.get::<String, _>("id"),
            r.get::<String, _>("name"),
            r.get::<String, _>("component"),
            r.get::<String, _>("currency"),
            r.get::<String, _>("status"),
            r.get::<String, _>("tenors_json").replace(",", ";"),
            r.get::<String, _>("values_json").replace(",", ";"),
            r.get::<Option<String>, _>("source").unwrap_or_default(),
        ));
    }

    // Log freeze
    let freeze_id = Uuid::new_v4().to_string();
    let _ = sqlx::query(
        "INSERT INTO freeze_log (id, dataset_id, row_counts) VALUES ($1, $2, $3)"
    )
    .bind(&freeze_id)
    .bind(&dataset_id)
    .bind(serde_json::to_string(&json!({
        "contracts": contracts.len(),
        "rate_curves": curves.len(),
    })).unwrap_or_default())
    .execute(&state.pool)
    .await;

    Ok(Json(json!({
        "freeze_id":         freeze_id,
        "dataset_id":        dataset_id,
        "contracts_csv":     csv_rows.join("\n"),
        "rate_curves_csv":   curves_csv.join("\n"),
        "contracts_count":   contracts.len(),
        "rate_curves_count": curves.len(),
        "frozen_at":         chrono::Utc::now().to_rfc3339(),
    })))
}

// ── Add items to a dataset (by entity_type + entity_id list) ─────────────────

#[derive(Debug, Deserialize)]
pub struct AddItemsRequest {
    pub entity_type: String,
    pub entity_ids:  Vec<String>,
}

pub async fn add_items_to_dataset(
    State(state): State<AppState>,
    Path(dataset_id): Path<String>,
    Json(req): Json<AddItemsRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut added = 0usize;
    for eid in &req.entity_ids {
        let res = sqlx::query(
            "INSERT INTO dataset_items (dataset_id, entity_type, entity_id)
             VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
        )
        .bind(&dataset_id)
        .bind(&req.entity_type)
        .bind(eid)
        .execute(&state.pool)
        .await;
        if let Ok(r) = res { if r.rows_affected() > 0 { added += 1; } }
    }
    Ok(Json(json!({ "added": added })))
}

// ── Dataset summary (item counts per type) ────────────────────────────────────

pub async fn dataset_summary(
    State(state): State<AppState>,
    Path(dataset_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let rows = sqlx::query(r#"
        SELECT entity_type, COUNT(*) as cnt
        FROM dataset_items
        WHERE dataset_id = $1
        GROUP BY entity_type
    "#)
    .bind(&dataset_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut summary = serde_json::Map::new();
    for r in rows {
        let et: String = r.get("entity_type");
        let cnt: i64   = r.get("cnt");
        summary.insert(et, json!(cnt));
    }

    Ok(Json(json!({ "dataset_id": dataset_id, "counts": summary })))
}

// ── Helpers for filesystem datasets ──────────────────────────────────────────

fn datasets_dir() -> PathBuf {
    let dir = std::env::var("DATASETS_DIR")
        .unwrap_or_else(|_| "../../data/datageneration_scripts/datasets".to_string());
    PathBuf::from(dir)
}

fn read_fs_meta(folder: &std::path::Path) -> Option<Value> {
    let meta_path = folder.join("dataset_meta.json");
    let content = std::fs::read_to_string(&meta_path).ok()?;
    serde_json::from_str(&content).ok()
}

// ── List available filesystem datasets ───────────────────────────────────────

pub async fn list_available_datasets(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let dir = datasets_dir();

    // All dataset IDs currently in DB
    let db_rows = sqlx::query("SELECT id FROM datasets")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let loaded_ids: std::collections::HashSet<String> =
        db_rows.iter().map(|r| r.get::<String, _>("id")).collect();

    let mut result: Vec<Value> = Vec::new();

    let entries = std::fs::read_dir(&dir)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let Some(meta) = read_fs_meta(&path) else { continue };

        let ds_id = meta.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let folder_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        result.push(json!({
            "folder":       folder_name,
            "loaded_in_db": loaded_ids.contains(&ds_id),
            "meta":         meta,
        }));
    }

    // Sort by name
    result.sort_by(|a, b| {
        let na = a["meta"]["name"].as_str().unwrap_or("");
        let nb = b["meta"]["name"].as_str().unwrap_or("");
        na.cmp(nb)
    });

    Ok(Json(json!({ "datasets_dir": dir.to_string_lossy(), "datasets": result })))
}

// ── Load a filesystem dataset into the DB ─────────────────────────────────────

pub async fn load_fs_dataset(
    State(state): State<AppState>,
    Path(folder): Path<String>,
) -> (StatusCode, Json<Value>) {
    let dir = datasets_dir().join(&folder);
    if !dir.is_dir() {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "folder not found"})));
    }

    let meta = match read_fs_meta(&dir) {
        Some(m) => m,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "dataset_meta.json missing"}))),
    };

    let dataset_id   = meta["id"].as_str().unwrap_or("").to_string();
    let dataset_name = meta["name"].as_str().unwrap_or("Unknown").to_string();
    let description  = meta["description"].as_str().unwrap_or("").to_string();
    let as_of_date   = meta["as_of_date"].as_str().map(|s| s.to_string());

    // Check not already loaded
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM datasets WHERE id = $1")
        .bind(&dataset_id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    if existing > 0 {
        return (StatusCode::CONFLICT, Json(json!({"error": "dataset already loaded", "dataset_id": dataset_id})));
    }

    // ── Insert dataset row ────────────────────────────────────────────────────
    let ins = sqlx::query("INSERT INTO datasets (id, name, description, status, source, as_of_date) VALUES ($1,$2,$3,'active','generated',$4::DATE)")
        .bind(&dataset_id).bind(&dataset_name).bind(&description).bind(&as_of_date)
        .execute(&state.pool).await;
    if ins.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "failed to insert dataset"})));
    }

    let mut stats = json!({"curves": 0, "runoff_models": 0, "contracts": 0});

    // ── Rate curves ───────────────────────────────────────────────────────────
    let curves_path = dir.join("rate_curves.csv");
    if curves_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&curves_path) {
            let mut rdr = csv::Reader::from_reader(content.as_bytes());
            let headers: Vec<String> = rdr.headers().ok()
                .map(|h| h.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let mut count = 0usize;
            for rec in rdr.records().flatten() {
                let row: std::collections::HashMap<&str, &str> = headers.iter()
                    .zip(rec.iter()).map(|(h, v)| (h.as_str(), v)).collect();
                let cid = row.get("id").copied().unwrap_or("").to_string();
                if cid.is_empty() { continue; }
                let _ = sqlx::query(r#"
                    INSERT INTO rate_curves (id, name, component, currency, version, status, tenors_json, values_json, source)
                    VALUES ($1,$2,$3,$4,1,'approved',$5,$6,$7)
                    ON CONFLICT (id) DO NOTHING
                "#)
                .bind(&cid)
                .bind(row.get("name").copied().unwrap_or(""))
                .bind(row.get("component").copied().unwrap_or("base_rate"))
                .bind(row.get("currency").copied().unwrap_or("EUR"))
                .bind(row.get("tenors_json").copied().unwrap_or("[]"))
                .bind(row.get("values_json").copied().unwrap_or("[]"))
                .bind(row.get("source").copied().unwrap_or("generated"))
                .execute(&state.pool).await;
                let _ = sqlx::query("INSERT INTO dataset_items (dataset_id, entity_type, entity_id) VALUES ($1,'rate_curve',$2) ON CONFLICT DO NOTHING")
                    .bind(&dataset_id).bind(&cid).execute(&state.pool).await;
                count += 1;
            }
            stats["curves"] = json!(count);
        }
    }

    // ── Runoff models ─────────────────────────────────────────────────────────
    let rm_path = dir.join("runoff_models.json");
    if rm_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&rm_path) {
            if let Ok(models) = serde_json::from_str::<Vec<Value>>(&content) {
                let mut count = 0usize;
                for m in &models {
                    let rid = m["id"].as_str().unwrap_or("").to_string();
                    if rid.is_empty() { continue; }
                    let _ = sqlx::query(r#"
                        INSERT INTO runoff_models (id, name, product_type, category, version, status, method, profile_json, parameters_json)
                        VALUES ($1,$2,$3,$4,1,'approved',$5,$6,$7)
                        ON CONFLICT (id) DO NOTHING
                    "#)
                    .bind(&rid)
                    .bind(m["name"].as_str().unwrap_or(""))
                    .bind(m["product_type"].as_str().unwrap_or("nmd"))
                    .bind(m["category"].as_str().unwrap_or("retail"))
                    .bind(m["method"].as_str().unwrap_or("behavioral_exponential"))
                    .bind(m["profile_json"].as_str().unwrap_or("[]"))
                    .bind(m["parameters_json"].as_str().unwrap_or("{}"))
                    .execute(&state.pool).await;
                    let _ = sqlx::query("INSERT INTO dataset_items (dataset_id, entity_type, entity_id) VALUES ($1,'runoff_model',$2) ON CONFLICT DO NOTHING")
                        .bind(&dataset_id).bind(&rid).execute(&state.pool).await;
                    count += 1;
                }
                stats["runoff_models"] = json!(count);
            }
        }
    }

    // ── Contracts ─────────────────────────────────────────────────────────────
    let contracts_path = dir.join("contracts.csv");
    if contracts_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&contracts_path) {
            let mut rdr = csv::Reader::from_reader(content.as_bytes());
            let headers: Vec<String> = rdr.headers().ok()
                .map(|h| h.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let mut count = 0usize;
            for rec in rdr.records().flatten() {
                let row: std::collections::HashMap<&str, &str> = headers.iter()
                    .zip(rec.iter()).map(|(h, v)| (h.as_str(), v)).collect();
                let cid = row.get("id").copied().filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| Uuid::new_v4().to_string());
                let contract_id_val = row.get("contract_id").copied().unwrap_or("");
                if contract_id_val.is_empty() { continue; }
                let notional: f64   = row.get("notional").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let irate: Option<f64>  = row.get("interest_rate").and_then(|v| v.parse().ok());
                let tenor: Option<i32>  = row.get("tenor_months").and_then(|v| v.parse().ok());
                let prepay_allow: bool  = row.get("prepayment_allowed").map(|v| v.to_lowercase() == "true").unwrap_or(false);
                let prepay_pen: f64     = row.get("prepayment_penalty").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let risk_w: f64         = row.get("risk_weight").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let opt = |key: &str| -> Option<&str> {
                    row.get(key).copied().filter(|s| !s.is_empty())
                };
                let _ = sqlx::query(r#"
                    INSERT INTO contracts (
                        id, contract_id, contract_type, side,
                        seller_id, branch_code, currency, rating,
                        notional, rate_type, interest_rate,
                        settlement_date, maturity_date, tenor_months,
                        payment_frequency, day_count, business_day_convention,
                        amortization_type, prepayment_allowed, prepayment_penalty,
                        guarantee_type, profiles_json, rates_json, risk_weight,
                        source_dataset_id
                    ) VALUES (
                        $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,
                        $12::DATE,$13::DATE,$14,$15,$16,$17,$18,$19,$20,
                        $21,$22,$23,$24,$25
                    ) ON CONFLICT (contract_id) DO NOTHING
                "#)
                .bind(&cid)
                .bind(contract_id_val)
                .bind(row.get("contract_type").copied().unwrap_or("UNKNOWN"))
                .bind(row.get("side").copied().unwrap_or("ASSET"))
                .bind(opt("seller_id")).bind(opt("branch_code"))
                .bind(row.get("currency").copied().unwrap_or("EUR"))
                .bind(opt("rating")).bind(notional)
                .bind(opt("rate_type")).bind(irate)
                .bind(opt("settlement_date")).bind(opt("maturity_date"))
                .bind(tenor)
                .bind(opt("payment_frequency")).bind(opt("day_count"))
                .bind(opt("business_day_convention")).bind(opt("amortization_type"))
                .bind(prepay_allow).bind(prepay_pen)
                .bind(opt("guarantee_type"))
                .bind(opt("profiles_json")).bind(opt("rates_json"))
                .bind(risk_w).bind(&dataset_id)
                .execute(&state.pool).await;
                let _ = sqlx::query("INSERT INTO dataset_items (dataset_id, entity_type, entity_id) VALUES ($1,'contract',$2) ON CONFLICT DO NOTHING")
                    .bind(&dataset_id).bind(&cid).execute(&state.pool).await;
                count += 1;
            }
            stats["contracts"] = json!(count);
        }
    }

    // ── Entities ──────────────────────────────────────────────────────────────
    let entity_files: &[(&str, &str, &str)] = &[
        ("entities_branches.csv",        "branch_id",  "org_branch"),
        ("entities_business_units.csv",  "bu_id",      "org_business_unit"),
        ("entities_departments.csv",     "dept_id",    "org_department"),
        ("entities_sellers.csv",         "seller_id",  "org_seller"),
        ("entities_treasuries.csv",      "treasury_id","org_treasury"),
    ];
    let mut entity_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (filename, id_col, entity_type) in entity_files {
        let path = dir.join(filename);
        if !path.exists() { continue; }
        if let Ok(content) = std::fs::read_to_string(&path) {
            let mut rdr = csv::Reader::from_reader(content.as_bytes());
            let headers: Vec<String> = rdr.headers().ok()
                .map(|h| h.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let mut count = 0usize;
            for rec in rdr.records().flatten() {
                let row: std::collections::HashMap<&str, &str> = headers.iter()
                    .zip(rec.iter()).map(|(h, v)| (h.as_str(), v)).collect();
                let eid = row.get(*id_col).copied().unwrap_or("").to_string();
                if eid.is_empty() { continue; }
                let eid_ref = eid.clone();

                match *entity_type {
                    "org_branch" => {
                        let _ = sqlx::query(r#"
                            INSERT INTO org_branches (id, branch_code, branch_name, country, currency, city, address, phone, status, created_date)
                            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10::DATE)
                            ON CONFLICT (branch_code) DO NOTHING
                        "#)
                        .bind(&eid)
                        .bind(row.get("branch_code").copied().unwrap_or(""))
                        .bind(row.get("branch_name").copied().unwrap_or(""))
                        .bind(row.get("country").copied().unwrap_or(""))
                        .bind(row.get("currency").copied().unwrap_or("EUR"))
                        .bind(row.get("city").copied().unwrap_or(""))
                        .bind(row.get("address").copied())
                        .bind(row.get("phone").copied())
                        .bind(row.get("status").copied().unwrap_or("active"))
                        .bind(row.get("created_date").copied().filter(|s| !s.is_empty()))
                        .execute(&state.pool).await;
                    }
                    "org_business_unit" => {
                        let _ = sqlx::query(r#"
                            INSERT INTO org_business_units (id, bu_name, branch_id, branch_code, currency, status, created_date)
                            VALUES ($1,$2,$3,$4,$5,$6,$7::DATE)
                            ON CONFLICT (id) DO NOTHING
                        "#)
                        .bind(&eid)
                        .bind(row.get("bu_name").copied().unwrap_or(""))
                        .bind(row.get("branch_id").copied().filter(|s| !s.is_empty()))
                        .bind(row.get("branch_code").copied().unwrap_or(""))
                        .bind(row.get("currency").copied().unwrap_or("EUR"))
                        .bind(row.get("status").copied().unwrap_or("active"))
                        .bind(row.get("created_date").copied().filter(|s| !s.is_empty()))
                        .execute(&state.pool).await;
                    }
                    "org_department" => {
                        let _ = sqlx::query(r#"
                            INSERT INTO org_departments (id, dept_name, bu_id, bu_name, branch_id, branch_code, status)
                            VALUES ($1,$2,$3,$4,$5,$6,$7)
                            ON CONFLICT (id) DO NOTHING
                        "#)
                        .bind(&eid)
                        .bind(row.get("dept_name").copied().unwrap_or(""))
                        .bind(row.get("bu_id").copied().filter(|s| !s.is_empty()))
                        .bind(row.get("bu_name").copied().unwrap_or(""))
                        .bind(row.get("branch_id").copied().filter(|s| !s.is_empty()))
                        .bind(row.get("branch_code").copied().unwrap_or(""))
                        .bind(row.get("status").copied().unwrap_or("active"))
                        .execute(&state.pool).await;
                    }
                    "org_seller" => {
                        let hire: Option<&str> = row.get("hire_date").copied().filter(|s| !s.is_empty());
                        let target_vol: Option<f64> = row.get("target_volume").and_then(|v| v.parse().ok());
                        let _ = sqlx::query(r#"
                            INSERT INTO org_sellers (id, seller_code, first_name, last_name, email, bu_id, bu_name, branch_id, branch_code, hire_date, status, target_volume, seniority)
                            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10::DATE,$11,$12,$13)
                            ON CONFLICT (id) DO NOTHING
                        "#)
                        .bind(&eid)
                        .bind(row.get("seller_code").copied().unwrap_or(""))
                        .bind(row.get("first_name").copied().unwrap_or(""))
                        .bind(row.get("last_name").copied().unwrap_or(""))
                        .bind(row.get("email").copied().unwrap_or(""))
                        .bind(row.get("bu_id").copied().filter(|s| !s.is_empty()))
                        .bind(row.get("bu_name").copied().unwrap_or(""))
                        .bind(row.get("branch_id").copied().filter(|s| !s.is_empty()))
                        .bind(row.get("branch_code").copied().unwrap_or(""))
                        .bind(hire)
                        .bind(row.get("status").copied().unwrap_or("active"))
                        .bind(target_vol)
                        .bind(row.get("seniority").copied().unwrap_or(""))
                        .execute(&state.pool).await;
                    }
                    "org_treasury" => {
                        let _ = sqlx::query(r#"
                            INSERT INTO org_treasuries (id, branch_id, branch_code, treasury_name, currency, status)
                            VALUES ($1,$2,$3,$4,$5,$6)
                            ON CONFLICT (id) DO NOTHING
                        "#)
                        .bind(&eid)
                        .bind(row.get("branch_id").copied().filter(|s| !s.is_empty()))
                        .bind(row.get("branch_code").copied().unwrap_or(""))
                        .bind(row.get("treasury_name").copied().unwrap_or(""))
                        .bind(row.get("currency").copied().unwrap_or("EUR"))
                        .bind(row.get("status").copied().unwrap_or("active"))
                        .execute(&state.pool).await;
                    }
                    _ => {}
                }

                let _ = sqlx::query("INSERT INTO dataset_items (dataset_id, entity_type, entity_id) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING")
                    .bind(&dataset_id).bind(entity_type).bind(&eid_ref).execute(&state.pool).await;
                count += 1;
            }
            entity_counts.insert(entity_type.to_string(), count);
        }
    }

    // ── Rate series (historical) ──────────────────────────────────────────────
    let rs_path = dir.join("rate_series.csv");
    let mut rs_count = 0usize;
    if rs_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&rs_path) {
            let mut rdr = csv::Reader::from_reader(content.as_bytes());
            let headers: Vec<String> = rdr.headers().ok()
                .map(|h| h.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            for rec in rdr.records().flatten() {
                let row: std::collections::HashMap<&str, &str> = headers.iter()
                    .zip(rec.iter()).map(|(h, v)| (h.as_str(), v)).collect();
                let series_name = row.get("series_name").copied().unwrap_or("").to_string();
                let obs_date    = row.get("obs_date").copied().unwrap_or("").to_string();
                let rate: f64   = row.get("rate").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                if series_name.is_empty() || obs_date.is_empty() { continue; }
                let tenor = row.get("tenor").copied().filter(|s| !s.is_empty()).map(|s| s.to_string());
                let id = format!("{series_name}-{obs_date}-{}", tenor.as_deref().unwrap_or("ON"));
                let _ = sqlx::query(r#"
                    INSERT INTO rate_series_data (id, series_name, component, currency, obs_date, tenor, rate, source)
                    VALUES ($1,$2,'base_rate',$3,$4::DATE,$5,$6,'generated')
                    ON CONFLICT DO NOTHING
                "#)
                .bind(&id)
                .bind(&series_name)
                .bind(match series_name.as_str() {
                    "ESTR"|"EURIBOR" => "EUR", "SOFR" => "USD", "SONIA" => "GBP", _ => "EUR"
                })
                .bind(&obs_date)
                .bind(&tenor)
                .bind(rate)
                .execute(&state.pool).await;
                rs_count += 1;
            }
        }
    }
    stats["entities"] = json!(entity_counts);
    stats["rate_series"] = json!(rs_count);

    (StatusCode::OK, Json(json!({
        "dataset_id": dataset_id,
        "name":       dataset_name,
        "loaded":     stats,
    })))
}

// ── Export dataset as ZIP ─────────────────────────────────────────────────────

pub async fn export_dataset_zip(
    State(state): State<AppState>,
    Path(dataset_id): Path<String>,
) -> Result<axum::response::Response, StatusCode> {
    use std::io::Write;

    // Gather dataset name
    let ds_row = sqlx::query("SELECT name FROM datasets WHERE id = $1")
        .bind(&dataset_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let ds_name: String = ds_row.get("name");

    let mut zip_buf: Vec<u8> = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut zip_buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // ── contracts.csv ─────────────────────────────────────────────────────
        {
            let rows = sqlx::query(r#"
                SELECT c.* FROM contracts c
                JOIN dataset_items di ON di.entity_id = c.id AND di.entity_type = 'contract'
                WHERE di.dataset_id = $1 ORDER BY c.contract_id
            "#).bind(&dataset_id).fetch_all(&state.pool).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let mut out = String::from("contract_id,contract_type,side,seller_id,branch_code,currency,rating,notional,rate_type,interest_rate,settlement_date,maturity_date,tenor_months,payment_frequency,day_count,business_day_convention,amortization_type,prepayment_allowed,prepayment_penalty,guarantee_type,profiles_json,rates_json,risk_weight\n");
            for r in &rows {
                let line = format!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    r.get::<String,_>("contract_id"),
                    r.get::<String,_>("contract_type"),
                    r.get::<String,_>("side"),
                    r.get::<Option<String>,_>("seller_id").unwrap_or_default(),
                    r.get::<Option<String>,_>("branch_code").unwrap_or_default(),
                    r.get::<String,_>("currency"),
                    r.get::<Option<String>,_>("rating").unwrap_or_default(),
                    r.get::<f64,_>("notional"),
                    r.get::<Option<String>,_>("rate_type").unwrap_or_default(),
                    r.get::<Option<f64>,_>("interest_rate").map(|v|v.to_string()).unwrap_or_default(),
                    r.get::<Option<String>,_>("settlement_date").unwrap_or_default(),
                    r.get::<Option<String>,_>("maturity_date").unwrap_or_default(),
                    r.get::<Option<i32>,_>("tenor_months").map(|v|v.to_string()).unwrap_or_default(),
                    r.get::<Option<String>,_>("payment_frequency").unwrap_or_default(),
                    r.get::<Option<String>,_>("day_count").unwrap_or_default(),
                    r.get::<Option<String>,_>("business_day_convention").unwrap_or_default(),
                    r.get::<Option<String>,_>("amortization_type").unwrap_or_default(),
                    r.get::<bool,_>("prepayment_allowed"),
                    r.get::<f64,_>("prepayment_penalty"),
                    r.get::<Option<String>,_>("guarantee_type").unwrap_or_default(),
                    r.get::<Option<String>,_>("profiles_json").unwrap_or_default().replace(',',";"),
                    r.get::<Option<String>,_>("rates_json").unwrap_or_default().replace(',',";"),
                    r.get::<f64,_>("risk_weight"),
                );
                out.push_str(&line);
            }
            zip.start_file("contracts.csv", opts).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            zip.write_all(out.as_bytes()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // ── rate_curves.csv ───────────────────────────────────────────────────
        {
            let rows = sqlx::query(r#"
                SELECT rc.* FROM rate_curves rc
                JOIN dataset_items di ON di.entity_id = rc.id AND di.entity_type = 'rate_curve'
                WHERE di.dataset_id = $1
            "#).bind(&dataset_id).fetch_all(&state.pool).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let mut out = String::from("id,name,component,currency,status,tenors_json,values_json,source\n");
            for r in &rows {
                out.push_str(&format!("{},{},{},{},{},{},{},{}\n",
                    r.get::<String,_>("id"),
                    r.get::<String,_>("name"),
                    r.get::<String,_>("component"),
                    r.get::<String,_>("currency"),
                    r.get::<String,_>("status"),
                    r.get::<String,_>("tenors_json").replace(',',";"),
                    r.get::<String,_>("values_json").replace(',',";"),
                    r.get::<Option<String>,_>("source").unwrap_or_default(),
                ));
            }
            zip.start_file("rate_curves.csv", opts).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            zip.write_all(out.as_bytes()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // ── rate_series.csv ───────────────────────────────────────────────────
        {
            let rows = sqlx::query(r#"
                SELECT rsd.* FROM rate_series_data rsd
                JOIN dataset_items di ON di.entity_id = rsd.id AND di.entity_type = 'rate_series_data'
                WHERE di.dataset_id = $1 ORDER BY rsd.series_name, rsd.obs_date
            "#).bind(&dataset_id).fetch_all(&state.pool).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let mut out = String::from("series_name,obs_date,tenor,rate\n");
            for r in &rows {
                out.push_str(&format!("{},{},{},{}\n",
                    r.get::<String,_>("series_name"),
                    r.get::<Option<String>,_>("obs_date").unwrap_or_default(),
                    r.get::<Option<String>,_>("tenor").unwrap_or_default(),
                    r.get::<f64,_>("rate"),
                ));
            }
            zip.start_file("rate_series.csv", opts).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            zip.write_all(out.as_bytes()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // ── entity CSVs ───────────────────────────────────────────────────────
        for (entity_type, filename, query_sql, cols) in &[
            ("org_branch", "entities_branches.csv",
             "SELECT ob.* FROM org_branches ob JOIN dataset_items di ON di.entity_id = ob.id AND di.entity_type = 'org_branch' WHERE di.dataset_id = $1",
             vec!["id","branch_code","branch_name","country","currency","city","status"]),
            ("org_business_unit", "entities_business_units.csv",
             "SELECT ob.* FROM org_business_units ob JOIN dataset_items di ON di.entity_id = ob.id AND di.entity_type = 'org_business_unit' WHERE di.dataset_id = $1",
             vec!["id","bu_name","branch_id","branch_code","currency","status"]),
            ("org_department", "entities_departments.csv",
             "SELECT od.* FROM org_departments od JOIN dataset_items di ON di.entity_id = od.id AND di.entity_type = 'org_department' WHERE di.dataset_id = $1",
             vec!["id","dept_name","bu_id","bu_name","branch_code","status"]),
            ("org_seller", "entities_sellers.csv",
             "SELECT os.* FROM org_sellers os JOIN dataset_items di ON di.entity_id = os.id AND di.entity_type = 'org_seller' WHERE di.dataset_id = $1",
             vec!["id","seller_code","first_name","last_name","email","bu_id","branch_code","status","seniority"]),
            ("org_treasury", "entities_treasuries.csv",
             "SELECT ot.* FROM org_treasuries ot JOIN dataset_items di ON di.entity_id = ot.id AND di.entity_type = 'org_treasury' WHERE di.dataset_id = $1",
             vec!["id","branch_id","branch_code","treasury_name","currency","status"]),
        ] {
            let rows = sqlx::query(query_sql).bind(&dataset_id).fetch_all(&state.pool).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let header = cols.join(",") + "\n";
            let mut out = header;
            for r in &rows {
                let line: Vec<String> = cols.iter().map(|c| {
                    r.try_get::<String,_>(*c).unwrap_or_default()
                }).collect();
                out.push_str(&(line.join(",") + "\n"));
            }
            zip.start_file(*filename, opts).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            zip.write_all(out.as_bytes()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let _ = entity_type; // suppress unused warning
        }

        zip.finish().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let safe_name = ds_name.replace(|c: char| !c.is_alphanumeric() && c != '-', "_");
    let filename  = format!("dataset_{safe_name}.zip");

    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/zip")
        .header("Content-Disposition", format!("attachment; filename=\"{filename}\""))
        .body(axum::body::Body::from(zip_buf))
        .unwrap())
}

// ── Universal CSV/ZIP ingest ──────────────────────────────────────────────────

fn detect_csv_type(headers: &[&str]) -> &'static str {
    let has = |k: &str| headers.contains(&k);
    if has("contract_id") || (has("contract_type") && has("notional")) { "contracts" }
    else if has("branch_id") && has("branch_name") { "branches" }
    else if has("bu_id") && has("bu_name") { "business_units" }
    else if has("dept_id") && has("dept_name") { "departments" }
    else if has("seller_id") && has("seller_code") { "sellers" }
    else if has("treasury_id") || has("treasury_name") { "treasuries" }
    else if has("tenors_json") && has("values_json") { "rate_curves" }
    else if has("series_name") && has("obs_date") { "rate_series" }
    else { "unknown" }
}

async fn import_csv_content(
    content: &str,
    dataset_id: &str,
    pool: &sqlx::PgPool,
) -> (String, usize) {
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let headers: Vec<String> = match rdr.headers() {
        Ok(h) => h.iter().map(|s| s.to_string()).collect(),
        Err(_) => return ("unknown".into(), 0),
    };
    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    let csv_type = detect_csv_type(&header_refs);
    let mut count = 0usize;

    for rec in rdr.records().flatten() {
        let row: std::collections::HashMap<&str, &str> = headers.iter()
            .zip(rec.iter()).map(|(h, v)| (h.as_str(), v)).collect();
        let opt = |k: &str| -> Option<&str> { row.get(k).copied().filter(|s| !s.is_empty()) };

        match csv_type {
            "contracts" => {
                let id = opt("id").map(|s| s.to_string()).unwrap_or_else(|| Uuid::new_v4().to_string());
                let cid = opt("contract_id").unwrap_or(""); if cid.is_empty() { continue; }
                let notional: f64 = row.get("notional").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let irate: Option<f64> = row.get("interest_rate").and_then(|v| v.parse().ok());
                let tenor: Option<i32> = row.get("tenor_months").and_then(|v| v.parse().ok());
                let prepay = row.get("prepayment_allowed").map(|v| v.to_lowercase()=="true").unwrap_or(false);
                let pen: f64 = row.get("prepayment_penalty").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let rw: f64 = row.get("risk_weight").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let r = sqlx::query(r#"INSERT INTO contracts (id,contract_id,contract_type,side,seller_id,branch_code,currency,rating,notional,rate_type,interest_rate,settlement_date,maturity_date,tenor_months,payment_frequency,day_count,business_day_convention,amortization_type,prepayment_allowed,prepayment_penalty,guarantee_type,profiles_json,rates_json,risk_weight,source_dataset_id) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12::DATE,$13::DATE,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25) ON CONFLICT (contract_id) DO NOTHING"#)
                    .bind(&id).bind(cid)
                    .bind(opt("contract_type").unwrap_or("UNKNOWN")).bind(opt("side").unwrap_or("ASSET"))
                    .bind(opt("seller_id")).bind(opt("branch_code"))
                    .bind(opt("currency").unwrap_or("EUR")).bind(opt("rating"))
                    .bind(notional).bind(opt("rate_type")).bind(irate)
                    .bind(opt("settlement_date")).bind(opt("maturity_date")).bind(tenor)
                    .bind(opt("payment_frequency")).bind(opt("day_count"))
                    .bind(opt("business_day_convention")).bind(opt("amortization_type"))
                    .bind(prepay).bind(pen).bind(opt("guarantee_type"))
                    .bind(opt("profiles_json")).bind(opt("rates_json")).bind(rw).bind(dataset_id)
                    .execute(pool).await;
                if r.is_ok() {
                    let _ = sqlx::query("INSERT INTO dataset_items (dataset_id,entity_type,entity_id) VALUES ($1,'contract',$2) ON CONFLICT DO NOTHING")
                        .bind(dataset_id).bind(&id).execute(pool).await;
                    count += 1;
                }
            }
            "branches" => {
                let id = opt("branch_id").or(opt("id")).unwrap_or(""); if id.is_empty() { continue; }
                let r = sqlx::query("INSERT INTO org_branches (id,branch_code,branch_name,country,currency,city,status,created_date) VALUES ($1,$2,$3,$4,$5,$6,$7,$8::DATE) ON CONFLICT (branch_code) DO NOTHING")
                    .bind(id).bind(opt("branch_code").unwrap_or(id))
                    .bind(opt("branch_name")).bind(opt("country"))
                    .bind(opt("currency").unwrap_or("EUR")).bind(opt("city"))
                    .bind(opt("status").unwrap_or("active")).bind(opt("created_date"))
                    .execute(pool).await;
                if r.is_ok() {
                    let _ = sqlx::query("INSERT INTO dataset_items (dataset_id,entity_type,entity_id) VALUES ($1,'org_branch',$2) ON CONFLICT DO NOTHING")
                        .bind(dataset_id).bind(id).execute(pool).await;
                    count += 1;
                }
            }
            "business_units" => {
                let id = opt("bu_id").or(opt("id")).unwrap_or(""); if id.is_empty() { continue; }
                let r = sqlx::query("INSERT INTO org_business_units (id,bu_name,branch_id,branch_code,currency,status) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (id) DO NOTHING")
                    .bind(id).bind(opt("bu_name").unwrap_or(""))
                    .bind(opt("branch_id")).bind(opt("branch_code").unwrap_or(""))
                    .bind(opt("currency").unwrap_or("EUR")).bind(opt("status").unwrap_or("active"))
                    .execute(pool).await;
                if r.is_ok() {
                    let _ = sqlx::query("INSERT INTO dataset_items (dataset_id,entity_type,entity_id) VALUES ($1,'org_business_unit',$2) ON CONFLICT DO NOTHING")
                        .bind(dataset_id).bind(id).execute(pool).await;
                    count += 1;
                }
            }
            "departments" => {
                let id = opt("dept_id").or(opt("id")).unwrap_or(""); if id.is_empty() { continue; }
                let r = sqlx::query("INSERT INTO org_departments (id,dept_name,bu_id,bu_name,branch_id,branch_code,status) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (id) DO NOTHING")
                    .bind(id).bind(opt("dept_name").unwrap_or(""))
                    .bind(opt("bu_id")).bind(opt("bu_name").unwrap_or(""))
                    .bind(opt("branch_id")).bind(opt("branch_code").unwrap_or(""))
                    .bind(opt("status").unwrap_or("active"))
                    .execute(pool).await;
                if r.is_ok() {
                    let _ = sqlx::query("INSERT INTO dataset_items (dataset_id,entity_type,entity_id) VALUES ($1,'org_department',$2) ON CONFLICT DO NOTHING")
                        .bind(dataset_id).bind(id).execute(pool).await;
                    count += 1;
                }
            }
            "sellers" => {
                let id = opt("seller_id").or(opt("id")).unwrap_or(""); if id.is_empty() { continue; }
                let tv: Option<f64> = row.get("target_volume").and_then(|v| v.parse().ok());
                let r = sqlx::query("INSERT INTO org_sellers (id,seller_code,first_name,last_name,email,bu_id,bu_name,branch_id,branch_code,hire_date,status,target_volume,seniority) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10::DATE,$11,$12,$13) ON CONFLICT (id) DO NOTHING")
                    .bind(id).bind(opt("seller_code").unwrap_or(""))
                    .bind(opt("first_name")).bind(opt("last_name")).bind(opt("email"))
                    .bind(opt("bu_id")).bind(opt("bu_name").unwrap_or(""))
                    .bind(opt("branch_id")).bind(opt("branch_code").unwrap_or(""))
                    .bind(opt("hire_date")).bind(opt("status").unwrap_or("active"))
                    .bind(tv).bind(opt("seniority"))
                    .execute(pool).await;
                if r.is_ok() {
                    let _ = sqlx::query("INSERT INTO dataset_items (dataset_id,entity_type,entity_id) VALUES ($1,'org_seller',$2) ON CONFLICT DO NOTHING")
                        .bind(dataset_id).bind(id).execute(pool).await;
                    count += 1;
                }
            }
            "treasuries" => {
                let id = opt("treasury_id").or(opt("id")).unwrap_or(""); if id.is_empty() { continue; }
                let r = sqlx::query("INSERT INTO org_treasuries (id,branch_id,branch_code,treasury_name,currency,status) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (id) DO NOTHING")
                    .bind(id).bind(opt("branch_id")).bind(opt("branch_code").unwrap_or(""))
                    .bind(opt("treasury_name").unwrap_or("")).bind(opt("currency").unwrap_or("EUR"))
                    .bind(opt("status").unwrap_or("active"))
                    .execute(pool).await;
                if r.is_ok() {
                    let _ = sqlx::query("INSERT INTO dataset_items (dataset_id,entity_type,entity_id) VALUES ($1,'org_treasury',$2) ON CONFLICT DO NOTHING")
                        .bind(dataset_id).bind(id).execute(pool).await;
                    count += 1;
                }
            }
            "rate_curves" => {
                let id = opt("id").unwrap_or(""); if id.is_empty() { continue; }
                let r = sqlx::query("INSERT INTO rate_curves (id,name,component,currency,version,status,tenors_json,values_json,source) VALUES ($1,$2,$3,$4,1,'approved',$5,$6,$7) ON CONFLICT (id) DO NOTHING")
                    .bind(id).bind(opt("name").unwrap_or(id))
                    .bind(opt("component").unwrap_or("base_rate"))
                    .bind(opt("currency").unwrap_or("EUR"))
                    .bind(opt("tenors_json").unwrap_or("[]"))
                    .bind(opt("values_json").unwrap_or("[]"))
                    .bind(opt("source"))
                    .execute(pool).await;
                if r.is_ok() {
                    let _ = sqlx::query("INSERT INTO dataset_items (dataset_id,entity_type,entity_id) VALUES ($1,'rate_curve',$2) ON CONFLICT DO NOTHING")
                        .bind(dataset_id).bind(id).execute(pool).await;
                    count += 1;
                }
            }
            "rate_series" => {
                let series = opt("series_name").unwrap_or(""); if series.is_empty() { continue; }
                let obs = opt("obs_date").unwrap_or(""); if obs.is_empty() { continue; }
                let rate: f64 = row.get("rate").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let tenor_val = opt("tenor");
                let id = format!("{series}-{obs}-{}", tenor_val.unwrap_or("ON"));
                let ccy = match series { "ESTR"|"EURIBOR" => "EUR", "SOFR" => "USD", "SONIA" => "GBP", _ => "EUR" };
                let _ = sqlx::query("INSERT INTO rate_series_data (id,series_name,component,currency,obs_date,tenor,rate,source) VALUES ($1,$2,'base_rate',$3,$4::DATE,$5,$6,'uploaded') ON CONFLICT DO NOTHING")
                    .bind(&id).bind(series).bind(ccy).bind(obs).bind(tenor_val).bind(rate)
                    .execute(pool).await;
                // rate_series_data are not linked per-dataset (too many rows) — just insert globally
                count += 1;
            }
            _ => {}
        }
    }
    (csv_type.to_string(), count)
}

pub async fn ingest_files(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    use std::io::Read;

    let mut file_bytes: Vec<u8> = Vec::new();
    let mut file_name = String::new();
    let mut dataset_name = String::from("Upload");
    let mut dataset_id_param: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let fname = field.name().unwrap_or("").to_string();
        match fname.as_str() {
            "file" => {
                file_name = field.file_name().unwrap_or("file").to_string();
                file_bytes = field.bytes().await.unwrap_or_default().to_vec();
            }
            "dataset_name" => {
                dataset_name = String::from_utf8(field.bytes().await.unwrap_or_default().to_vec()).unwrap_or_default();
            }
            "dataset_id" => {
                let v = String::from_utf8(field.bytes().await.unwrap_or_default().to_vec()).unwrap_or_default();
                if !v.is_empty() { dataset_id_param = Some(v); }
            }
            _ => {}
        }
    }

    if file_bytes.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "no file received"})));
    }

    // Ensure dataset exists
    let dataset_id = match dataset_id_param {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            let _ = sqlx::query("INSERT INTO datasets (id,name,source) VALUES ($1,$2,'uploaded')")
                .bind(&id).bind(&dataset_name).execute(&state.pool).await;
            id
        }
    };

    let mut summary: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    let is_zip = file_name.ends_with(".zip")
        || (file_bytes.len() >= 4 && file_bytes[..4] == [0x50, 0x4B, 0x03, 0x04]);

    if is_zip {
        // Extract all CSV contents first (sync), then process (async) to avoid !Send across await
        use std::io::Read;
        let csv_contents: Vec<String> = {
            let cursor = std::io::Cursor::new(&file_bytes);
            match zip::ZipArchive::new(cursor) {
                Ok(mut archive) => {
                    let mut out = Vec::new();
                    for i in 0..archive.len() {
                        if let Ok(mut entry) = archive.by_index(i) {
                            if entry.name().ends_with(".csv") {
                                let mut content = String::new();
                                if entry.read_to_string(&mut content).is_ok() {
                                    out.push(content);
                                }
                            }
                        }
                    }
                    out
                }
                Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("invalid ZIP: {e}")}))),
            }
        };
        for content in csv_contents {
            let (csv_type, count) = import_csv_content(&content, &dataset_id, &state.pool).await;
            *summary.entry(csv_type).or_insert(0) += count;
        }
    } else {
        // Single CSV file
        let content = match String::from_utf8(file_bytes) {
            Ok(s) => s,
            Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "file is not valid UTF-8"}))),
        };
        let (csv_type, count) = import_csv_content(&content, &dataset_id, &state.pool).await;
        summary.insert(csv_type, count);
    }

    (StatusCode::OK, Json(json!({
        "dataset_id": dataset_id,
        "imported": summary,
    })))
}

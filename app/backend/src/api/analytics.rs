/// GET /api/analytics/portfolio-nim?portfolio_id=X
///
/// Returns NIM breakdown for each position of a portfolio using the most recent
/// completed execution for that portfolio.
///
/// Response:
/// {
///   "portfolio_id": "...",
///   "execution_id": "...",
///   "method": "...",
///   "positions": [
///     { "index": 0, "branch": "Nord", "product_type": "mortgage", "seller": "...",
///       "outstanding": 1000000, "client_rate": 0.04, "ftp_rate": 0.03, "nim": 0.01 }
///   ],
///   "heatmap": {
///     "by_branch":  { "Nord": { "nim": 0.012, "outstanding": 1e6 }, ... },
///     "by_product": { "mortgage": { ... }, ... },
///     "by_seller":  { "Smith": { ... }, ... }
///   }
/// }
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

use crate::db::AppState;

#[derive(Deserialize)]
pub struct NimQuery {
    pub portfolio_id: String,
    pub execution_id: Option<String>,
}

#[derive(Serialize)]
pub struct PositionNim {
    pub index: usize,
    pub branch: Option<String>,
    pub product_type: String,
    pub seller: Option<String>,
    pub outstanding: f64,
    pub client_rate: Option<f64>,
    pub ftp_rate: Option<f64>,
    pub nim: Option<f64>,
}

#[derive(Serialize)]
pub struct BucketStats {
    pub outstanding: f64,
    pub count: usize,
    pub avg_client_rate: Option<f64>,
    pub avg_ftp_rate: Option<f64>,
    pub avg_nim: Option<f64>,
    pub total_nim_income: f64, // nim × outstanding (monthly, annualised ÷ 12 to keep monthly)
}

#[derive(Serialize)]
pub struct NimResponse {
    pub portfolio_id: String,
    pub execution_id: Option<String>,
    pub method: Option<String>,
    pub positions: Vec<PositionNim>,
    pub by_branch:  HashMap<String, BucketStats>,
    pub by_product: HashMap<String, BucketStats>,
    pub by_seller:  HashMap<String, BucketStats>,
}

fn aggregate(positions: &[PositionNim]) -> BucketStats {
    let count = positions.len();
    if count == 0 {
        return BucketStats {
            outstanding: 0.0, count: 0,
            avg_client_rate: None, avg_ftp_rate: None, avg_nim: None,
            total_nim_income: 0.0,
        };
    }
    let total_out: f64 = positions.iter().map(|p| p.outstanding).sum();
    let weighted_avg = |f: fn(&PositionNim) -> Option<f64>| {
        let num: f64 = positions.iter()
            .filter_map(|p| f(p).map(|v| v * p.outstanding))
            .sum();
        if total_out > 0.0 { Some(num / total_out) } else { None }
    };
    let total_nim_income: f64 = positions.iter()
        .filter_map(|p| p.nim.map(|n| n * p.outstanding / 12.0))
        .sum();
    BucketStats {
        outstanding: total_out,
        count,
        avg_client_rate: weighted_avg(|p| p.client_rate),
        avg_ftp_rate:    weighted_avg(|p| p.ftp_rate),
        avg_nim:         weighted_avg(|p| p.nim),
        total_nim_income,
    }
}

pub async fn portfolio_nim(
    State(state): State<AppState>,
    Query(q): Query<NimQuery>,
) -> Result<Json<NimResponse>, StatusCode> {
    let pool = &state.pool;

    // Load positions ordered by insertion
    let pos_rows = sqlx::query(
        r#"SELECT id, product_type, branch, seller, outstanding, client_rate
           FROM portfolio_positions WHERE portfolio_id = $1 ORDER BY id"#,
    )
    .bind(&q.portfolio_id)
    .fetch_all(pool).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Load last completed execution (or the specified one)
    let exec_row = if let Some(ref eid) = q.execution_id {
        sqlx::query(
            "SELECT id, method, result_json FROM executions WHERE id = $1 AND status = 'completed'"
        )
        .bind(eid)
        .fetch_optional(pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query(
            r#"SELECT id, method, result_json FROM executions
               WHERE portfolio_id = $1 AND status = 'completed'
               ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(&q.portfolio_id)
        .fetch_optional(pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    // Parse ftp_rate matrix from result
    let ftp_matrix: Option<Vec<Vec<f64>>> = exec_row.as_ref().and_then(|r| {
        let json: Option<String> = r.get("result_json");
        let json = json?;
        let v: Value = serde_json::from_str(&json).ok()?;
        let arr = v.get("ftp_rate")?;
        serde_json::from_value(arr.clone()).ok()
    });

    let exec_id: Option<String> = exec_row.as_ref().map(|r| r.get("id"));
    let exec_method: Option<String> = exec_row.as_ref().map(|r| r.get("method"));

    // Build position list
    let positions: Vec<PositionNim> = pos_rows.iter().enumerate().map(|(i, r)| {
        let outstanding: f64 = r.get("outstanding");
        let client_rate: Option<f64> = r.get("client_rate");
        let ftp_rate: Option<f64> = ftp_matrix.as_ref()
            .and_then(|m| m.get(i))
            .and_then(|row| row.first())
            .copied();
        let nim: Option<f64> = client_rate.zip(ftp_rate).map(|(c, f)| c - f);
        PositionNim {
            index: i,
            branch:       r.get("branch"),
            product_type: r.get("product_type"),
            seller:       r.get("seller"),
            outstanding,
            client_rate,
            ftp_rate,
            nim,
        }
    }).collect();

    // Aggregate
    let mut by_branch:  HashMap<String, BucketStats> = HashMap::new();
    let mut by_product: HashMap<String, BucketStats> = HashMap::new();
    let mut by_seller:  HashMap<String, BucketStats> = HashMap::new();

    let group = |positions: &Vec<PositionNim>, key_fn: fn(&PositionNim) -> String| {
        let mut map: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, p) in positions.iter().enumerate() {
            map.entry(key_fn(p)).or_default().push(i);
        }
        map
    };

    for (key, idxs) in group(&positions, |p| p.branch.clone().unwrap_or_else(|| "N/A".into())) {
        let subset: Vec<&PositionNim> = idxs.iter().map(|&i| &positions[i]).collect();
        let owned: Vec<PositionNim> = subset.into_iter().map(|p| PositionNim {
            index: p.index, branch: p.branch.clone(), product_type: p.product_type.clone(),
            seller: p.seller.clone(), outstanding: p.outstanding,
            client_rate: p.client_rate, ftp_rate: p.ftp_rate, nim: p.nim,
        }).collect();
        by_branch.insert(key, aggregate(&owned));
    }
    for (key, idxs) in group(&positions, |p| p.product_type.clone()) {
        let owned: Vec<PositionNim> = idxs.iter().map(|&i| PositionNim {
            index: positions[i].index, branch: positions[i].branch.clone(),
            product_type: positions[i].product_type.clone(), seller: positions[i].seller.clone(),
            outstanding: positions[i].outstanding, client_rate: positions[i].client_rate,
            ftp_rate: positions[i].ftp_rate, nim: positions[i].nim,
        }).collect();
        by_product.insert(key, aggregate(&owned));
    }
    for (key, idxs) in group(&positions, |p| p.seller.clone().unwrap_or_else(|| "N/A".into())) {
        let owned: Vec<PositionNim> = idxs.iter().map(|&i| PositionNim {
            index: positions[i].index, branch: positions[i].branch.clone(),
            product_type: positions[i].product_type.clone(), seller: positions[i].seller.clone(),
            outstanding: positions[i].outstanding, client_rate: positions[i].client_rate,
            ftp_rate: positions[i].ftp_rate, nim: positions[i].nim,
        }).collect();
        by_seller.insert(key, aggregate(&owned));
    }

    Ok(Json(NimResponse {
        portfolio_id: q.portfolio_id,
        execution_id: exec_id,
        method: exec_method,
        positions,
        by_branch,
        by_product,
        by_seller,
    }))
}

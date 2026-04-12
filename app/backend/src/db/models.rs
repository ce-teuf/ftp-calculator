use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateCurve {
    pub id: String,
    pub name: String,
    pub component: String,
    pub currency: String,
    pub version: i32,
    pub status: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub tenors_json: String,
    pub values_json: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateSeries {
    pub id: String,
    pub name: String,
    pub component: String,
    pub frequency: String,
    pub dates_json: String,
    pub values_json: String,
    pub tenor: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunoffModel {
    pub id: String,
    pub name: String,
    pub product_type: String,
    pub category: Option<String>,
    pub version: i32,
    pub status: String,
    pub method: String,
    pub profile_json: String,
    pub parameters_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub status: String,
    pub as_of_date: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub id: String,
    pub portfolio_id: String,
    pub position_ref: Option<String>,
    pub product_type: String,
    pub branch: Option<String>,
    pub seller: Option<String>,
    pub currency: String,
    pub outstanding: f64,
    pub origination_date: Option<String>,
    pub maturity_date: Option<String>,
    pub client_rate: Option<f64>,
    pub runoff_model_id: Option<String>,
    pub risk_weight: f64,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: String,
    pub label: Option<String>,
    pub method: String,
    pub portfolio_id: String,
    pub curve_ids_json: String,
    pub runoff_ids_json: Option<String>,
    pub parameters_json: String,
    pub seeds_json: Option<String>,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlcoApproval {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub by_user: String,
    pub at: String,
    pub comment: Option<String>,
}

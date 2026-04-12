use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeedManager {
    monte_carlo_rates: Option<u64>,
    prepayment_cpr: Option<u64>,
    nmd_behavioral: Option<u64>,
}

impl SeedManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn monte_carlo(&self) -> Option<u64> {
        self.monte_carlo_rates
    }

    pub fn prepayment(&self) -> Option<u64> {
        self.prepayment_cpr
    }

    pub fn nmd_behavioral(&self) -> Option<u64> {
        self.nmd_behavioral
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeInput {
    pub method: String,
    pub outstanding: Vec<f64>,
    pub profiles: Vec<Vec<f64>>,
    pub rates: Vec<Vec<f64>>,
    pub seeds: Option<SeedConfig>,
    pub parameters: ComputeParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeParameters {
    pub capital_ratio: Option<f64>,
    pub coe: Option<f64>,
    pub hurdle: Option<f64>,
    pub market_rate: Option<f64>,
    pub input_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedConfig {
    pub monte_carlo_rates: Option<u64>,
    pub prepayment_cpr: Option<u64>,
    pub nmd_behavioral: Option<u64>,
}

pub struct Runner;

impl Runner {
    pub fn run(input: ComputeInput) -> Result<ComputeOutput, String> {
        Ok(ComputeOutput {
            stock_amort: vec![],
            ftp_rate: vec![],
            ftp_int: vec![],
            market_rate: vec![],
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeOutput {
    pub stock_amort: Vec<f64>,
    pub ftp_rate: Vec<f64>,
    pub ftp_int: Vec<f64>,
    pub market_rate: Vec<f64>,
}

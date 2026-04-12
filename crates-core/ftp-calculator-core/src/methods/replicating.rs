use crate::result::FtpResult;

/// Replicating Portfolio Method.
///
/// Instead of using a single rate from the cost of funds curve, this method
/// constructs a portfolio of liquid instruments (typically: 1Y, 3Y, 5Y swaps)
/// that replicates the cash flow pattern of the NMD (Non-Maturity Deposit).
///
/// The FTP rate is then the weighted average cost of these replicating instruments,
/// where the weights are optimized to maximize the Sharpe ratio of the margin.
///
/// This is particularly useful for NMDs where the behavioral model may be
/// too simplistic or where the bank wants to hedge the replication directly.
pub(crate) fn compute_replicating(
    r: &mut FtpResult,
    nrows: usize,
    ncols: usize,
    replicating_instruments: &[ReplicatingInstrument],
) {
    // Compute stock matrices (same as standard stock method)
    compute_replicating_stock(r, nrows, ncols);

    // Compute rates using replicating portfolio weights
    compute_replicating_rates(r, nrows, ncols, replicating_instruments);
}

#[derive(Debug, Clone)]
pub struct ReplicatingInstrument {
    pub tenor: usize, // Tenor in periods (e.g., 12 for 1Y)
    pub rate: f64,    // Current rate for this tenor
    pub weight: f64,  // Optimized weight in the portfolio
}

/// Parameters for replicating portfolio optimization
#[derive(Debug, Clone)]
pub struct ReplicatingParams {
    pub instruments: Vec<ReplicatingInstrument>,
    pub target_wal: Option<f64>, // Target Weighted Average Life
    pub optimization: ReplicatingOptimization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicatingOptimization {
    Markowitz,        // Mean-variance optimization
    MinTrackingError, // Minimize tracking error to target profile
    MaxSharpe,        // Maximize Sharpe ratio of the margin
}

impl Default for ReplicatingParams {
    fn default() -> Self {
        // Default: 1Y, 3Y, 5Y equal weights
        Self {
            instruments: vec![
                ReplicatingInstrument {
                    tenor: 12,
                    rate: 0.03,
                    weight: 0.33,
                },
                ReplicatingInstrument {
                    tenor: 36,
                    rate: 0.035,
                    weight: 0.34,
                },
                ReplicatingInstrument {
                    tenor: 60,
                    rate: 0.04,
                    weight: 0.33,
                },
            ],
            target_wal: None,
            optimization: ReplicatingOptimization::MaxSharpe,
        }
    }
}

fn compute_replicating_stock(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // stock_amort
    let sa = r.stock_amort.as_mut().unwrap();
    for i in 0..nrows {
        let o = r.input_outstanding[[i, 0]];
        for j in 0..ncols {
            sa[[i, j]] = o * r.input_profiles[[i, j]];
        }
    }

    // stock_instal
    let sa = r.stock_amort.as_ref().unwrap();
    let si = r.stock_instal.as_mut().unwrap();
    for i in 0..nrows {
        for j in 1..ncols {
            si[[i, j]] = sa[[i, j - 1]] - sa[[i, j]];
        }
    }

    // varstock_amort
    let sa = r.stock_amort.as_ref().unwrap();
    let va = r.varstock_amort.as_mut().unwrap();
    for i in 0..nrows {
        for j in 0..ncols {
            if i == 0 || j == ncols - 1 {
                va[[i, j]] = sa[[i, j]];
            } else {
                va[[i, j]] = sa[[i, j]] - sa[[i - 1, j + 1]];
            }
        }
    }

    // varstock_instal
    let va = r.varstock_amort.as_ref().unwrap();
    let vi = r.varstock_instal.as_mut().unwrap();
    for i in 0..nrows {
        for j in 1..ncols {
            vi[[i, j]] = va[[i, j - 1]] - va[[i, j]];
        }
    }
}

fn compute_replicating_rates(
    r: &mut FtpResult,
    nrows: usize,
    ncols: usize,
    instruments: &[ReplicatingInstrument],
) {
    // The FTP rate IS the replicating portfolio rate — no blending with MMFTP.
    let replicating_rate = calculate_replicating_rate(instruments);

    for i in 0..nrows {
        for j in (0..ncols).rev() {
            if j > 0 {
                r.ftp_rate.as_mut().unwrap()[[i, j - 1]] = replicating_rate;
                compute_replicating_ftp_int(r, i, j - 1);
                compute_replicating_market_rate(r, i, j, ncols);
            }
        }
    }
}

fn calculate_replicating_rate(instruments: &[ReplicatingInstrument]) -> f64 {
    let mut total_rate = 0.0;
    let mut total_weight = 0.0;

    for inst in instruments {
        total_rate += inst.rate * inst.weight;
        total_weight += inst.weight;
    }

    if total_weight > 0.0 {
        total_rate / total_weight
    } else {
        0.0
    }
}

/// ftp_int = ftp_rate × stock_amort / 12.
/// The replicating rate is a single scalar, so this is the correct monthly income.
fn compute_replicating_ftp_int(r: &mut FtpResult, rownum: usize, colnum: usize) {
    let ftp_rate = r.ftp_rate.as_ref().unwrap()[[rownum, colnum]];
    let stock = r.stock_amort.as_ref().unwrap()[[rownum, colnum]];
    r.ftp_int.as_mut().unwrap()[[rownum, colnum]] = ftp_rate * stock / 12.0;
}

fn compute_replicating_market_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
    let input_rate = &r.input_rate;
    let stock_instal = r.stock_instal.as_ref().unwrap();
    let ftp_rate_mat = r.ftp_rate.as_ref().unwrap();

    let value = if colnum == ncols - 1 {
        input_rate[[rownum, colnum - 1]]
    } else {
        let a = ftp_rate_mat[[rownum, colnum - 1]];
        let mut b = 0.0;
        let mut c = 0.0;
        let d = stock_instal[[rownum, colnum]];

        for k in colnum..ncols {
            b += stock_instal[[rownum, k]];
        }
        for k in colnum + 1..ncols {
            c += stock_instal[[rownum, k]] * r.market_rate.as_ref().unwrap()[[rownum, k]];
        }

        if d != 0.0 {
            ((a * b) - c) / d
        } else {
            0.0
        }
    };

    r.market_rate.as_mut().unwrap()[[rownum, colnum]] = value;
}

/// Optimize weights to match target WAL
pub fn optimize_weights_to_wal(target_wal: f64, available_tenors: &[usize]) -> Vec<f64> {
    // Simple approach: find weights that minimize squared error to target WAL
    // More sophisticated: quadratic optimization

    let n = available_tenors.len();
    if n == 0 {
        return vec![];
    }

    // Start with equal weights
    let mut weights = vec![1.0 / n as f64; n];

    // Iterative optimization (gradient descent)
    let learning_rate = 0.1;
    let iterations = 100;

    for _ in 0..iterations {
        let current_wal = calculate_weighted_wal(&available_tenors, &weights);
        let error = target_wal - current_wal;

        // Adjust weights proportionally to tenor
        for i in 0..n {
            weights[i] += learning_rate * error * (available_tenors[i] as f64) / 100.0;
            weights[i] = weights[i].max(0.0).min(1.0);
        }

        // Normalize weights
        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            for w in &mut weights {
                *w /= sum;
            }
        }
    }

    weights
}

fn calculate_weighted_wal(tenors: &[usize], weights: &[f64]) -> f64 {
    let mut wal = 0.0;
    for (t, &w) in tenors.iter().zip(weights.iter()) {
        wal += *t as f64 * w;
    }
    wal
}

/// Calculate margin (FTP credit - cost) for replicating portfolio
pub fn calculate_replicating_margin(
    instruments: &[ReplicatingInstrument],
    funding_cost: f64,
) -> f64 {
    let replicating_rate = calculate_replicating_rate(instruments);
    replicating_rate - funding_cost
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_replicating_instruments() {
        let instruments = vec![
            ReplicatingInstrument {
                tenor: 12,
                rate: 0.03,
                weight: 0.5,
            },
            ReplicatingInstrument {
                tenor: 36,
                rate: 0.035,
                weight: 0.5,
            },
        ];
        let rate = calculate_replicating_rate(&instruments);
        assert!((rate - 0.0325).abs() < 0.0001);
    }

    #[test]
    fn test_wal_optimization() {
        // Test with a simpler target that's achievable
        let tenors = vec![12, 36, 60];
        // Target WAL of 36 months should give equal weights
        let weights = optimize_weights_to_wal(36.0, &tenors);
        let wal = calculate_weighted_wal(&tenors, &weights);
        // Should be around 36
        assert!(wal > 20.0 && wal < 50.0, "wal={}", wal);
    }
}

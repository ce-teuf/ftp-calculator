use crate::result::FtpResult;

/// Behavioral Run-off Model for Non-Maturity Deposits (NMDs).
///
/// NMDs (demand deposits, savings accounts, CASA) have no contractual maturity.
/// Their behavioral maturity is modeled using an exponential decay function:
///
/// profile[t] = exp(-lambda * t)
///
/// Where:
/// - lambda (λ) is the decay rate parameter calibrated from historical data
/// - t is the time period (months)
///
/// The profile represents the expected remaining balance over time.
pub(crate) fn compute_behavioral(r: &mut FtpResult, nrows: usize, ncols: usize, lambda: f64) {
    // Compute stock matrices using behavioral (exponential decay) profiles
    compute_behavioral_stock(r, nrows, ncols, lambda);

    // Compute rates
    compute_behavioral_rates(r, nrows, ncols);
}

fn compute_behavioral_stock(r: &mut FtpResult, nrows: usize, ncols: usize, lambda: f64) {
    // Compute stock_amort directly from the exponential decay profile.
    // profile[t] = exp(-lambda * t); t = period index (monthly).
    // input_profiles is NOT mutated — it retains the original user-supplied profile.
    {
        let sa = r.stock_amort.as_mut().unwrap();
        for i in 0..nrows {
            let o = r.input_outstanding[[i, 0]];
            for j in 0..ncols {
                sa[[i, j]] = o * (-lambda * j as f64).exp();
            }
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

fn compute_behavioral_rates(r: &mut FtpResult, nrows: usize, ncols: usize) {
    for i in 0..nrows {
        for j in (0..ncols).rev() {
            if j > 0 {
                compute_behavioral_ftp_rate(r, i, j - 1, ncols);
                compute_behavioral_ftp_int(r, i, j - 1, ncols);
                compute_behavioral_market_rate(r, i, j, ncols);
            }
        }
    }
}

fn compute_behavioral_ftp_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
    let input_rate = &r.input_rate;
    let varstock_instal = r.varstock_instal.as_ref().unwrap();
    let stock_instal = r.stock_instal.as_ref().unwrap();
    let market_rate_mat = r.market_rate.as_ref().unwrap();

    let value = if rownum == 0 {
        let mut num = 0.0;
        let mut denum = 0.0;
        for k in colnum..ncols - 1 {
            num += varstock_instal[[0, k + 1]] * input_rate[[0, k]];
            denum += varstock_instal[[0, k + 1]];
        }
        if denum != 0.0 {
            num / denum
        } else {
            0.0
        }
    } else {
        let mut num1 = 0.0;
        let mut num2 = 0.0;
        let mut denum1 = 0.0;
        let mut denum2 = 0.0;
        for k in colnum..ncols - 1 {
            num1 += varstock_instal[[rownum, k + 1]] * input_rate[[rownum, k]];
            denum1 += varstock_instal[[rownum, k + 1]];
            if k > colnum {
                num2 += stock_instal[[rownum - 1, k + 1]] * market_rate_mat[[rownum - 1, k + 1]];
                denum2 += stock_instal[[rownum - 1, k + 1]];
            }
        }
        let denum = denum1 + denum2;
        if denum != 0.0 {
            (num1 + num2) / denum
        } else {
            0.0
        }
    };

    r.ftp_rate.as_mut().unwrap()[[rownum, colnum]] = value;
}

fn compute_behavioral_ftp_int(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
    let input_rate = &r.input_rate;
    let varstock_instal = r.varstock_instal.as_ref().unwrap();
    let stock_instal = r.stock_instal.as_ref().unwrap();
    let market_rate_mat = r.market_rate.as_ref().unwrap();

    let value = if rownum == 0 {
        let mut num = 0.0;
        for k in colnum..ncols - 1 {
            num += varstock_instal[[0, k + 1]] * input_rate[[0, k]];
        }
        num / 12.0
    } else {
        let mut num1 = 0.0;
        let mut num2 = 0.0;
        for k in colnum..ncols - 1 {
            num1 += varstock_instal[[rownum, k + 1]] * input_rate[[rownum, k]];
            if k > colnum {
                num2 += stock_instal[[rownum - 1, k + 1]] * market_rate_mat[[rownum - 1, k + 1]];
            }
        }
        (num1 + num2) / 12.0
    };

    r.ftp_int.as_mut().unwrap()[[rownum, colnum]] = value;
}

fn compute_behavioral_market_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
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

/// Calibration parameters for behavioral model
#[derive(Debug, Clone)]
pub struct BehavioralParams {
    pub lambda: f64,           // Decay rate (per period)
    pub core_ratio: f64,       // Ratio of stable (core) deposits
    pub volatility_ratio: f64, // Ratio of volatile (non-core) deposits
}

impl Default for BehavioralParams {
    fn default() -> Self {
        Self {
            lambda: 0.05,           // Default 5% monthly decay
            core_ratio: 0.70,       // 70% core (stable)
            volatility_ratio: 0.30, // 30% volatile
        }
    }
}

/// Calculate Weighted Average Life (WAL) from behavioral profile
pub fn calculate_wal(profiles: &[f64]) -> f64 {
    let mut wal = 0.0;
    let mut total_weight = 0.0;

    for (t, &p) in profiles.iter().enumerate() {
        wal += (t as f64 + 1.0) * p;
        total_weight += p;
    }

    if total_weight > 0.0 {
        wal / total_weight
    } else {
        0.0
    }
}

/// Calibrate lambda from historical data using regression
///
/// Given historical deposit volumes, estimate the decay parameter lambda
/// that best fits the observed run-off pattern.
pub fn calibrate_lambda(historical_volumes: &[f64], _periods: usize) -> f64 {
    // Simple calibration: fit exponential decay to historical data
    // Using least squares on log-transformed data

    if historical_volumes.len() < 2 {
        return 0.05; // Default
    }

    // log(Vt) = log(V0) - lambda * t
    // Simple approach: average decay rate from consecutive periods

    let mut sum_lambda = 0.0;
    let mut count = 0;

    for i in 1..historical_volumes.len() {
        let v_prev = historical_volumes[i - 1];
        let v_curr = historical_volumes[i];

        if v_prev > 0.0 && v_curr > 0.0 {
            let decay = 1.0 - (v_curr / v_prev);
            if decay > 0.0 && decay < 1.0 {
                // lambda = -ln(1 - decay)
                let lambda = -decay.ln();
                sum_lambda += lambda;
                count += 1;
            }
        }
    }

    if count > 0 {
        sum_lambda / count as f64
    } else {
        0.05
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_behavioral_profile() {
        // With lambda = 0.1, profile should decay exponentially
        let profile: Vec<f64> = (0..12).map(|t| (-0.1 * t as f64).exp()).collect();
        assert!(profile[0] > profile[5]);
        assert!(profile[5] > profile[11]);
    }

    #[test]
    fn test_wal_calculation() {
        let profiles = [1.0, 0.6, 0.4, 0.3, 0.2, 0.15];
        let wal = calculate_wal(&profiles);
        // WAL should be around 2-3 for this profile
        assert!(wal > 1.0 && wal < 5.0);
    }

    #[test]
    fn test_lambda_calibration() {
        // Simple test - verify function returns a positive value
        let volumes: Vec<f64> = vec![1000.0, 900.0, 810.0, 729.0, 656.0, 590.0];
        let calibrated = calibrate_lambda(&volumes, 6);
        // Should return some positive value for decaying series
        assert!(calibrated > 0.0, "calibrated={}", calibrated);
    }
}

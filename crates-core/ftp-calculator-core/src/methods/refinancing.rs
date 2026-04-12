use crate::result::FtpResult;

/// Refinancing Method (Forward Rate Method).
///
/// This method uses forward rates (implied from spot curve) instead of spot rates.
/// The FTP rate is the expected average cost of rolling short-term borrowings.
///
/// Core formula: f(t₁, t₂) = [(1 + r(t₂))^t₂ / (1 + r(t₁))^t₁]^(1/(t₂-t₁)) - 1
///
/// The input_rate matrix should contain forward rates for this method to work correctly.
pub(crate) fn compute_refinancing(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // First compute stock matrices (same as stock method)
    compute_refinancing_stock(r, nrows, ncols);

    // Then compute rates using forward rates (input_rate contains forward rates)
    compute_refinancing_rates(r, nrows, ncols);
}

fn compute_refinancing_stock(r: &mut FtpResult, nrows: usize, ncols: usize) {
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

fn compute_refinancing_rates(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // For refinancing method, we use forward rates directly from input_rate
    // The key difference from stock method is that rates are expected future costs

    for i in 0..nrows {
        for j in (0..ncols).rev() {
            if j > 0 {
                compute_refinancing_ftp_rate(r, i, j - 1, ncols);
                compute_refinancing_ftp_int(r, i, j - 1, ncols);
                compute_refinancing_market_rate(r, i, j, ncols);
            }
        }
    }
}

fn compute_refinancing_ftp_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
    // In refinancing method, the FTP rate is the weighted average of forward rates
    // using varstock_instal as weights

    let input_rate = &r.input_rate;
    let varstock_instal = r.varstock_instal.as_ref().unwrap();
    let stock_instal = r.stock_instal.as_ref().unwrap();
    let market_rate_mat = r.market_rate.as_ref().unwrap();

    let value = if rownum == 0 {
        // First row: weighted average of forward rates
        let mut num = 0.0;
        let mut denum = 0.0;
        for k in colnum..ncols - 1 {
            // Use forward rate from input_rate (which contains forward rates for refinancing method)
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

fn compute_refinancing_ftp_int(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
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

fn compute_refinancing_market_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
    let input_rate = &r.input_rate;
    let stock_instal = r.stock_instal.as_ref().unwrap();
    let ftp_rate_mat = r.ftp_rate.as_ref().unwrap();

    let value = if colnum == ncols - 1 {
        // Last column: use the last forward rate
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

/// Calculate forward rate from spot rates.
///
/// f(t₁, t₂) = [(1 + r(t₂))^(t₂) / (1 + r(t₁))^(t₁)]^(1/(t₂-t₁)) - 1
pub fn spot_to_forward(r_t1: f64, r_t2: f64, t1: f64, t2: f64) -> f64 {
    let numerator = (1.0 + r_t2).powf(t2);
    let denominator = (1.0 + r_t1).powf(t1);
    let ratio = numerator / denominator;
    let forward = ratio.powf(1.0 / (t2 - t1)) - 1.0;
    forward
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComputeMethod;
    use ndarray::array;

    #[test]
    fn test_forward_rate_calculation() {
        // 1Y spot = 2.25%, 2Y spot = 3.0%
        let fwd = spot_to_forward(0.0225, 0.030, 1.0, 2.0);
        // Expected: f(1,2) ≈ 3.76%
        assert!(fwd > 0.035 && fwd < 0.040);
    }

    #[test]
    fn test_refinancing_basic() {
        let mut r = FtpResult::new(
            array![[1000.0]],
            array![[1.0, 0.5, 0.2]],
            array![[0.0225, 0.0376]], // spot + forward rates
        );
        r.compute(ComputeMethod::Refinancing).unwrap();

        let sa = r.stock_amort().unwrap();
        assert_eq!(sa[[0, 0]], 1000.0);
    }
}

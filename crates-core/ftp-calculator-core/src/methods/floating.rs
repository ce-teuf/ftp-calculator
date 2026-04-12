use crate::result::FtpResult;

/// Floating-Rate Method with Double Profile.
///
/// For floating-rate products, there are TWO distinct profiles:
/// 1. **Interest Rate Profile**: determined by repricing frequency (e.g., 12M reset = 1Y profile)
/// 2. **Liquidity Profile**: determined by contractual maturity (e.g., 5Y loan = 5Y profile)
///
/// The FTP rate has two components:
/// - Interest rate component: from the short-term reference rate (matching repricing)
/// - Liquidity/term component: from the full contractual maturity
///
/// This is critical for products like floating-rate loans where the interest rate
/// resets frequently but the bank must provide funding for the full maturity.
pub(crate) fn compute_floating(
    r: &mut FtpResult,
    nrows: usize,
    ncols: usize,
    repricing_tenor: usize,
) {
    // Compute stock matrices
    compute_floating_stock(r, nrows, ncols);

    // Compute rates with double profile
    compute_floating_rates(r, nrows, ncols, repricing_tenor);
}

fn compute_floating_stock(r: &mut FtpResult, nrows: usize, ncols: usize) {
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

fn compute_floating_rates(r: &mut FtpResult, nrows: usize, ncols: usize, repricing_tenor: usize) {
    for i in 0..nrows {
        for j in (0..ncols).rev() {
            if j > 0 {
                compute_floating_ftp_rate(r, i, j - 1, ncols, repricing_tenor);
                // ftp_int must use the same effective rate as ftp_rate (with repricing split)
                compute_floating_ftp_int(r, i, j - 1);
                compute_floating_market_rate(r, i, j, ncols);
            }
        }
    }
}

fn compute_floating_ftp_rate(
    r: &mut FtpResult,
    rownum: usize,
    colnum: usize,
    ncols: usize,
    repricing_tenor: usize,
) {
    let input_rate = &r.input_rate;
    let varstock_instal = r.varstock_instal.as_ref().unwrap();
    let stock_instal = r.stock_instal.as_ref().unwrap();
    let market_rate_mat = r.market_rate.as_ref().unwrap();

    // For floating rate, we separate:
    // - Interest rate component: based on short-term repricing tenor
    // - Liquidity component: based on full tenor (from input_rate)

    // The input_rate matrix is assumed to contain:
    // - Columns for liquidity profile (full maturity)
    // - Additional data for repricing profile could be passed separately

    let value = if rownum == 0 {
        // For first row, compute weighted average with liquidity premium
        let mut num = 0.0;
        let mut denum = 0.0;

        // Split the calculation: short-term rate (interest) + term premium (liquidity)
        let short_term_rate = if repricing_tenor < ncols - 1 {
            input_rate[[0, repricing_tenor]]
        } else {
            input_rate[[0, 0]]
        };

        for k in colnum..ncols - 1 {
            let weight = varstock_instal[[0, k + 1]];
            // Use a blend: mostly short-term rate + liquidity premium from term
            let liquidity_premium = if k > repricing_tenor {
                input_rate[[0, k]] - short_term_rate
            } else {
                0.0
            };
            num += weight * (short_term_rate + liquidity_premium);
            denum += weight;
        }

        if denum != 0.0 {
            num / denum
        } else {
            short_term_rate
        }
    } else {
        let mut num1 = 0.0;
        let mut num2 = 0.0;
        let mut denum1 = 0.0;
        let mut denum2 = 0.0;

        let short_term_rate = if repricing_tenor < ncols - 1 {
            input_rate[[rownum, repricing_tenor]]
        } else {
            input_rate[[rownum, 0]]
        };

        for k in colnum..ncols - 1 {
            let weight = varstock_instal[[rownum, k + 1]];
            let liquidity_premium = if k > repricing_tenor {
                input_rate[[rownum, k]] - short_term_rate
            } else {
                0.0
            };
            num1 += weight * (short_term_rate + liquidity_premium);
            denum1 += weight;

            if k > colnum {
                num2 += stock_instal[[rownum - 1, k + 1]] * market_rate_mat[[rownum - 1, k + 1]];
                denum2 += stock_instal[[rownum - 1, k + 1]];
            }
        }

        let denum = denum1 + denum2;
        if denum != 0.0 {
            (num1 + num2) / denum
        } else {
            short_term_rate
        }
    };

    r.ftp_rate.as_mut().unwrap()[[rownum, colnum]] = value;
}

/// ftp_int = ftp_rate × stock_amort / 12.
/// Uses the effective floating rate (already computed in ftp_rate), not raw input_rate.
fn compute_floating_ftp_int(r: &mut FtpResult, rownum: usize, colnum: usize) {
    let ftp_rate = r.ftp_rate.as_ref().unwrap()[[rownum, colnum]];
    let stock = r.stock_amort.as_ref().unwrap()[[rownum, colnum]];
    r.ftp_int.as_mut().unwrap()[[rownum, colnum]] = ftp_rate * stock / 12.0;
}

fn compute_floating_market_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
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

/// Parameters for floating rate calculation
#[derive(Debug, Clone)]
pub struct FloatingRateParams {
    pub repricing_tenor: usize,      // e.g., 12 for 12-month reset
    pub reference_rate_tenor: usize, // e.g., 3 for 3M SOFR
}

impl Default for FloatingRateParams {
    fn default() -> Self {
        Self {
            repricing_tenor: 12,     // Default to 12M reset
            reference_rate_tenor: 3, // Default to 3M reference
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComputeMethod;
    use ndarray::array;

    #[test]
    fn test_floating_rate_basic() {
        let mut r = FtpResult::new(
            array![[1000.0]],
            array![[1.0, 0.5, 0.2]],
            array![[0.01, 0.02]],
        );
        r.compute(ComputeMethod::Floating).unwrap();

        let sa = r.stock_amort().unwrap();
        assert_eq!(sa[[0, 0]], 1000.0);
    }
}

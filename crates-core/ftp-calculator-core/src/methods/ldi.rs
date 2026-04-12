use crate::result::FtpResult;

/// LDI (Liability-Driven Investment) Method for Pension/Insurance portfolios.
///
/// This method is specifically designed for pension funds and insurance companies
/// where the liability profile drives the investment strategy. The FTP rate is
/// based on the duration-matched cost of hedging the liability cash flows.
///
/// Key characteristics:
/// - Horizon: Intergenerational (long-dated, often 20-30+ years)
/// - Risk measure: SCR (Solvency Capital Requirement) based
/// - Matching: Asset-liability matching (ALM) approach
///
/// For LDI, the FTP rate considers:
/// 1. Duration-matched funding cost from the yield curve
/// 2. Capital charge based on SCR
/// 3. Contingent liquidity for long-term commitment
pub(crate) fn compute_ldi(r: &mut FtpResult, nrows: usize, ncols: usize, ldi_params: &LdiParams) {
    compute_ldi_stock(r, nrows, ncols);
    compute_ldi_rates(r, nrows, ncols, ldi_params);
}

#[derive(Debug, Clone)]
pub struct LdiParams {
    pub scr_rate: f64,      // Solvency Capital Requirement as % of liabilities
    pub capital_ratio: f64, // Capital ratio for capital charge
    pub coe: f64,           // Cost of Equity (hurdle rate)
    pub loading_rate: f64,  // Additional loading for contingencies
}

impl Default for LdiParams {
    fn default() -> Self {
        Self {
            scr_rate: 0.10,      // 10% SCR typical for pension funds
            capital_ratio: 0.08, // 8% regulatory minimum
            coe: 0.10,           // 10% Cost of Equity
            loading_rate: 0.005, // 50 bps contingency loading
        }
    }
}

fn compute_ldi_stock(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // Same as standard stock method for stock_amort
    let sa = r.stock_amort.as_mut().unwrap();
    for i in 0..nrows {
        let o = r.input_outstanding[[i, 0]];
        for j in 0..ncols {
            sa[[i, j]] = o * r.input_profiles[[i, j]];
        }
    }

    let sa = r.stock_amort.as_ref().unwrap();
    let si = r.stock_instal.as_mut().unwrap();
    for i in 0..nrows {
        for j in 1..ncols {
            si[[i, j]] = sa[[i, j - 1]] - sa[[i, j]];
        }
    }

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

    let va = r.varstock_amort.as_ref().unwrap();
    let vi = r.varstock_instal.as_mut().unwrap();
    for i in 0..nrows {
        for j in 1..ncols {
            vi[[i, j]] = va[[i, j - 1]] - va[[i, j]];
        }
    }
}

fn compute_ldi_rates(r: &mut FtpResult, nrows: usize, ncols: usize, params: &LdiParams) {
    for i in 0..nrows {
        for j in (0..ncols).rev() {
            if j > 0 {
                compute_ldi_ftp_rate(r, i, j - 1, ncols, params);
                // ftp_int must use the LDI-adjusted rate, not raw input_rate
                compute_ldi_ftp_int(r, i, j - 1);
                compute_ldi_market_rate(r, i, j, ncols);
            }
        }
    }
}

fn compute_ldi_ftp_rate(
    r: &mut FtpResult,
    rownum: usize,
    colnum: usize,
    ncols: usize,
    params: &LdiParams,
) {
    let input_rate = &r.input_rate;
    let varstock_instal = r.varstock_instal.as_ref().unwrap();
    let stock_instal = r.stock_instal.as_ref().unwrap();
    let market_rate_mat = r.market_rate.as_ref().unwrap();

    // Standard rate calculation
    let standard_rate = if rownum == 0 {
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

    // Add LDI-specific charges
    let capital_charge = params.scr_rate * params.capital_ratio * params.coe;
    let ldi_rate = standard_rate + capital_charge + params.loading_rate;

    r.ftp_rate.as_mut().unwrap()[[rownum, colnum]] = ldi_rate;
}

/// ftp_int = ftp_rate × stock_amort / 12.
/// Uses the LDI-adjusted rate (standard + capital_charge + loading), not raw input_rate.
fn compute_ldi_ftp_int(r: &mut FtpResult, rownum: usize, colnum: usize) {
    let ftp_rate = r.ftp_rate.as_ref().unwrap()[[rownum, colnum]];
    let stock = r.stock_amort.as_ref().unwrap()[[rownum, colnum]];
    r.ftp_int.as_mut().unwrap()[[rownum, colnum]] = ftp_rate * stock / 12.0;
}

fn compute_ldi_market_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    use ndarray::array;

    #[test]
    fn test_ldi_basic() {
        let params = LdiParams::default();
        let _r = FtpResult::new(
            array![[1000.0]],
            array![[1.0, 0.5, 0.2]],
            array![[0.01, 0.02]],
        );

        // Note: This would need a compute method for LDI
        // For now just test params work
        assert!(params.scr_rate > 0.0);
    }
}

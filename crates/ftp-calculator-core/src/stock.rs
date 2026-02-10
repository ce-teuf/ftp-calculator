use crate::result::FtpResult;

/// Executes the **stock** method computation on an `FtpResult`.
///
/// All output matrices must already be initialised (via `compute()`).
pub(crate) fn compute_stock(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // --- Phase 1: stock_amort (vectorisable) ---
    // stock_amort[i,j] = outstanding[i,0] * profiles[i,j]
    {
        let sa = r.stock_amort.as_mut().unwrap();
        for i in 0..nrows {
            let o = r.input_outstanding[[i, 0]];
            for j in 0..ncols {
                sa[[i, j]] = o * r.input_profiles[[i, j]];
            }
        }
    }

    // --- Phase 2: stock_instal ---
    // stock_instal[i,0] = 0
    // stock_instal[i,j] = stock_amort[i,j-1] - stock_amort[i,j]  for j > 0
    {
        let sa = r.stock_amort.as_ref().unwrap();
        let si = r.stock_instal.as_mut().unwrap();
        for i in 0..nrows {
            for j in 1..ncols {
                si[[i, j]] = sa[[i, j - 1]] - sa[[i, j]];
            }
        }
    }

    // --- Phase 3: varstock_amort (stock variation) ---
    // row 0 or last col: varstock_amort[i,j] = stock_amort[i,j]
    // else:              varstock_amort[i,j] = stock_amort[i,j] - stock_amort[i-1,j+1]
    {
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
    }

    // --- Phase 4: varstock_instal ---
    // varstock_instal[i,0] = 0
    // varstock_instal[i,j] = varstock_amort[i,j-1] - varstock_amort[i,j]  for j > 0
    {
        let va = r.varstock_amort.as_ref().unwrap();
        let vi = r.varstock_instal.as_mut().unwrap();
        for i in 0..nrows {
            for j in 1..ncols {
                vi[[i, j]] = va[[i, j - 1]] - va[[i, j]];
            }
        }
    }

    // --- Phase 5: ftp_rate, ftp_int, market_rate (reverse-column, row-by-row) ---
    compute_rates(r, nrows, ncols);
}

/// Computes ftp_rate, ftp_int, and market_rate (shared by stock and flux).
pub(crate) fn compute_rates(r: &mut FtpResult, nrows: usize, ncols: usize) {
    for i in 0..nrows {
        for j in (0..ncols).rev() {
            if j > 0 {
                compute_ftp_rate(r, i, j - 1, ncols);
                compute_ftp_int(r, i, j - 1, ncols);
                compute_market_rate(r, i, j, ncols);
            }
        }
    }
}

/// FTP rate for cell (rownum, colnum).
///
/// Row 0:  weighted average of varstock_instal × input_rate
/// Row >0: weighted average of (varstock_instal × input_rate) + (stock_instal × market_rate)
fn compute_ftp_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
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

/// FTP interest for cell (rownum, colnum).
fn compute_ftp_int(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
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

/// Market rate for cell (rownum, colnum).
fn compute_market_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
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
    use crate::ComputeMethod;
    use ndarray::array;

    #[test]
    fn test_stock_amort_is_outstanding_times_profile() {
        let mut r = FtpResult::new(
            array![[1000.0]],
            array![[1.0, 0.5, 0.2]],
            array![[0.01, 0.02]],
        );
        r.compute(ComputeMethod::Stock).unwrap();
        let sa = r.stock_amort().unwrap();
        assert_eq!(sa[[0, 0]], 1000.0);
        assert_eq!(sa[[0, 1]], 500.0);
        assert_eq!(sa[[0, 2]], 200.0);
    }

    #[test]
    fn test_stock_instal_is_diff_of_adjacent() {
        let mut r = FtpResult::new(
            array![[1000.0]],
            array![[1.0, 0.5, 0.2]],
            array![[0.01, 0.02]],
        );
        r.compute(ComputeMethod::Stock).unwrap();
        let si = r.stock_instal().unwrap();
        assert_eq!(si[[0, 0]], 0.0);
        assert_eq!(si[[0, 1]], 500.0);
        assert_eq!(si[[0, 2]], 300.0);
    }

    #[test]
    fn test_varstock_first_row_equals_stock_amort() {
        let mut r = FtpResult::new(
            array![[1000.0]],
            array![[1.0, 0.5, 0.2]],
            array![[0.01, 0.02]],
        );
        r.compute(ComputeMethod::Stock).unwrap();
        let va = r.varstock_amort().unwrap();
        let sa = r.stock_amort().unwrap();
        assert_eq!(va[[0, 0]], sa[[0, 0]]);
        assert_eq!(va[[0, 1]], sa[[0, 1]]);
    }
}

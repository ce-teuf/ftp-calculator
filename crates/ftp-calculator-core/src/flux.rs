use ndarray::s;

use crate::result::FtpResult;
use crate::stock::compute_rates;
use crate::utils::extract_anti_diagonal_rect2;

/// Executes the **flux** method computation on an `FtpResult`.
///
/// All output matrices must already be initialised (via `compute()`).
pub(crate) fn compute_flux(r: &mut FtpResult, nrows: usize, ncols: usize) {
    for i in 0..nrows {
        for j in 0..ncols {
            // 1. New product (varstock_amort)
            flux_stock_var(r, i, j);
            // 2. varstock_instal
            flux_varstock_instal(r, i, j);
            // 3. stock_amort (sum of anti-diagonals)
            flux_stock_amort(r, i, j);
            // 4. stock_instal
            flux_stock_instal(r, i, j);
        }
    }

    // 5. ftp_rate, ftp_int, market_rate (same as stock method)
    compute_rates(r, nrows, ncols);
}

/// Flux method: new product (varstock_amort).
///
/// - Row 0:     `profile[i,j] * outstanding[i,0]`
/// - Col 0:     `max(0, outstanding[i,0] - sum of varstock_amort[i-k, k] for k=1..i)`
/// - Otherwise: `varstock_amort[i,0] * profile[i,j]`
fn flux_stock_var(r: &mut FtpResult, rownum: usize, colnum: usize) {
    let outstanding = &r.input_outstanding;
    let profiles = &r.input_profiles;

    let value = if rownum == 0 {
        profiles[[rownum, colnum]] * outstanding[[rownum, 0]]
    } else if colnum == 0 {
        let va = r.varstock_amort.as_ref().unwrap();
        let mut front_amt: f64 = 0.0;
        for i in 1..=rownum {
            front_amt += va[[rownum - i, i]];
        }
        front_amt = outstanding[[rownum, 0]] - front_amt;
        if front_amt < 0.0 {
            0.0
        } else {
            front_amt
        }
    } else {
        let va = r.varstock_amort.as_ref().unwrap();
        va[[rownum, 0]] * profiles[[rownum, colnum]]
    };

    r.varstock_amort.as_mut().unwrap()[[rownum, colnum]] = value;
}

/// varstock_instal[i,0] = 0
/// varstock_instal[i,j] = varstock_amort[i,j-1] - varstock_amort[i,j]  for j > 0
fn flux_varstock_instal(r: &mut FtpResult, rownum: usize, colnum: usize) {
    if colnum > 0 {
        let va = r.varstock_amort.as_ref().unwrap();
        let val = va[[rownum, colnum - 1]] - va[[rownum, colnum]];
        r.varstock_instal.as_mut().unwrap()[[rownum, colnum]] = val;
    }
}

/// Flux method: stock_amort via anti-diagonal sums.
///
/// - Row 0: `stock_amort[0,j] = varstock_amort[0,j]`
/// - Row >0: sum of anti-diagonal of `varstock_amort[0..=i, j..ncols]`
fn flux_stock_amort(r: &mut FtpResult, rownum: usize, colnum: usize) {
    let va = r.varstock_amort.as_ref().unwrap();

    let value = if rownum == 0 {
        va[[rownum, colnum]]
    } else {
        let (_, ncols) = va.dim();
        let slice = va.slice(s![0..rownum + 1, colnum..ncols]);
        let diag = extract_anti_diagonal_rect2(&slice);
        diag.iter().sum::<f64>()
    };

    r.stock_amort.as_mut().unwrap()[[rownum, colnum]] = value;
}

/// stock_instal[i,0] = 0
/// stock_instal[i,j] = stock_amort[i,j-1] - stock_amort[i,j]  for j > 0
fn flux_stock_instal(r: &mut FtpResult, rownum: usize, colnum: usize) {
    if colnum > 0 {
        let sa = r.stock_amort.as_ref().unwrap();
        let val = sa[[rownum, colnum - 1]] - sa[[rownum, colnum]];
        r.stock_instal.as_mut().unwrap()[[rownum, colnum]] = val;
    }
}

#[cfg(test)]
mod tests {
    use crate::result::FtpResult;
    use crate::ComputeMethod;
    use ndarray::array;

    #[test]
    fn test_flux_first_row_matches_outstanding_times_profile() {
        let mut r = FtpResult::new(
            array![[800.0]],
            array![[1.00, 0.60, 0.30]],
            array![[0.01200, 0.01300]],
        );
        r.compute(ComputeMethod::Flux).unwrap();
        let va = r.varstock_amort().unwrap();
        assert_eq!(va[[0, 0]], 800.0);
        assert_eq!(va[[0, 1]], 480.0);
        assert_eq!(va[[0, 2]], 240.0);
    }

    #[test]
    fn test_flux_stock_amort_first_row_equals_varstock() {
        let mut r = FtpResult::new(
            array![[800.0]],
            array![[1.00, 0.60, 0.30]],
            array![[0.01200, 0.01300]],
        );
        r.compute(ComputeMethod::Flux).unwrap();
        let sa = r.stock_amort().unwrap();
        let va = r.varstock_amort().unwrap();
        assert_eq!(sa[[0, 0]], va[[0, 0]]);
        assert_eq!(sa[[0, 1]], va[[0, 1]]);
        assert_eq!(sa[[0, 2]], va[[0, 2]]);
    }
}

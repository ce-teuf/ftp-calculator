use crate::result::FtpResult;

/// Duration Method computation.
///
/// The FTP rate is assigned based on the modified duration of the instrument
/// rather than its principal cash flow schedule. The modified duration captures
/// the average time-sensitivity of the instrument's value to interest rate changes.
///
/// FTP rate = CoF curve rate at tenor = modified_duration
pub(crate) fn compute_duration(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // Duration method: first compute stock_amort using the standard method
    // then adjust ftp_rate based on duration-based tenor lookup

    // Phase 1-4: same as stock method (stock_amort, stock_instal, varstock_amort, varstock_instal)
    compute_duration_stock(r, nrows, ncols);

    // Phase 5: ftp_rate, ftp_int, market_rate - using duration-adjusted rates
    compute_duration_rates(r, nrows, ncols);
}

fn compute_duration_stock(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // Same as stock method for stock_amort computation
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

fn compute_duration_rates(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // Duration method: FTP rate = CoF curve read at the modified duration tenor.
    // One scalar rate per row, uniform across all columns. No blending with MMFTP.

    for i in 0..nrows {
        let duration = compute_modified_duration(r, i, ncols);

        // Map continuous duration to the nearest available rate column (0-indexed).
        // duration is in periods; rate matrix has ncols-1 columns (col j covers period j+1).
        let tenor_idx = (duration.round() as usize).saturating_sub(1).min(ncols - 2);
        let duration_rate = r.input_rate[[i, tenor_idx]];

        // Assign the duration rate to every cell in this row, then back-solve market_rate.
        for j in (0..ncols).rev() {
            if j > 0 {
                r.ftp_rate.as_mut().unwrap()[[i, j - 1]] = duration_rate;
                compute_ftp_int(r, i, j - 1);
                compute_market_rate(r, i, j, ncols);
            }
        }
    }
}

fn compute_modified_duration(r: &FtpResult, row: usize, ncols: usize) -> f64 {
    // Modified duration = Σ(t × w_t) where w_t = PV(CF_t) / Total PV
    // For simplicity, we use the profile weights as an approximation

    let mut total_weight = 0.0;
    let mut weighted_sum = 0.0;

    for j in 0..ncols {
        let weight = r.input_profiles[[row, j]];
        total_weight += weight;
        weighted_sum += (j as f64 + 1.0) * weight;
    }

    if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        1.0
    }
}

/// ftp_int = ftp_rate × stock_amort / 12.
/// For the duration method the rate is a scalar per row, so this is exact.
fn compute_ftp_int(r: &mut FtpResult, rownum: usize, colnum: usize) {
    let ftp_rate = r.ftp_rate.as_ref().unwrap()[[rownum, colnum]];
    let stock = r.stock_amort.as_ref().unwrap()[[rownum, colnum]];
    r.ftp_int.as_mut().unwrap()[[rownum, colnum]] = ftp_rate * stock / 12.0;
}

fn compute_market_rate(r: &mut FtpResult, rownum: usize, colnum: usize, ncols: usize) {
    let input_rate = &r.input_rate;
    let stock_instal = r.stock_instal.as_ref().unwrap();
    let ftp_rate_mat = r.ftp_rate.as_ref().unwrap();

    let value = if colnum == ncols - 1 {
        input_rate[[rownum, colnum - 1]]
    } else {
        let a = ftp_rate_mat[[rownum, colnum]];
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
    fn test_duration_method_basic() {
        let mut r = FtpResult::new(
            array![[1000.0]],
            array![[1.0, 0.5, 0.2]],
            array![[0.01, 0.02]],
        );
        r.compute(ComputeMethod::Duration).unwrap();

        let sa = r.stock_amort().unwrap();
        assert_eq!(sa[[0, 0]], 1000.0);
        assert_eq!(sa[[0, 1]], 500.0);
    }
}

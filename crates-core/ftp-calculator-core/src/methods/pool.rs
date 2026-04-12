use crate::result::FtpResult;

/// Pool Method computation.
///
/// Single Pool: One blended FTP rate for all assets/liabilities regardless of maturity.
/// Multiple Pool: Separate rates for maturity bands (<1Y, 1-5Y, >5Y).
///
/// The pool rate is calculated as a weighted average of all positions' rates.
pub(crate) fn compute_pool(r: &mut FtpResult, nrows: usize, ncols: usize, pool_type: PoolType) {
    // Compute standard stock first
    compute_pool_stock(r, nrows, ncols);

    // Compute pool rates based on pool type
    match pool_type {
        PoolType::Single => compute_single_pool_rates(r, nrows, ncols),
        PoolType::Multiple(buckets) => compute_multiple_pool_rates(r, nrows, ncols, &buckets),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolType {
    Single,
    Multiple(Vec<PoolBucket>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolBucket {
    pub name: String,
    pub min_tenor: usize,
    pub max_tenor: usize,
}

fn compute_pool_stock(r: &mut FtpResult, nrows: usize, ncols: usize) {
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

fn compute_single_pool_rates(r: &mut FtpResult, nrows: usize, ncols: usize) {
    // Calculate the single pool rate as weighted average of all input rates
    let mut total_weighted_rate = 0.0;
    let mut total_weight = 0.0;

    for i in 0..nrows {
        for j in 0..ncols - 1 {
            let weight = r.varstock_instal.as_ref().unwrap()[[i, j + 1]];
            let rate = r.input_rate[[i, j]];
            total_weighted_rate += weight * rate;
            total_weight += weight;
        }
    }

    let pool_rate = if total_weight > 0.0 {
        total_weighted_rate / total_weight
    } else {
        0.0
    };

    // Apply the same pool rate to all cells
    for i in 0..nrows {
        for j in 0..ncols {
            r.ftp_rate.as_mut().unwrap()[[i, j]] = pool_rate;
        }
    }

    // Compute ftp_int and market_rate
    compute_pool_ftp_int(r, nrows, ncols, pool_rate);
    compute_pool_market_rate(r, nrows, ncols);
}

fn compute_multiple_pool_rates(
    r: &mut FtpResult,
    nrows: usize,
    ncols: usize,
    buckets: &[PoolBucket],
) {
    // For each bucket, compute a separate pool rate
    let mut bucket_rates: Vec<f64> = vec![0.0; buckets.len()];

    for (bucket_idx, bucket) in buckets.iter().enumerate() {
        let mut total_weighted_rate = 0.0;
        let mut total_weight = 0.0;

        for i in 0..nrows {
            for j in bucket.min_tenor..bucket.max_tenor.min(ncols - 1) {
                let weight = r.varstock_instal.as_ref().unwrap()[[i, j + 1]];
                let rate = r.input_rate[[i, j]];
                total_weighted_rate += weight * rate;
                total_weight += weight;
            }
        }

        bucket_rates[bucket_idx] = if total_weight > 0.0 {
            total_weighted_rate / total_weight
        } else {
            0.0
        };
    }

    // Assign rates based on which bucket each position falls into
    for i in 0..nrows {
        for j in 0..ncols {
            let mut assigned_rate = 0.0;
            for (bucket_idx, bucket) in buckets.iter().enumerate() {
                if j >= bucket.min_tenor && j < bucket.max_tenor {
                    assigned_rate = bucket_rates[bucket_idx];
                    break;
                }
            }
            r.ftp_rate.as_mut().unwrap()[[i, j]] = assigned_rate;
        }
    }

    // Compute ftp_int and market_rate
    let default_rate = bucket_rates.first().copied().unwrap_or(0.0);
    compute_pool_ftp_int(r, nrows, ncols, default_rate);
    compute_pool_market_rate(r, nrows, ncols);
}

fn compute_pool_ftp_int(r: &mut FtpResult, nrows: usize, ncols: usize, _pool_rate: f64) {
    // For pool method, ftp_int is based on the pool rate applied to outstanding
    for i in 0..nrows {
        for j in 0..ncols {
            let outstanding = r.stock_amort.as_ref().unwrap()[[i, j]];
            let ftp_rate = r.ftp_rate.as_ref().unwrap()[[i, j]];
            r.ftp_int.as_mut().unwrap()[[i, j]] = outstanding * ftp_rate / 12.0;
        }
    }
}

fn compute_pool_market_rate(r: &mut FtpResult, nrows: usize, ncols: usize) {
    for i in 0..nrows {
        for j in 0..ncols {
            if j == ncols - 1 {
                // Last column: use the last available rate
                r.market_rate.as_mut().unwrap()[[i, j]] = r.input_rate[[i, (ncols - 2).min(j)]];
            } else {
                // For pool method, market rate equals ftp rate
                r.market_rate.as_mut().unwrap()[[i, j]] = r.ftp_rate.as_ref().unwrap()[[i, j]];
            }
        }
    }
}

// Public API for pool method
pub fn compute_single_pool(r: &mut FtpResult, nrows: usize, ncols: usize) {
    compute_pool(r, nrows, ncols, PoolType::Single);
}

pub fn compute_multiple_pool(
    r: &mut FtpResult,
    nrows: usize,
    ncols: usize,
    buckets: Vec<PoolBucket>,
) {
    compute_pool(r, nrows, ncols, PoolType::Multiple(buckets));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComputeMethod;
    use ndarray::array;

    #[test]
    fn test_single_pool_basic() {
        let mut r = FtpResult::new(
            array![[1000.0]],
            array![[1.0, 0.5, 0.2]],
            array![[0.01, 0.02]],
        );
        // Pool method uses the standard compute() interface
        r.compute(ComputeMethod::Pool).unwrap();

        let sa = r.stock_amort().unwrap();
        assert_eq!(sa[[0, 0]], 1000.0);
    }
}

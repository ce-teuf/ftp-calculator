use crate::error::FtpError;

pub fn validate_dimensions(
    nrows_outs: usize,
    ncols_outs: usize,
    nrows_profiles: usize,
    ncols_profiles: usize,
    nrows_rate: usize,
    ncols_rate: usize,
) -> Result<(), FtpError> {
    if nrows_outs != nrows_profiles || nrows_outs != nrows_rate {
        return Err(FtpError::DimensionMismatch {
            expected: (nrows_outs, 0),
            got: (nrows_profiles, nrows_rate),
        });
    }
    if ncols_outs != 1 {
        return Err(FtpError::InvalidOutstandingColumns { got: ncols_outs });
    }
    if ncols_profiles - 1 != ncols_rate {
        return Err(FtpError::RateProfileColumnMismatch {
            rate_cols: ncols_rate,
            profile_cols: ncols_profiles,
        });
    }
    Ok(())
}

pub fn validate_positive_values(arr: &ndarray::Array2<f64>, name: &str) -> Result<(), FtpError> {
    for ((i, j), val) in arr.indexed_iter() {
        if val.is_nan() || val.is_infinite() {
            return Err(FtpError::InvalidValue {
                field: name.to_string(),
                message: format!("NaN or Inf at [{}, {}]", i, j),
            });
        }
        if *val < 0.0 {
            return Err(FtpError::InvalidValue {
                field: name.to_string(),
                message: format!("Negative value at [{}, {}]: {}", i, j, val),
            });
        }
    }
    Ok(())
}

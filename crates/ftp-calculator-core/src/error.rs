use std::fmt;

/// Errors that can occur during FTP calculations.
#[derive(Debug)]
pub enum FtpError {
    /// Input matrices have different numbers of rows.
    DimensionMismatch {
        expected: (usize, usize),
        got: (usize, usize),
    },
    /// `input_outstanding` must have exactly 1 column.
    InvalidOutstandingColumns { got: usize },
    /// `input_rate` must have exactly one fewer column than `input_profiles`.
    RateProfileColumnMismatch {
        rate_cols: usize,
        profile_cols: usize,
    },
}

impl fmt::Display for FtpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FtpError::DimensionMismatch { expected, got } => {
                write!(
                    f,
                    "dimension mismatch: expected {}x{}, got {}x{}",
                    expected.0, expected.1, got.0, got.1
                )
            }
            FtpError::InvalidOutstandingColumns { got } => {
                write!(f, "'input_outstanding' must have 1 column, got {}", got)
            }
            FtpError::RateProfileColumnMismatch {
                rate_cols,
                profile_cols,
            } => {
                write!(
                    f,
                    "'input_rate' must have one fewer column than 'input_profiles' \
                     (rate_cols={}, profile_cols={})",
                    rate_cols, profile_cols
                )
            }
        }
    }
}

impl std::error::Error for FtpError {}

use ndarray::Array2;

use crate::error::FtpError;
use crate::flux;
use crate::stock;

/// Method used for FTP computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeMethod {
    Stock,
    Flux,
}

/// Main structure holding all FTP calculation inputs and outputs.
///
/// # Examples
///
/// ```
/// use ftp_core::{FtpResult, ComputeMethod};
/// use ndarray::array;
///
/// let mut result = FtpResult::new(
///     array![[1000.0]],
///     array![[1.0, 0.5, 0.2]],
///     array![[0.01, 0.02]],
/// );
/// result.compute(ComputeMethod::Stock).unwrap();
/// assert!(result.stock_amort().is_some());
/// ```
pub struct FtpResult {
    // Inputs
    pub(crate) input_outstanding: Array2<f64>,
    pub(crate) input_profiles: Array2<f64>,
    pub(crate) input_rate: Array2<f64>,

    // Outputs
    pub(crate) stock_amort: Option<Array2<f64>>,
    pub(crate) stock_instal: Option<Array2<f64>>,
    pub(crate) varstock_amort: Option<Array2<f64>>,
    pub(crate) varstock_instal: Option<Array2<f64>>,
    pub(crate) ftp_rate: Option<Array2<f64>>,
    pub(crate) ftp_int: Option<Array2<f64>>,
    pub(crate) market_rate: Option<Array2<f64>>,
}

impl FtpResult {
    /// Creates a new `FtpResult` with the given input matrices.
    pub fn new(
        input_outstanding: Array2<f64>,
        input_profiles: Array2<f64>,
        input_rate: Array2<f64>,
    ) -> Self {
        Self {
            input_outstanding,
            input_profiles,
            input_rate,
            stock_amort: None,
            stock_instal: None,
            varstock_amort: None,
            varstock_instal: None,
            ftp_rate: None,
            ftp_int: None,
            market_rate: None,
        }
    }

    /// Validates that input matrix dimensions are consistent.
    fn check_dims(&self) -> Result<(), FtpError> {
        let (nrows_outs, ncols_outs) = self.input_outstanding.dim();
        let (nrows_profiles, _ncols_profiles) = self.input_profiles.dim();
        let (nrows_rate, ncols_rate) = self.input_rate.dim();

        if nrows_outs != nrows_profiles || nrows_outs != nrows_rate {
            return Err(FtpError::DimensionMismatch {
                expected: (nrows_outs, 0),
                got: (nrows_profiles, nrows_rate),
            });
        }
        if ncols_outs != 1 {
            return Err(FtpError::InvalidOutstandingColumns { got: ncols_outs });
        }
        let ncols_profiles = self.input_profiles.dim().1;
        if ncols_profiles - 1 != ncols_rate {
            return Err(FtpError::RateProfileColumnMismatch {
                rate_cols: ncols_rate,
                profile_cols: ncols_profiles,
            });
        }
        Ok(())
    }

    /// Runs the FTP computation using the specified method.
    pub fn compute(&mut self, method: ComputeMethod) -> Result<(), FtpError> {
        self.check_dims()?;

        let (nrows, ncols) = self.input_profiles.dim();

        // Initialize output arrays
        self.stock_amort = Some(Array2::<f64>::zeros((nrows, ncols)));
        self.stock_instal = Some(Array2::<f64>::zeros((nrows, ncols)));
        self.varstock_amort = Some(Array2::<f64>::zeros((nrows, ncols)));
        self.varstock_instal = Some(Array2::<f64>::zeros((nrows, ncols)));
        self.ftp_rate = Some(Array2::<f64>::zeros((nrows, ncols)));
        self.ftp_int = Some(Array2::<f64>::zeros((nrows, ncols)));
        self.market_rate = Some(Array2::<f64>::zeros((nrows, ncols)));

        match method {
            ComputeMethod::Stock => stock::compute_stock(self, nrows, ncols),
            ComputeMethod::Flux => flux::compute_flux(self, nrows, ncols),
        }

        Ok(())
    }

    // --- Getters ---

    pub fn input_outstanding(&self) -> &Array2<f64> {
        &self.input_outstanding
    }

    pub fn input_profiles(&self) -> &Array2<f64> {
        &self.input_profiles
    }

    pub fn input_rate(&self) -> &Array2<f64> {
        &self.input_rate
    }

    pub fn stock_amort(&self) -> Option<&Array2<f64>> {
        self.stock_amort.as_ref()
    }

    pub fn stock_instal(&self) -> Option<&Array2<f64>> {
        self.stock_instal.as_ref()
    }

    pub fn varstock_amort(&self) -> Option<&Array2<f64>> {
        self.varstock_amort.as_ref()
    }

    pub fn varstock_instal(&self) -> Option<&Array2<f64>> {
        self.varstock_instal.as_ref()
    }

    pub fn ftp_rate(&self) -> Option<&Array2<f64>> {
        self.ftp_rate.as_ref()
    }

    pub fn ftp_int(&self) -> Option<&Array2<f64>> {
        self.ftp_int.as_ref()
    }

    pub fn market_rate(&self) -> Option<&Array2<f64>> {
        self.market_rate.as_ref()
    }
}

pub mod validation;

use ndarray::Array2;

pub struct FtpInputs {
    pub outstanding: Array2<f64>,
    pub profiles: Array2<f64>,
    pub rates: Array2<f64>,
}

impl FtpInputs {
    pub fn new(outstanding: Array2<f64>, profiles: Array2<f64>, rates: Array2<f64>) -> Self {
        Self {
            outstanding,
            profiles,
            rates,
        }
    }

    pub fn nrows(&self) -> usize {
        self.outstanding.dim().0
    }

    pub fn ncols(&self) -> usize {
        self.profiles.dim().1
    }
}

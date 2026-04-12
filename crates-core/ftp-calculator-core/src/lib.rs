//! # FTP Core Library
//!
//! Core library for Funds Transfer Pricing (FTP) calculations.
//!
//! ## Main features
//!
//! - Multiple computation methods (Stock, Flux, Duration, Pool, etc.)
//! - Matrix-based FTP rate, interest, and market rate calculations
//! - Curve components management (14 composantes de taux)
//! - Input validation
//!
//! ## Usage
//!
//! ```rust
//! use ftp_calculator_core::{FtpResult, ComputeMethod};
//! use ndarray::array;
//!
//! let mut result = FtpResult::new(
//!     array![[1000.0]],
//!     array![[1.0, 0.5, 0.2]],
//!     array![[0.01, 0.02]],
//! );
//! result.compute(ComputeMethod::Stock).unwrap();
//! ```

pub mod curve;
pub mod error;
pub mod inputs;
pub mod methods;
pub mod result;
pub mod utils;

pub use crate::curve::components::{CurveBundle, RateCurve};
pub use crate::error::FtpError;
pub use crate::inputs::FtpInputs;
pub use crate::result::{ComputeMethod, FtpResult};

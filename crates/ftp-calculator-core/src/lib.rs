//! # FTP Core Library
//!
//! Core library for Funds Transfer Pricing (FTP) calculations.
//!
//! ## Main features
//!
//! - Stock and flux computation methods
//! - Matrix-based FTP rate, interest, and market rate calculations
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

mod error;
mod flux;
mod result;
mod stock;
mod utils;

pub use crate::error::FtpError;
pub use crate::result::{ComputeMethod, FtpResult};

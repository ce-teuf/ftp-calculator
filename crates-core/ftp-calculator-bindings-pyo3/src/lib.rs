use ndarray::Array2;
use numpy::{IntoPyArray, PyArray2, PyReadonlyArray2};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use ftp_calculator_core::{ComputeMethod, FtpError, FtpResult};

/// Convert an FtpError into a Python ValueError.
fn ftp_err(e: FtpError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(e.to_string())
}

/// Helper: get a computed output or raise ValueError.
fn require_output<'a>(opt: Option<&'a Array2<f64>>, name: &str) -> PyResult<&'a Array2<f64>> {
    opt.ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "'{name}' not available — call compute() first"
        ))
    })
}

/// FTP Calculator — wraps the Rust ftp_core engine.
///
/// Usage:
///     calc = FtpCalculator(outstanding, profiles, rates)
///     calc.compute("stock")
///     result = calc.stock_amort  # numpy 2D array
#[pyclass]
struct FtpCalculator {
    inner: FtpResult,
}

#[pymethods]
impl FtpCalculator {
    #[new]
    fn new(
        outstanding: PyReadonlyArray2<'_, f64>,
        profiles: PyReadonlyArray2<'_, f64>,
        rates: PyReadonlyArray2<'_, f64>,
    ) -> Self {
        Self {
            inner: FtpResult::new(
                outstanding.as_array().to_owned(),
                profiles.as_array().to_owned(),
                rates.as_array().to_owned(),
            ),
        }
    }

    /// Run the FTP computation. method must be "stock" or "flux".
    fn compute(&mut self, method: &str) -> PyResult<()> {
        let m = match method {
            "stock" => ComputeMethod::Stock,
            "flux" => ComputeMethod::Flux,
            other => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "unknown method '{other}' — use 'stock' or 'flux'"
                )));
            }
        };
        self.inner.compute(m).map_err(ftp_err)
    }

    /// (rows, cols) of the profile matrix.
    #[getter]
    fn dims(&self) -> (usize, usize) {
        self.inner.input_profiles().dim()
    }

    // --- output getters (return numpy arrays) ---

    #[getter]
    fn stock_amort<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = require_output(self.inner.stock_amort(), "stock_amort")?;
        Ok(arr.to_owned().into_pyarray(py))
    }

    #[getter]
    fn stock_instal<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = require_output(self.inner.stock_instal(), "stock_instal")?;
        Ok(arr.to_owned().into_pyarray(py))
    }

    #[getter]
    fn varstock_amort<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = require_output(self.inner.varstock_amort(), "varstock_amort")?;
        Ok(arr.to_owned().into_pyarray(py))
    }

    #[getter]
    fn varstock_instal<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = require_output(self.inner.varstock_instal(), "varstock_instal")?;
        Ok(arr.to_owned().into_pyarray(py))
    }

    #[getter]
    fn ftp_rate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = require_output(self.inner.ftp_rate(), "ftp_rate")?;
        Ok(arr.to_owned().into_pyarray(py))
    }

    #[getter]
    fn ftp_int<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = require_output(self.inner.ftp_int(), "ftp_int")?;
        Ok(arr.to_owned().into_pyarray(py))
    }

    #[getter]
    fn market_rate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = require_output(self.inner.market_rate(), "market_rate")?;
        Ok(arr.to_owned().into_pyarray(py))
    }

    fn __repr__(&self) -> String {
        let (r, c) = self.inner.input_profiles().dim();
        let computed = self.inner.stock_amort().is_some();
        format!("FtpCalculator(rows={r}, cols={c}, computed={computed})")
    }
}

// --- One-shot convenience functions ---

fn run_compute<'py>(
    py: Python<'py>,
    outstanding: PyReadonlyArray2<'py, f64>,
    profiles: PyReadonlyArray2<'py, f64>,
    rates: PyReadonlyArray2<'py, f64>,
    method: ComputeMethod,
) -> PyResult<Bound<'py, PyDict>> {
    let mut r = FtpResult::new(
        outstanding.as_array().to_owned(),
        profiles.as_array().to_owned(),
        rates.as_array().to_owned(),
    );
    r.compute(method).map_err(ftp_err)?;

    let dict = PyDict::new(py);
    dict.set_item(
        "stock_amort",
        r.stock_amort().unwrap().to_owned().into_pyarray(py),
    )?;
    dict.set_item(
        "stock_instal",
        r.stock_instal().unwrap().to_owned().into_pyarray(py),
    )?;
    dict.set_item(
        "varstock_amort",
        r.varstock_amort().unwrap().to_owned().into_pyarray(py),
    )?;
    dict.set_item(
        "varstock_instal",
        r.varstock_instal().unwrap().to_owned().into_pyarray(py),
    )?;
    dict.set_item(
        "ftp_rate",
        r.ftp_rate().unwrap().to_owned().into_pyarray(py),
    )?;
    dict.set_item("ftp_int", r.ftp_int().unwrap().to_owned().into_pyarray(py))?;
    dict.set_item(
        "market_rate",
        r.market_rate().unwrap().to_owned().into_pyarray(py),
    )?;
    Ok(dict)
}

/// Compute FTP using the stock method. Returns a dict of numpy arrays.
#[pyfunction]
fn compute_stock<'py>(
    py: Python<'py>,
    outstanding: PyReadonlyArray2<'py, f64>,
    profiles: PyReadonlyArray2<'py, f64>,
    rates: PyReadonlyArray2<'py, f64>,
) -> PyResult<Bound<'py, PyDict>> {
    run_compute(py, outstanding, profiles, rates, ComputeMethod::Stock)
}

/// Compute FTP using the flux method. Returns a dict of numpy arrays.
#[pyfunction]
fn compute_flux<'py>(
    py: Python<'py>,
    outstanding: PyReadonlyArray2<'py, f64>,
    profiles: PyReadonlyArray2<'py, f64>,
    rates: PyReadonlyArray2<'py, f64>,
) -> PyResult<Bound<'py, PyDict>> {
    run_compute(py, outstanding, profiles, rates, ComputeMethod::Flux)
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FtpCalculator>()?;
    m.add_function(wrap_pyfunction!(compute_stock, m)?)?;
    m.add_function(wrap_pyfunction!(compute_flux, m)?)?;
    Ok(())
}

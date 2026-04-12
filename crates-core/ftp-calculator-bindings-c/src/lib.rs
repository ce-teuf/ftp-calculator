use std::cell::RefCell;
use std::ffi::c_char;
use std::slice;

use ftp_calculator_core::{ComputeMethod, FtpResult};
use ndarray::Array2;

// Thread-local storage for the last error message.
thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

fn set_last_error(msg: String) {
    LAST_ERROR.with(|e| *e.borrow_mut() = msg);
}

/// Opaque handle returned to C callers.
pub struct FtpHandle {
    inner: FtpResult,
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

/// Creates a new `FtpHandle` from raw row-major data.
///
/// - `outstanding`: pointer to `outs_rows` doubles (column vector)
/// - `profiles`:    pointer to `prof_rows * prof_cols` doubles (row-major)
/// - `rates`:       pointer to `rate_rows * rate_cols` doubles (row-major)
///
/// Returns a heap-allocated handle, or null on failure.
///
/// # Safety
/// Caller must ensure all pointers are valid and point to properly aligned data
/// with the specified dimensions.
#[no_mangle]
pub unsafe extern "C" fn ftp_create(
    outstanding: *const f64,
    outs_rows: i32,
    profiles: *const f64,
    prof_rows: i32,
    prof_cols: i32,
    rates: *const f64,
    rate_rows: i32,
    rate_cols: i32,
) -> *mut FtpHandle {
    if outstanding.is_null() || profiles.is_null() || rates.is_null() {
        set_last_error("null pointer argument".into());
        return std::ptr::null_mut();
    }
    if outs_rows <= 0 || prof_rows <= 0 || prof_cols <= 0 || rate_rows <= 0 || rate_cols <= 0 {
        set_last_error("dimensions must be positive".into());
        return std::ptr::null_mut();
    }

    let outs_rows = outs_rows as usize;
    let prof_rows = prof_rows as usize;
    let prof_cols = prof_cols as usize;
    let rate_rows = rate_rows as usize;
    let rate_cols = rate_cols as usize;

    let outs_slice = slice::from_raw_parts(outstanding, outs_rows);
    let prof_slice = slice::from_raw_parts(profiles, prof_rows * prof_cols);
    let rate_slice = slice::from_raw_parts(rates, rate_rows * rate_cols);

    let input_outstanding = match Array2::from_shape_vec((outs_rows, 1), outs_slice.to_vec()) {
        Ok(a) => a,
        Err(e) => {
            set_last_error(format!("outstanding array: {}", e));
            return std::ptr::null_mut();
        }
    };
    let input_profiles = match Array2::from_shape_vec((prof_rows, prof_cols), prof_slice.to_vec()) {
        Ok(a) => a,
        Err(e) => {
            set_last_error(format!("profiles array: {}", e));
            return std::ptr::null_mut();
        }
    };
    let input_rate = match Array2::from_shape_vec((rate_rows, rate_cols), rate_slice.to_vec()) {
        Ok(a) => a,
        Err(e) => {
            set_last_error(format!("rates array: {}", e));
            return std::ptr::null_mut();
        }
    };

    let result = FtpResult::new(input_outstanding, input_profiles, input_rate);
    let handle = Box::new(FtpHandle { inner: result });
    Box::into_raw(handle)
}

/// Frees an `FtpHandle`. No-op if `handle` is null.
///
/// # Safety
/// Caller must ensure the handle was created by `ftp_create` and has not been freed already.
#[no_mangle]
pub unsafe extern "C" fn ftp_free(handle: *mut FtpHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

// ---------------------------------------------------------------------------
// Compute
// ---------------------------------------------------------------------------

/// Runs the FTP computation.
///
/// - `method`: 0 = Stock, 1 = Flux
///
/// Returns 0 on success, -1 on error (call `ftp_get_last_error`).
///
/// # Safety
/// Caller must ensure the handle is valid and not null.
#[no_mangle]
pub unsafe extern "C" fn ftp_compute(handle: *mut FtpHandle, method: i32) -> i32 {
    if handle.is_null() {
        set_last_error("null handle".into());
        return -1;
    }
    let h = &mut *handle;
    let compute_method = match method {
        0 => ComputeMethod::Stock,
        1 => ComputeMethod::Flux,
        _ => {
            set_last_error(format!(
                "unknown method: {} (expected 0=Stock, 1=Flux)",
                method
            ));
            return -1;
        }
    };
    match h.inner.compute(compute_method) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

// ---------------------------------------------------------------------------
// Dimension query
// ---------------------------------------------------------------------------

/// Writes the output matrix dimensions (rows, cols) into the provided pointers.
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
/// Caller must ensure all pointers are valid and not null.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_dims(
    handle: *const FtpHandle,
    out_rows: *mut i32,
    out_cols: *mut i32,
) -> i32 {
    if handle.is_null() || out_rows.is_null() || out_cols.is_null() {
        set_last_error("null pointer argument".into());
        return -1;
    }
    let h = &*handle;
    let (r, c) = h.inner.input_profiles().dim();
    *out_rows = r as i32;
    *out_cols = c as i32;
    0
}

// ---------------------------------------------------------------------------
// Getters — copy matrix data into caller-provided buffer
// ---------------------------------------------------------------------------

/// Helper: copies an `Option<&Array2<f64>>` into a flat `out_buf` of length `buf_len`.
///
/// Returns 0 on success, -1 on error.
unsafe fn copy_matrix(
    mat: Option<&Array2<f64>>,
    name: &str,
    out_buf: *mut f64,
    buf_len: i32,
) -> i32 {
    if out_buf.is_null() {
        set_last_error("null output buffer".into());
        return -1;
    }
    let arr = match mat {
        Some(a) => a,
        None => {
            set_last_error(format!("{}: not yet computed", name));
            return -1;
        }
    };
    let total = arr.len();
    if (buf_len as usize) < total {
        set_last_error(format!(
            "{}: buffer too small ({} < {})",
            name, buf_len, total
        ));
        return -1;
    }
    // ndarray default layout is row-major — iterate in standard order
    let dst = slice::from_raw_parts_mut(out_buf, total);
    for (i, val) in arr.iter().enumerate() {
        dst[i] = *val;
    }
    0
}

/// Copies stock_amort into `out_buf`. Returns 0 on success, -1 on error.
///
/// # Safety
/// Caller must ensure handle and out_buf are valid pointers with sufficient capacity.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_stock_amort(
    handle: *const FtpHandle,
    out_buf: *mut f64,
    buf_len: i32,
) -> i32 {
    if handle.is_null() {
        set_last_error("null handle".into());
        return -1;
    }
    copy_matrix(
        (*handle).inner.stock_amort(),
        "stock_amort",
        out_buf,
        buf_len,
    )
}

/// Copies stock_instal into `out_buf`. Returns 0 on success, -1 on error.
///
/// # Safety
/// Caller must ensure handle and out_buf are valid pointers with sufficient capacity.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_stock_instal(
    handle: *const FtpHandle,
    out_buf: *mut f64,
    buf_len: i32,
) -> i32 {
    if handle.is_null() {
        set_last_error("null handle".into());
        return -1;
    }
    copy_matrix(
        (*handle).inner.stock_instal(),
        "stock_instal",
        out_buf,
        buf_len,
    )
}

/// Copies varstock_amort into `out_buf`. Returns 0 on success, -1 on error.
///
/// # Safety
/// Caller must ensure handle and out_buf are valid pointers with sufficient capacity.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_varstock_amort(
    handle: *const FtpHandle,
    out_buf: *mut f64,
    buf_len: i32,
) -> i32 {
    if handle.is_null() {
        set_last_error("null handle".into());
        return -1;
    }
    copy_matrix(
        (*handle).inner.varstock_amort(),
        "varstock_amort",
        out_buf,
        buf_len,
    )
}

/// Copies varstock_instal into `out_buf`. Returns 0 on success, -1 on error.
///
/// # Safety
/// Caller must ensure handle and out_buf are valid pointers with sufficient capacity.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_varstock_instal(
    handle: *const FtpHandle,
    out_buf: *mut f64,
    buf_len: i32,
) -> i32 {
    if handle.is_null() {
        set_last_error("null handle".into());
        return -1;
    }
    copy_matrix(
        (*handle).inner.varstock_instal(),
        "varstock_instal",
        out_buf,
        buf_len,
    )
}

/// Copies ftp_rate into `out_buf`. Returns 0 on success, -1 on error.
///
/// # Safety
/// Caller must ensure handle and out_buf are valid pointers with sufficient capacity.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_ftp_rate(
    handle: *const FtpHandle,
    out_buf: *mut f64,
    buf_len: i32,
) -> i32 {
    if handle.is_null() {
        set_last_error("null handle".into());
        return -1;
    }
    copy_matrix((*handle).inner.ftp_rate(), "ftp_rate", out_buf, buf_len)
}

/// Copies ftp_int into `out_buf`. Returns 0 on success, -1 on error.
///
/// # Safety
/// Caller must ensure handle and out_buf are valid pointers with sufficient capacity.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_ftp_int(
    handle: *const FtpHandle,
    out_buf: *mut f64,
    buf_len: i32,
) -> i32 {
    if handle.is_null() {
        set_last_error("null handle".into());
        return -1;
    }
    copy_matrix((*handle).inner.ftp_int(), "ftp_int", out_buf, buf_len)
}

/// Copies market_rate into `out_buf`. Returns 0 on success, -1 on error.
///
/// # Safety
/// Caller must ensure handle and out_buf are valid pointers with sufficient capacity.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_market_rate(
    handle: *const FtpHandle,
    out_buf: *mut f64,
    buf_len: i32,
) -> i32 {
    if handle.is_null() {
        set_last_error("null handle".into());
        return -1;
    }
    copy_matrix(
        (*handle).inner.market_rate(),
        "market_rate",
        out_buf,
        buf_len,
    )
}

// ---------------------------------------------------------------------------
// Error reporting
// ---------------------------------------------------------------------------

/// Copies the last error message into `buf` (max `buf_len` bytes, NUL-terminated).
///
/// Returns 0 on success, -1 if `buf` is null or the message was truncated.
///
/// # Safety
/// Caller must ensure buf is a valid pointer to a buffer of at least buf_len bytes.
#[no_mangle]
pub unsafe extern "C" fn ftp_get_last_error(buf: *mut c_char, buf_len: i32) -> i32 {
    if buf.is_null() || buf_len <= 0 {
        return -1;
    }
    LAST_ERROR.with(|e| {
        let msg = e.borrow();
        let bytes = msg.as_bytes();
        let max = (buf_len as usize) - 1; // leave room for NUL
        let copy_len = bytes.len().min(max);
        let dst = slice::from_raw_parts_mut(buf as *mut u8, buf_len as usize);
        dst[..copy_len].copy_from_slice(&bytes[..copy_len]);
        dst[copy_len] = 0; // NUL terminator
        if bytes.len() > max {
            -1
        } else {
            0
        }
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_compute_read_stock() {
        unsafe {
            let outstanding = [1000.0f64, 1200.0, 1350.0];
            let profiles = [
                1.00, 0.50, 0.20, 0.05, // row 0
                1.00, 0.50, 0.20, 0.05, // row 1
                1.00, 0.50, 0.20, 0.05, // row 2
            ];
            let rates = [
                0.01300, 0.01400, 0.01600, // row 0
                0.01360, 0.01460, 0.01660, // row 1
                0.01430, 0.01530, 0.01730, // row 2
            ];

            let h = ftp_create(
                outstanding.as_ptr(),
                3,
                profiles.as_ptr(),
                3,
                4,
                rates.as_ptr(),
                3,
                3,
            );
            assert!(!h.is_null());

            // Compute stock
            let rc = ftp_compute(h, 0);
            assert_eq!(rc, 0);

            // Check dims
            let mut rows: i32 = 0;
            let mut cols: i32 = 0;
            assert_eq!(ftp_get_dims(h, &mut rows, &mut cols), 0);
            assert_eq!(rows, 3);
            assert_eq!(cols, 4);

            // Read stock_amort
            let size = (rows * cols) as usize;
            let mut buf = vec![0.0f64; size];
            assert_eq!(ftp_get_stock_amort(h, buf.as_mut_ptr(), size as i32), 0);
            assert_eq!(buf[0], 1000.0); // [0,0]
            assert_eq!(buf[1], 500.0); // [0,1]
            assert_eq!(buf[4], 1200.0); // [1,0]
            assert_eq!(buf[11], 67.5); // [2,3]

            // Read ftp_rate
            assert_eq!(ftp_get_ftp_rate(h, buf.as_mut_ptr(), size as i32), 0);
            assert!((buf[0] - 0.0137894737).abs() < 1e-8);

            ftp_free(h);
        }
    }

    #[test]
    fn test_create_compute_read_flux() {
        unsafe {
            let outstanding = [800.0f64, 900.0];
            let profiles = [
                1.00, 0.60, 0.30, // row 0
                1.00, 0.60, 0.30, // row 1
            ];
            let rates = [
                0.01200, 0.01300, // row 0
                0.01250, 0.01350, // row 1
            ];

            let h = ftp_create(
                outstanding.as_ptr(),
                2,
                profiles.as_ptr(),
                2,
                3,
                rates.as_ptr(),
                2,
                2,
            );
            assert!(!h.is_null());

            let rc = ftp_compute(h, 1); // Flux
            assert_eq!(rc, 0);

            let mut rows: i32 = 0;
            let mut cols: i32 = 0;
            ftp_get_dims(h, &mut rows, &mut cols);
            let size = (rows * cols) as usize;
            let mut buf = vec![0.0f64; size];

            // varstock_amort
            assert_eq!(ftp_get_varstock_amort(h, buf.as_mut_ptr(), size as i32), 0);
            assert_eq!(buf[0], 800.0);
            assert_eq!(buf[1], 480.0);
            assert_eq!(buf[3], 420.0);

            // stock_amort
            assert_eq!(ftp_get_stock_amort(h, buf.as_mut_ptr(), size as i32), 0);
            assert_eq!(buf[3], 900.0); // [1,0]
            assert_eq!(buf[4], 492.0); // [1,1]

            ftp_free(h);
        }
    }

    #[test]
    fn test_null_handle_returns_error() {
        unsafe {
            assert_eq!(ftp_compute(std::ptr::null_mut(), 0), -1);

            let mut buf = vec![0.0f64; 10];
            assert_eq!(
                ftp_get_stock_amort(std::ptr::null(), buf.as_mut_ptr(), 10),
                -1
            );

            let mut rows: i32 = 0;
            let mut cols: i32 = 0;
            assert_eq!(ftp_get_dims(std::ptr::null(), &mut rows, &mut cols), -1);
        }
    }

    #[test]
    fn test_invalid_method_returns_error() {
        unsafe {
            let outstanding = [1000.0f64];
            let profiles = [1.00, 0.50];
            let rates = [0.01];

            let h = ftp_create(
                outstanding.as_ptr(),
                1,
                profiles.as_ptr(),
                1,
                2,
                rates.as_ptr(),
                1,
                1,
            );
            assert!(!h.is_null());
            assert_eq!(ftp_compute(h, 99), -1);

            // Read the error
            let mut err_buf = vec![0i8; 256];
            ftp_get_last_error(err_buf.as_mut_ptr(), 256);
            let msg = std::ffi::CStr::from_ptr(err_buf.as_ptr()).to_string_lossy();
            assert!(msg.contains("unknown method"));

            ftp_free(h);
        }
    }

    #[test]
    fn test_buffer_too_small_returns_error() {
        unsafe {
            let outstanding = [1000.0f64];
            let profiles = [1.00, 0.50];
            let rates = [0.01];

            let h = ftp_create(
                outstanding.as_ptr(),
                1,
                profiles.as_ptr(),
                1,
                2,
                rates.as_ptr(),
                1,
                1,
            );
            ftp_compute(h, 0);

            // Buffer of 1 when we need 2
            let mut buf = vec![0.0f64; 1];
            assert_eq!(ftp_get_stock_amort(h, buf.as_mut_ptr(), 1), -1);

            ftp_free(h);
        }
    }

    #[test]
    fn test_get_before_compute_returns_error() {
        unsafe {
            let outstanding = [1000.0f64];
            let profiles = [1.00, 0.50];
            let rates = [0.01];

            let h = ftp_create(
                outstanding.as_ptr(),
                1,
                profiles.as_ptr(),
                1,
                2,
                rates.as_ptr(),
                1,
                1,
            );

            let mut buf = vec![0.0f64; 2];
            assert_eq!(ftp_get_stock_amort(h, buf.as_mut_ptr(), 2), -1);

            ftp_free(h);
        }
    }
}

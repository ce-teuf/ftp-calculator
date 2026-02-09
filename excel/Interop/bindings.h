#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// Opaque handle returned to C callers.
struct FtpHandle;

extern "C" {

/// Creates a new `FtpHandle` from raw row-major data.
///
/// - `outstanding`: pointer to `outs_rows` doubles (column vector)
/// - `profiles`:    pointer to `prof_rows * prof_cols` doubles (row-major)
/// - `rates`:       pointer to `rate_rows * rate_cols` doubles (row-major)
///
/// Returns a heap-allocated handle, or null on failure.
FtpHandle *ftp_create(const double *outstanding,
                      int32_t outs_rows,
                      const double *profiles,
                      int32_t prof_rows,
                      int32_t prof_cols,
                      const double *rates,
                      int32_t rate_rows,
                      int32_t rate_cols);

/// Frees an `FtpHandle`. No-op if `handle` is null.
void ftp_free(FtpHandle *handle);

/// Runs the FTP computation.
///
/// - `method`: 0 = Stock, 1 = Flux
///
/// Returns 0 on success, -1 on error (call `ftp_get_last_error`).
int32_t ftp_compute(FtpHandle *handle, int32_t method);

/// Writes the output matrix dimensions (rows, cols) into the provided pointers.
///
/// Returns 0 on success, -1 on error.
int32_t ftp_get_dims(const FtpHandle *handle, int32_t *out_rows, int32_t *out_cols);

/// Copies stock_amort into `out_buf`. Returns 0 on success, -1 on error.
int32_t ftp_get_stock_amort(const FtpHandle *handle, double *out_buf, int32_t buf_len);

/// Copies stock_instal into `out_buf`. Returns 0 on success, -1 on error.
int32_t ftp_get_stock_instal(const FtpHandle *handle, double *out_buf, int32_t buf_len);

/// Copies varstock_amort into `out_buf`. Returns 0 on success, -1 on error.
int32_t ftp_get_varstock_amort(const FtpHandle *handle, double *out_buf, int32_t buf_len);

/// Copies varstock_instal into `out_buf`. Returns 0 on success, -1 on error.
int32_t ftp_get_varstock_instal(const FtpHandle *handle, double *out_buf, int32_t buf_len);

/// Copies ftp_rate into `out_buf`. Returns 0 on success, -1 on error.
int32_t ftp_get_ftp_rate(const FtpHandle *handle, double *out_buf, int32_t buf_len);

/// Copies ftp_int into `out_buf`. Returns 0 on success, -1 on error.
int32_t ftp_get_ftp_int(const FtpHandle *handle, double *out_buf, int32_t buf_len);

/// Copies market_rate into `out_buf`. Returns 0 on success, -1 on error.
int32_t ftp_get_market_rate(const FtpHandle *handle, double *out_buf, int32_t buf_len);

/// Copies the last error message into `buf` (max `buf_len` bytes, NUL-terminated).
///
/// Returns 0 on success, -1 if `buf` is null or the message was truncated.
int32_t ftp_get_last_error(char *buf, int32_t buf_len);

}  // extern "C"

/// Interpolation methods for rate curves.
///
/// Given a curve defined at arbitrary tenor points, produce values
/// at the 12 standard FTP tenor points.
///
/// FTP standard tenors (months): 1,3,6,12,24,36,60,84,120,180,240,360

pub const FTP_MONTHS: [f64; 12] = [
    1.0, 3.0, 6.0, 12.0, 24.0, 36.0, 60.0, 84.0, 120.0, 180.0, 240.0, 360.0,
];

// ── Tenor string → months ─────────────────────────────────────────────────────

pub fn tenor_to_months(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() { return None; }

    if let Some(n) = s.strip_suffix('D').or_else(|| s.strip_suffix('d')) {
        return n.parse::<f64>().ok().map(|v| v / 30.0);
    }
    if let Some(n) = s.strip_suffix('W').or_else(|| s.strip_suffix('w')) {
        return n.parse::<f64>().ok().map(|v| v * 7.0 / 30.0);
    }
    if let Some(n) = s.strip_suffix('M').or_else(|| s.strip_suffix('m')) {
        return n.parse::<f64>().ok();
    }
    if let Some(n) = s.strip_suffix('Y').or_else(|| s.strip_suffix('y')) {
        return n.parse::<f64>().ok().map(|v| v * 12.0);
    }
    // Fallback: try parsing as plain number (assume months)
    s.parse::<f64>().ok()
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Interpolate curve values at FTP standard tenors.
///
/// `tenors_json`: JSON array of tenor strings (e.g. `["1M","3M","1Y"]`)
///                OR JSON array of numbers (months).
/// `values_json`: JSON array of f64, same length as tenors.
/// `method`:      "linear" | "cubic" | "flat_forward"
///
/// Returns 12 values at FTP_MONTHS. Falls back to linear on unknown method.
pub fn interpolate_to_ftp(
    tenors_json: &str,
    values_json: &str,
    method: &str,
) -> Result<[f64; 12], String> {
    // ── Parse curve definition ────────────────────────────────────────────────
    let raw_tenors: Vec<serde_json::Value> =
        serde_json::from_str(tenors_json).map_err(|e| format!("tenors JSON: {e}"))?;
    let raw_values: Vec<f64> =
        serde_json::from_str(values_json).map_err(|e| format!("values JSON: {e}"))?;

    if raw_tenors.len() != raw_values.len() || raw_tenors.is_empty() {
        return Err(format!(
            "tenor/value length mismatch ({} vs {})",
            raw_tenors.len(),
            raw_values.len()
        ));
    }

    // Convert tenors to months
    let mut xs: Vec<f64> = Vec::with_capacity(raw_tenors.len());
    let mut ys: Vec<f64> = Vec::with_capacity(raw_tenors.len());

    for (t, v) in raw_tenors.iter().zip(raw_values.iter()) {
        let m = match t {
            serde_json::Value::String(s) => tenor_to_months(s),
            serde_json::Value::Number(n) => n.as_f64(),
            _ => None,
        };
        if let Some(m) = m {
            xs.push(m);
            ys.push(*v);
        }
    }

    if xs.is_empty() {
        return Err("No valid tenor/value pairs".into());
    }

    // Sort by tenor
    let mut pairs: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let xs: Vec<f64> = pairs.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = pairs.iter().map(|p| p.1).collect();

    // ── Apply method ──────────────────────────────────────────────────────────
    let mut out = [0.0f64; 12];
    for (i, &q) in FTP_MONTHS.iter().enumerate() {
        out[i] = match method {
            "cubic"        => interp_cubic(&xs, &ys, q),
            "flat_forward" => interp_flat_forward(&xs, &ys, q),
            _              => interp_linear(&xs, &ys, q),   // default: linear
        };
    }
    Ok(out)
}

// ── Linear interpolation ──────────────────────────────────────────────────────

fn interp_linear(xs: &[f64], ys: &[f64], q: f64) -> f64 {
    let n = xs.len();
    if n == 1 { return ys[0]; }

    // Extrapolate flat outside range
    if q <= xs[0]       { return ys[0]; }
    if q >= xs[n - 1]   { return ys[n - 1]; }

    // Find bracket
    let idx = xs.partition_point(|&x| x <= q).saturating_sub(1).min(n - 2);
    let t = (q - xs[idx]) / (xs[idx + 1] - xs[idx]);
    ys[idx] + t * (ys[idx + 1] - ys[idx])
}

// ── Flat-forward (step function, use left neighbor) ───────────────────────────

fn interp_flat_forward(xs: &[f64], ys: &[f64], q: f64) -> f64 {
    let n = xs.len();
    if n == 1 { return ys[0]; }

    if q <= xs[0]     { return ys[0]; }
    if q >= xs[n - 1] { return ys[n - 1]; }

    // Use the value of the segment that starts at or before q
    let idx = xs.partition_point(|&x| x <= q).saturating_sub(1);
    ys[idx]
}

// ── Natural cubic spline ──────────────────────────────────────────────────────
//
// Classic tridiagonal algorithm (Thomas method).
// Natural boundary conditions: S''(x0) = S''(xn) = 0.

fn interp_cubic(xs: &[f64], ys: &[f64], q: f64) -> f64 {
    let n = xs.len();
    if n == 1 { return ys[0]; }
    if n == 2 { return interp_linear(xs, ys, q); }

    // Flat extrapolation outside range
    if q <= xs[0]     { return ys[0]; }
    if q >= xs[n - 1] { return ys[n - 1]; }

    // Build second derivatives via Thomas algorithm
    let m = n - 1; // number of intervals
    let mut h  = vec![0.0f64; m];
    for i in 0..m { h[i] = xs[i + 1] - xs[i]; }

    let mut alpha = vec![0.0f64; m];
    for i in 1..m {
        alpha[i] = 3.0 * (ys[i + 1] - ys[i]) / h[i]
                 - 3.0 * (ys[i]     - ys[i-1]) / h[i-1];
    }

    let mut l  = vec![1.0f64; n];
    let mut mu = vec![0.0f64; n];
    let mut z  = vec![0.0f64; n];

    for i in 1..m {
        l[i]  = 2.0 * (xs[i+1] - xs[i-1]) - h[i-1] * mu[i-1];
        if l[i].abs() < 1e-15 { l[i] = 1e-15; }
        mu[i] = h[i] / l[i];
        z[i]  = (alpha[i] - h[i-1] * z[i-1]) / l[i];
    }

    let mut c = vec![0.0f64; n]; // second derivatives
    // back-substitution (natural: c[n-1] = 0)
    for j in (0..m).rev() {
        c[j] = z[j] - mu[j] * c[j + 1];
    }

    // Find bracket
    let idx = xs.partition_point(|&x| x <= q).saturating_sub(1).min(m - 1);
    let dx = q - xs[idx];
    let hi = h[idx];

    let b = (ys[idx+1] - ys[idx]) / hi
          - hi * (2.0 * c[idx] + c[idx+1]) / 3.0;
    let d = (c[idx+1] - c[idx]) / (3.0 * hi);

    ys[idx] + b * dx + c[idx] * dx * dx + d * dx * dx * dx
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenor_parse() {
        assert_eq!(tenor_to_months("1M"),  Some(1.0));
        assert_eq!(tenor_to_months("6M"),  Some(6.0));
        assert_eq!(tenor_to_months("1Y"),  Some(12.0));
        assert_eq!(tenor_to_months("10Y"), Some(120.0));
        assert_eq!(tenor_to_months("30Y"), Some(360.0));
    }

    #[test]
    fn linear_at_knots() {
        // If query is exactly at a knot, all methods must return the knot value.
        let xs = vec![1.0, 12.0, 120.0];
        let ys = vec![0.03, 0.035, 0.04];
        assert!((interp_linear(&xs, &ys, 12.0) - 0.035).abs() < 1e-10);
        assert!((interp_flat_forward(&xs, &ys, 12.0) - 0.035).abs() < 1e-10);
        assert!((interp_cubic(&xs, &ys, 12.0) - 0.035).abs() < 1e-10);
    }

    #[test]
    fn ftp_interpolate_12_values() {
        // A curve defined at exactly the 12 FTP tenors → identity
        let tenors = r#"["1M","3M","6M","12M","24M","36M","60M","84M","120M","180M","240M","360M"]"#;
        let vals_vec: Vec<f64> = (0..12).map(|i| 0.02 + i as f64 * 0.002).collect();
        let vals = serde_json::to_string(&vals_vec).unwrap();
        let result = interpolate_to_ftp(tenors, &vals, "linear").unwrap();
        for (i, &v) in vals_vec.iter().enumerate() {
            assert!((result[i] - v).abs() < 1e-10, "i={i}: {:.6} vs {:.6}", result[i], v);
        }
    }
}

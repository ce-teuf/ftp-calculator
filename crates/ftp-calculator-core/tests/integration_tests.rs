use ftp_calculator_core::{ComputeMethod, FtpResult};
use ndarray::array;

#[test]
fn test_ftp_result_compute_stock_method() {
    let v_outstanding = array![[1000.0], [1200.0], [1350.0]];
    let m_profile = array![
        [1.00, 0.50, 0.20, 0.05],
        [1.00, 0.50, 0.20, 0.05],
        [1.00, 0.50, 0.20, 0.05]
    ];
    let m_taux = array![
        [0.01300, 0.01400, 0.01600],
        [0.01360, 0.01460, 0.01660],
        [0.01430, 0.01530, 0.01730]
    ];

    let mut ftp_res = FtpResult::new(v_outstanding, m_profile, m_taux);
    ftp_res.compute(ComputeMethod::Stock).unwrap();

    // Check outputs exist
    assert!(ftp_res.stock_amort().is_some());
    assert!(ftp_res.stock_instal().is_some());
    assert!(ftp_res.varstock_amort().is_some());
    assert!(ftp_res.ftp_rate().is_some());
    assert!(ftp_res.market_rate().is_some());

    let stock_amort = ftp_res.stock_amort().unwrap();
    assert_eq!(stock_amort.dim(), (3, 4));

    // Verify actual computed values (non-regression)
    assert_eq!(stock_amort[[0, 0]], 1000.0);
    assert_eq!(stock_amort[[0, 1]], 500.0);
    assert_eq!(stock_amort[[1, 0]], 1200.0);
    assert_eq!(stock_amort[[2, 3]], 67.5);

    let stock_instal = ftp_res.stock_instal().unwrap();
    assert_eq!(stock_instal[[0, 0]], 0.0);
    assert_eq!(stock_instal[[0, 1]], 500.0);
    assert_eq!(stock_instal[[0, 2]], 300.0);
    assert_eq!(stock_instal[[0, 3]], 150.0);

    let varstock_amort = ftp_res.varstock_amort().unwrap();
    assert_eq!(varstock_amort[[0, 0]], 1000.0);
    assert_eq!(varstock_amort[[1, 0]], 700.0);
    assert_eq!(varstock_amort[[1, 1]], 400.0);
    assert_eq!(varstock_amort[[2, 0]], 750.0);

    let varstock_instal = ftp_res.varstock_instal().unwrap();
    assert_eq!(varstock_instal[[1, 1]], 300.0);
    assert_eq!(varstock_instal[[1, 2]], 210.0);

    let ftp_rate = ftp_res.ftp_rate().unwrap();
    assert!((ftp_rate[[0, 0]] - 0.0137894737).abs() < 1e-8);
    assert!((ftp_rate[[0, 1]] - 0.0146666667).abs() < 1e-8);
    assert!((ftp_rate[[0, 2]] - 0.016).abs() < 1e-10);
    assert_eq!(ftp_rate[[0, 3]], 0.0);

    let ftp_int = ftp_res.ftp_int().unwrap();
    assert!((ftp_int[[0, 0]] - 1.0916666667).abs() < 1e-8);
    assert!((ftp_int[[0, 1]] - 0.55).abs() < 1e-10);
    assert!((ftp_int[[1, 0]] - 1.3253333333).abs() < 1e-8);

    let market_rate = ftp_res.market_rate().unwrap();
    assert!((market_rate[[0, 0]] - 0.0).abs() < 1e-10);
    assert!((market_rate[[0, 1]] - 0.013).abs() < 1e-10);
    assert!((market_rate[[0, 2]] - 0.014).abs() < 1e-10);
    assert!((market_rate[[0, 3]] - 0.016).abs() < 1e-10);
}

#[test]
fn test_ftp_result_compute_flux_method() {
    let v_outstanding = array![[800.0], [900.0]];
    let m_profile = array![[1.00, 0.60, 0.30], [1.00, 0.60, 0.30]];
    let m_taux = array![[0.01200, 0.01300], [0.01250, 0.01350]];

    let mut ftp_res = FtpResult::new(v_outstanding, m_profile, m_taux);
    ftp_res.compute(ComputeMethod::Flux).unwrap();

    assert!(ftp_res.varstock_amort().is_some());
    assert!(ftp_res.ftp_int().is_some());

    let varstock_amort = ftp_res.varstock_amort().unwrap();
    assert_eq!(varstock_amort.dim(), (2, 3));

    // Verify actual computed values (non-regression)
    assert_eq!(varstock_amort[[0, 0]], 800.0);
    assert_eq!(varstock_amort[[0, 1]], 480.0);
    assert_eq!(varstock_amort[[0, 2]], 240.0);
    assert_eq!(varstock_amort[[1, 0]], 420.0);
    assert_eq!(varstock_amort[[1, 1]], 252.0);
    assert_eq!(varstock_amort[[1, 2]], 126.0);

    let stock_amort = ftp_res.stock_amort().unwrap();
    assert_eq!(stock_amort[[0, 0]], 800.0);
    assert_eq!(stock_amort[[0, 1]], 480.0);
    assert_eq!(stock_amort[[1, 0]], 900.0);
    assert_eq!(stock_amort[[1, 1]], 492.0);
    assert_eq!(stock_amort[[1, 2]], 126.0);

    let stock_instal = ftp_res.stock_instal().unwrap();
    assert_eq!(stock_instal[[0, 1]], 320.0);
    assert_eq!(stock_instal[[1, 1]], 408.0);
    assert_eq!(stock_instal[[1, 2]], 366.0);

    let ftp_rate = ftp_res.ftp_rate().unwrap();
    assert!((ftp_rate[[0, 0]] - 0.0124285714).abs() < 1e-8);
    assert!((ftp_rate[[0, 1]] - 0.013).abs() < 1e-10);

    let ftp_int = ftp_res.ftp_int().unwrap();
    assert!((ftp_int[[0, 0]] - 0.58).abs() < 1e-10);
    assert!((ftp_int[[0, 1]] - 0.26).abs() < 1e-10);

    let market_rate = ftp_res.market_rate().unwrap();
    assert!((market_rate[[0, 1]] - 0.012).abs() < 1e-10);
    assert!((market_rate[[0, 2]] - 0.013).abs() < 1e-10);
    assert!((market_rate[[1, 1]] - 0.0124768672).abs() < 1e-8);
}

#[test]
fn test_ftp_result_invalid_dimensions() {
    let v_outstanding = array![[1000.0], [1200.0]]; // 2 rows
    let m_profile = array![[1.00, 0.50]]; // 1 row
    let m_taux = array![[0.01300]]; // 1 row

    let mut ftp_res = FtpResult::new(v_outstanding, m_profile, m_taux);
    let result = ftp_res.compute(ComputeMethod::Stock);
    assert!(result.is_err());
}

#[test]
fn test_ftp_result_invalid_outstanding_columns() {
    let v_outstanding = array![[1000.0, 2000.0]]; // 2 cols
    let m_profile = array![[1.0, 0.5]];
    let m_taux = array![[0.01]];

    let mut ftp_res = FtpResult::new(v_outstanding, m_profile, m_taux);
    let result = ftp_res.compute(ComputeMethod::Stock);
    assert!(result.is_err());
}

#[test]
fn test_ftp_result_invalid_rate_columns() {
    let v_outstanding = array![[1000.0]];
    let m_profile = array![[1.0, 0.5, 0.2]];
    let m_taux = array![[0.01]]; // should be 2 cols

    let mut ftp_res = FtpResult::new(v_outstanding, m_profile, m_taux);
    let result = ftp_res.compute(ComputeMethod::Stock);
    assert!(result.is_err());
}

#[test]
fn test_ftp_result_new_creation() {
    let v_outstanding = array![[1000.0]];
    let m_profile = array![[1.00, 0.50]];
    let m_taux = array![[0.01300]];

    let ftp_res = FtpResult::new(v_outstanding.clone(), m_profile.clone(), m_taux.clone());

    assert_eq!(ftp_res.input_outstanding().dim(), (1, 1));
    assert_eq!(ftp_res.input_profiles().dim(), (1, 2));
    assert_eq!(ftp_res.input_rate().dim(), (1, 1));
    assert!(ftp_res.stock_amort().is_none());
    assert!(ftp_res.ftp_rate().is_none());
}

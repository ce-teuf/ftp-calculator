#%%
import numpy as np
import pytest

from ftp_calculator import FtpCalculator, compute_stock, compute_flux

#%%
# --- Reference data (same as integration_tests.rs) ---

STOCK_OUTSTANDING = np.array([[1000.0], [1200.0], [1350.0]])
STOCK_PROFILES = np.array([
    [1.00, 0.50, 0.20, 0.05],
    [1.00, 0.50, 0.20, 0.05],
    [1.00, 0.50, 0.20, 0.05],
])
STOCK_RATES = np.array([
    [0.01300, 0.01400, 0.01600],
    [0.01360, 0.01460, 0.01660],
    [0.01430, 0.01530, 0.01730],
])

FLUX_OUTSTANDING = np.array([[800.0], [900.0]])
FLUX_PROFILES = np.array([[1.00, 0.60, 0.30], [1.00, 0.60, 0.30]])
FLUX_RATES = np.array([[0.01200, 0.01300], [0.01250, 0.01350]])


class TestComputeStock:
    """Stock method — values match integration_tests.rs."""

    def test_stock_amort(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        sa = calc.stock_amort
        assert sa.shape == (3, 4)
        assert sa[0, 0] == 1000.0
        assert sa[0, 1] == 500.0
        assert sa[1, 0] == 1200.0
        assert sa[2, 3] == 67.5

    def test_stock_instal(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        si = calc.stock_instal
        assert si[0, 0] == 0.0
        assert si[0, 1] == 500.0
        assert si[0, 2] == 300.0
        assert si[0, 3] == 150.0

    def test_varstock_amort(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        va = calc.varstock_amort
        assert va[0, 0] == 1000.0
        assert va[1, 0] == 700.0
        assert va[1, 1] == 400.0
        assert va[2, 0] == 750.0

    def test_varstock_instal(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        vi = calc.varstock_instal
        assert vi[1, 1] == 300.0
        assert vi[1, 2] == 210.0

    def test_ftp_rate(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        fr = calc.ftp_rate
        assert abs(fr[0, 0] - 0.0137894737) < 1e-8
        assert abs(fr[0, 1] - 0.0146666667) < 1e-8
        assert abs(fr[0, 2] - 0.016) < 1e-10
        assert fr[0, 3] == 0.0

    def test_ftp_int(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        fi = calc.ftp_int
        assert abs(fi[0, 0] - 1.0916666667) < 1e-8
        assert abs(fi[0, 1] - 0.55) < 1e-10
        assert abs(fi[1, 0] - 1.3253333333) < 1e-8

    def test_market_rate(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        mr = calc.market_rate
        assert abs(mr[0, 0] - 0.0) < 1e-10
        assert abs(mr[0, 1] - 0.013) < 1e-10
        assert abs(mr[0, 2] - 0.014) < 1e-10
        assert abs(mr[0, 3] - 0.016) < 1e-10


class TestComputeFlux:
    """Flux method — values match integration_tests.rs."""

    def test_varstock_amort(self):
        calc = FtpCalculator(FLUX_OUTSTANDING, FLUX_PROFILES, FLUX_RATES)
        calc.compute("flux")
        va = calc.varstock_amort
        assert va.shape == (2, 3)
        assert va[0, 0] == 800.0
        assert va[0, 1] == 480.0
        assert va[0, 2] == 240.0
        assert va[1, 0] == 420.0
        assert va[1, 1] == 252.0
        assert va[1, 2] == 126.0

    def test_stock_amort(self):
        calc = FtpCalculator(FLUX_OUTSTANDING, FLUX_PROFILES, FLUX_RATES)
        calc.compute("flux")
        sa = calc.stock_amort
        assert sa[0, 0] == 800.0
        assert sa[0, 1] == 480.0
        assert sa[1, 0] == 900.0
        assert sa[1, 1] == 492.0
        assert sa[1, 2] == 126.0

    def test_stock_instal(self):
        calc = FtpCalculator(FLUX_OUTSTANDING, FLUX_PROFILES, FLUX_RATES)
        calc.compute("flux")
        si = calc.stock_instal
        assert si[0, 1] == 320.0
        assert si[1, 1] == 408.0
        assert si[1, 2] == 366.0

    def test_ftp_rate(self):
        calc = FtpCalculator(FLUX_OUTSTANDING, FLUX_PROFILES, FLUX_RATES)
        calc.compute("flux")
        fr = calc.ftp_rate
        assert abs(fr[0, 0] - 0.0124285714) < 1e-8
        assert abs(fr[0, 1] - 0.013) < 1e-10

    def test_ftp_int(self):
        calc = FtpCalculator(FLUX_OUTSTANDING, FLUX_PROFILES, FLUX_RATES)
        calc.compute("flux")
        fi = calc.ftp_int
        assert abs(fi[0, 0] - 0.58) < 1e-10
        assert abs(fi[0, 1] - 0.26) < 1e-10

    def test_market_rate(self):
        calc = FtpCalculator(FLUX_OUTSTANDING, FLUX_PROFILES, FLUX_RATES)
        calc.compute("flux")
        mr = calc.market_rate
        assert abs(mr[0, 1] - 0.012) < 1e-10
        assert abs(mr[0, 2] - 0.013) < 1e-10
        assert abs(mr[1, 1] - 0.0124768672) < 1e-8


class TestCalculatorClass:
    """FtpCalculator API: init, dims, repr."""

    def test_dims(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        assert calc.dims == (3, 4)

    def test_repr_before_compute(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        assert "computed=false" in repr(calc).lower()

    def test_repr_after_compute(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        assert "computed=true" in repr(calc).lower()

    def test_outputs_are_numpy(self):
        calc = FtpCalculator(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        calc.compute("stock")
        assert isinstance(calc.stock_amort, np.ndarray)
        assert calc.stock_amort.dtype == np.float64


class TestOneShotFunctions:
    """compute_stock / compute_flux convenience functions."""

    def test_compute_stock_returns_dict(self):
        result = compute_stock(STOCK_OUTSTANDING, STOCK_PROFILES, STOCK_RATES)
        assert isinstance(result, dict)
        expected_keys = {
            "stock_amort", "stock_instal", "varstock_amort", "varstock_instal",
            "ftp_rate", "ftp_int", "market_rate",
        }
        assert set(result.keys()) == expected_keys
        assert result["stock_amort"][0, 0] == 1000.0

    def test_compute_flux_returns_dict(self):
        result = compute_flux(FLUX_OUTSTANDING, FLUX_PROFILES, FLUX_RATES)
        assert isinstance(result, dict)
        assert result["varstock_amort"][0, 0] == 800.0


class TestErrorHandling:
    """Validation errors should raise ValueError."""

    def test_invalid_dimensions(self):
        outstanding = np.array([[1000.0], [1200.0]])  # 2 rows
        profiles = np.array([[1.00, 0.50]])             # 1 row
        rates = np.array([[0.01300]])                    # 1 row
        calc = FtpCalculator(outstanding, profiles, rates)
        with pytest.raises(ValueError):
            calc.compute("stock")

    def test_invalid_method(self):
        calc = FtpCalculator(
            np.array([[1000.0]]),
            np.array([[1.0, 0.5]]),
            np.array([[0.01]]),
        )
        with pytest.raises(ValueError, match="unknown method"):
            calc.compute("invalid")

    def test_get_before_compute(self):
        calc = FtpCalculator(
            np.array([[1000.0]]),
            np.array([[1.0, 0.5]]),
            np.array([[0.01]]),
        )
        with pytest.raises(ValueError, match="compute"):
            _ = calc.stock_amort

    def test_oneshot_invalid_dimensions(self):
        outstanding = np.array([[1000.0], [1200.0]])
        profiles = np.array([[1.00, 0.50]])
        rates = np.array([[0.01300]])
        with pytest.raises(ValueError):
            compute_stock(outstanding, profiles, rates)

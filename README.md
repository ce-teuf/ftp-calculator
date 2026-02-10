# FTP Calculator

A cross-language **Funds Transfer Pricing (FTP)** computation library. FTP is a methodology used by banks and financial institutions to allocate internal interest rates and measure profitability across business units based on their funding activities.

The core engine is written in **Rust** and exposed via **Python** (PyO3) and **C** bindings (for Excel integration).

## Features

- Two calculation methods: **Stock** and **Flux**
- Matrix-based computation using `ndarray`
- Python package with high-level wrappers
- Excel Add-In (.xll) with native performance
- Computes FTP rates, market rates, stock amortization, installments, and FTP interest

---

## 🚀 Quick Start

### Python (End Users)

**Install from PyPI:**

```bash
pip install ftp-calculator
```

**Usage:**

```python
import numpy as np
from ftp_core import FtpCalculator, compute_stock

outstanding = np.array([[1000.0], [1200.0], [1350.0]])
profiles = np.array([
    [1.00, 0.50, 0.20, 0.05],
    [1.00, 0.50, 0.20, 0.05],
    [1.00, 0.50, 0.20, 0.05],
])
rates = np.array([
    [0.01300, 0.01400, 0.01600],
    [0.01360, 0.01460, 0.01660],
    [0.01430, 0.01530, 0.01730],
])

# Class-based API
calc = FtpCalculator(outstanding, profiles, rates)
calc.compute("stock")  # or "flux"
print(calc.ftp_rate)   # numpy 2D array
print(calc.ftp_int)

# One-shot API (returns a dict of numpy arrays)
result = compute_stock(outstanding, profiles, rates)
print(result["ftp_rate"])
```

### Excel (End Users)

**Download the add-in:**

1. Go to the [Releases page](https://github.com/ce-teuf/FTP_CALCULATOR/releases)
2. Download the latest `ftp_calculator-vX.X.X-AddIn64.xll` file
3. Open it in Excel — a custom **FTP** tab will appear in the ribbon

**Worksheet Functions:**

All functions are array formulas. Select the output range, type the formula, and press **Ctrl+Shift+Enter**.

**Full compute (all 7 outputs stacked vertically):**

```excel
=FTP_COMPUTE_STOCK(outstanding, profiles, rates)
=FTP_COMPUTE_FLUX(outstanding, profiles, rates)
```

Returns 7 labeled blocks (stock_amort, stock_instal, varstock_amort, varstock_instal, ftp_rate, ftp_int, market_rate) stacked vertically. Select a range of `7 * (rows + 1)` rows by `cols` columns.

**Individual output matrices:**

```excel
=FTP_STOCK_AMORT(outstanding, profiles, rates, method)
=FTP_STOCK_INSTAL(outstanding, profiles, rates, method)
=FTP_VARSTOCK_AMORT(outstanding, profiles, rates, method)
=FTP_VARSTOCK_INSTAL(outstanding, profiles, rates, method)
=FTP_FTP_RATE(outstanding, profiles, rates, method)
=FTP_FTP_INT(outstanding, profiles, rates, method)
=FTP_MARKET_RATE(outstanding, profiles, rates, method)
```

Where `method` is `0` for Stock, `1` for Flux.

**Example:**

Given the following data in Excel:

| | A (outstanding) | B:E (profiles) | | | | F:H (rates) | | |
|---|---|---|---|---|---|---|---|---|
| 1 | 1000 | 1.00 | 0.50 | 0.20 | 0.05 | 0.013 | 0.014 | 0.016 |
| 2 | 1200 | 1.00 | 0.50 | 0.20 | 0.05 | 0.0136 | 0.0146 | 0.0166 |
| 3 | 1350 | 1.00 | 0.50 | 0.20 | 0.05 | 0.0143 | 0.0153 | 0.0173 |

To get the FTP rate matrix, select a 3x4 range and enter:

```excel
=FTP_FTP_RATE(A1:A3, B1:E3, F1:H3, 0)
```

Errors are returned as `#ERR: <message>` strings in the first cell.

---

## 🛠️ Development Setup

### Prerequisites

- **Rust** (edition 2021)
- **Python 3.9+**
- **.NET SDK 6.0** (for Excel Add-In, Windows only)

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/ce-teuf/FTP_CALCULATOR.git
cd FTP_CALCULATOR

# Create and activate Python virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install development tools
make setup-dev
```

### Build

```bash
# Build all bindings (C, Python, Excel)
make build-all

# Build individually
make build-c-bindings      # C bindings (for Excel)
make build-py-bindings     # Python bindings
make build-excel-addin     # Excel add-in (Windows only)
```

### Test

```bash
# Run all tests
make test

# Unit tests only
make unit

# Integration tests only
make integration

# Tests with output
make detailed

# Code coverage
make tarpaulin
```

### Python Development Workflow

```bash
# Install package in editable mode with dependencies
cd python-lib
pip install -e ".[dev]"

# Or use maturin for development (faster iteration)
cd ../crates/ftp_core_bindings_pyo3
maturin develop
```

### Excel Development Workflow (Windows)

```bash
# Build C bindings first
make build-c-bindings

# Build Excel add-in
make build-excel-addin

# The .xll file will be in:
# excel-addin/ftp_addin/bin/Release/net6.0-windows/ftp_addin-AddIn64-packed.xll
```

---

## 📦 Project Structure

```
ftp-calculator/
├── crates/
│   ├── ftp_core/                  # Core Rust calculation library
│   ├── ftp_core_bindings_c/       # C bindings (for Excel)
│   └── ftp_core_bindings_pyo3/    # Python bindings (PyO3)
├── python-lib/                    # Python wrapper package (published to PyPI)
├── excel-addin/                   # .NET Excel Add-In
├── docs/                          # MkDocs documentation site
├── scripts/                       # Release management
├── .github/workflows/             # CI/CD pipelines
├── .venv/                         # Python virtual environment
├── Makefile                       # Build orchestration
└── ftp-core-test.ods              # Test data spreadsheet
```

---

## 📖 Core Concepts

The central computation takes three input matrices:

| Input | Shape | Description |
|---|---|---|
| `outstanding` | (n, 1) | Loan/position amounts |
| `profiles` | (n, m) | Repricing profiles |
| `rates` | (n, m-1) | Market rates |

And produces:

| Output | Description |
|---|---|
| `stock_amort` | Amortized stock |
| `stock_instal` | Stock installments |
| `varstock_amort` | Variable stock (amortized) |
| `varstock_instal` | Variable stock (installments) |
| `ftp_rate` | Calculated FTP rate |
| `ftp_int` | FTP interest (monthly) |
| `market_rate` | Market rate |

### Stock vs Flux Method

- **Stock method** — sequential calculation using anti-diagonal matrix extraction for variable stock
- **Flux method** — alternative approach using `max_zero` clamping for profile-based variable stock calculation

---

## 🔄 CI/CD

### Local CI Simulation

```bash
make ci
```

### Releases

Releases are managed via GitHub Actions. When a tag is pushed:

1. **Python package** is published to PyPI
2. **Rust crates** are published to crates.io
3. **Excel add-in** (.xll) is attached to the GitHub Release
4. **Documentation** is deployed to GitHub Pages

**Create a release:**

```bash
# Prepare and validate
make release-prepare

# Bump version (patch, minor, or major)
make release-bump-patch
make release-bump-minor
make release-bump-major
```

The script will:
- Update versions in all `Cargo.toml` and `pyproject.toml` files
- Run validation checks (format, lint, build)
- Create a git tag
- Push the tag to trigger GitHub Actions

---

## 📚 Documentation

```bash
# Serve docs locally
make serve-docs

# Build docs
make build-docs
```

Published at: https://ce-teuf.github.io/FTP_CALCULATOR

---

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📝 License

MIT OR Apache-2.0

## 👤 Author

Charles Teuf

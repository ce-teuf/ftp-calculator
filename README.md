# FTP Calculator

A cross-language **Funds Transfer Pricing (FTP)** computation library. FTP is a methodology used by banks and financial institutions to allocate internal interest rates and measure profitability across business units based on their funding activities.

The core engine is written in **Rust** and exposed via **Python** (PyO3) and **C** bindings (for Excel integration).

## Features

- Two calculation methods: **Stock** and **Flux**
- Matrix-based computation using `ndarray`
- Python package with high-level wrappers
- C FFI for Excel Add-In integration (.NET)
- Computes FTP rates, market rates, stock amortization, installments, and FTP interest

## Project Structure

```
ftp-calculator/
├── crates/
│   ├── ftp_core/                  # Core Rust calculation library
│   ├── ftp_core_bindings_c/       # C bindings (for Excel)
│   └── ftp_core_bindings_pyo3/    # Python bindings (PyO3)
├── python/                        # Python wrapper package
├── excel/                         # .NET Excel Add-In
├── docs/                          # MkDocs documentation site
├── scripts/                       # Release management
├── .github/workflows/             # CI/CD pipelines
├── Makefile                       # Build orchestration
└── ftp-core-test.ods              # Test data spreadsheet
```

## Prerequisites

- **Rust** (edition 2021)
- **Python 3** and **Maturin** (for Python bindings)
- **.NET SDK** (for Excel Add-In, Windows only)
- **MkDocs** with Material theme (for documentation)

Install development tools:

```bash
make setup-dev
```

## Build

```bash
# Build all bindings (C, Python, Excel)
make build-all

# Build individually
make build-c-bindings
make build-py-bindings
make build-excel
```

## Test

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

## Python Usage

```bash
pip install -e python/
```

```python
from ftp_core import calculer

result = calculer(outstanding, profiles, rates, method="stock")
```

## Documentation

```bash
# Serve docs locally
make serve-docs

# Build docs
make build-docs
```

Published at: https://ce-teuf.github.io/FTP_CALCULATOR

## Core Concepts

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

## CI

```bash
# Run full CI pipeline locally
make ci
```

## License

MIT OR Apache-2.0

## Author

Charles Teuf

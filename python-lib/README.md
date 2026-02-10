# ftp-calculator

Python bindings for the FTP (Funds Transfer Pricing) calculator core engine written in Rust.

## Installation

```bash
pip install ftp-calculator
```

## Usage

```python
import numpy as np
from ftp_core import FtpCalculator, compute_stock, compute_flux

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

# One-shot functional API
result = compute_stock(outstanding, profiles, rates)
print(result["ftp_rate"])
print(result["ftp_int"])
```

## API

### `FtpCalculator(outstanding, profiles, rates)`

Create a calculator instance with input matrices.

**Parameters:**
- `outstanding`: numpy array of shape (n, 1) - loan/position amounts
- `profiles`: numpy array of shape (n, m) - repricing profiles
- `rates`: numpy array of shape (n, m-1) - market rates

**Methods:**
- `compute(method)`: Run computation using "stock" or "flux" method

**Properties (available after compute):**
- `stock_amort`: Amortized stock
- `stock_instal`: Stock installments
- `varstock_amort`: Variable stock (amortized)
- `varstock_instal`: Variable stock (installments)
- `ftp_rate`: Calculated FTP rate
- `ftp_int`: FTP interest (monthly)
- `market_rate`: Market rate

### `compute_stock(outstanding, profiles, rates)`

One-shot computation using the stock method.

**Returns:** Dictionary with all output matrices as numpy arrays.

### `compute_flux(outstanding, profiles, rates)`

One-shot computation using the flux method.

**Returns:** Dictionary with all output matrices as numpy arrays.

## Development

This package is part of the [FTP Calculator](https://github.com/ce-teuf/FTP_CALCULATOR) project.

See the main repository for build instructions and documentation.

## License

MIT OR Apache-2.0

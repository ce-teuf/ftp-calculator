"""FTP Core — Funds Transfer Pricing calculator powered by Rust."""

from ftp_core._core import FtpCalculator, compute_stock, compute_flux

__all__ = ["FtpCalculator", "compute_stock", "compute_flux"]
__version__ = "0.1.0"

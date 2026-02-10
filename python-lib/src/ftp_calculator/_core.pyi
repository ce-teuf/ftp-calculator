import numpy as np
import numpy.typing as npt

class FtpCalculator:
    """FTP Calculator — wraps the Rust ftp_core engine."""

    def __init__(
        self,
        outstanding: npt.NDArray[np.float64],
        profiles: npt.NDArray[np.float64],
        rates: npt.NDArray[np.float64],
    ) -> None: ...
    def compute(self, method: str) -> None:
        """Run FTP computation. method must be 'stock' or 'flux'."""
        ...
    @property
    def dims(self) -> tuple[int, int]: ...
    @property
    def stock_amort(self) -> npt.NDArray[np.float64]: ...
    @property
    def stock_instal(self) -> npt.NDArray[np.float64]: ...
    @property
    def varstock_amort(self) -> npt.NDArray[np.float64]: ...
    @property
    def varstock_instal(self) -> npt.NDArray[np.float64]: ...
    @property
    def ftp_rate(self) -> npt.NDArray[np.float64]: ...
    @property
    def ftp_int(self) -> npt.NDArray[np.float64]: ...
    @property
    def market_rate(self) -> npt.NDArray[np.float64]: ...
    def __repr__(self) -> str: ...

def compute_stock(
    outstanding: npt.NDArray[np.float64],
    profiles: npt.NDArray[np.float64],
    rates: npt.NDArray[np.float64],
) -> dict[str, npt.NDArray[np.float64]]:
    """Compute FTP using the stock method. Returns a dict of numpy arrays."""
    ...

def compute_flux(
    outstanding: npt.NDArray[np.float64],
    profiles: npt.NDArray[np.float64],
    rates: npt.NDArray[np.float64],
) -> dict[str, npt.NDArray[np.float64]]:
    """Compute FTP using the flux method. Returns a dict of numpy arrays."""
    ...

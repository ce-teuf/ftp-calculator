#!/usr/bin/env python3
"""
generate_ftp_spreads.py
Generates realistic synthetic historical spread series for FTP component construction:

  USD_CREDIT_SPREAD  — Component 2 USD: G-SIB senior unsecured bond z-spread over SOFR OIS
  EUR_CREDIT_SPREAD  — Component 2 EUR: EU G-SIB senior unsecured z-spread over €STR OIS
  USD_TLP            — Component 3 USD: FHLB advance spread over SOFR OIS (term liquidity premium)
  EUR_TLP            — Component 3 EUR: Covered bond / SNP spread over €STR OIS
  XCCY_EUR_USD       — Component 7B:    EUR/USD cross-currency basis (negative = EUR expensive)

All spreads stored as DECIMAL fractions (0.0040 = 40 bps). Daily, weekdays only.
Coverage: 2014-01-01 → today (~11 years).

Regime calibration:
  Credit spreads: QE compression, oil/China stress 2015-16, COVID spike 2020, hiking widening 2022-23
  TLP (USD):      Low post-GFC, widens with QT/hiking cycle, always upward-sloping
  TLP (EUR):      Positive even in negative rate era, reflects covered bond/SNP market
  XCCY basis:     Structural negative, spikes with USD funding stress (2016, 2020)
"""

from __future__ import annotations

import csv
import math
import random
from datetime import date, timedelta
from os import path

OUTPUT_DIR = path.dirname(path.abspath(__file__))
random.seed(7)   # different seed from generate_rate_series.py

START_DATE = date(2014, 1, 1)
END_DATE   = date.today()


# ── Tenor → years ─────────────────────────────────────────────────────────────

def tenor_years(t: str) -> float:
    if t.endswith("D"):  return int(t[:-1]) / 365
    if t.endswith("W"):  return int(t[:-1]) / 52
    if t.endswith("M"):  return int(t[:-1]) / 12
    if t.endswith("Y"):  return float(t[:-1])
    return 0.0


# ── Piecewise-linear interpolation ────────────────────────────────────────────

def _lerp(t: date, segments: list[tuple[date, float]]) -> float:
    if t <= segments[0][0]:  return segments[0][1]
    if t >= segments[-1][0]: return segments[-1][1]
    for i in range(1, len(segments)):
        d0, r0 = segments[i - 1]
        d1, r1 = segments[i]
        if d0 <= t <= d1:
            frac = (t - d0).days / max((d1 - d0).days, 1)
            return r0 + frac * (r1 - r0)
    return segments[-1][1]


# ══════════════════════════════════════════════════════════════════════════════
#  COMPONENT 2 — CREDIT SPREAD
#  Represents the bank's own wholesale funding cost above the risk-free rate.
#  Source: z-spread on senior unsecured bonds / AXI index.
#  Term structure: upward sloping (longer = more credit risk premium).
# ══════════════════════════════════════════════════════════════════════════════

# USD credit spread level (5Y anchor, in decimal — rest scaled by tenor factor)
_USD_CS_LEVEL: list[tuple[date, float]] = [
    (date(2014,  1,  1), 0.0038),   # 38bp — post-GFC normal
    (date(2014, 10,  1), 0.0042),   # Oct 2014 brief volatility
    (date(2015,  6,  1), 0.0035),   # Tightening on strong economy
    (date(2015, 12,  1), 0.0055),   # Oil / HY stress, energy contagion
    (date(2016,  2,  1), 0.0065),   # Peak 2016 stress (DB, oil)
    (date(2016,  7,  1), 0.0048),   # Recovery post-Brexit
    (date(2017,  6,  1), 0.0032),   # Tight: tax reform, strong growth
    (date(2018,  4,  1), 0.0038),   # Mild widening on rate vol
    (date(2018, 12,  1), 0.0055),   # Q4 2018 equity selloff / inversion
    (date(2019,  6,  1), 0.0040),   # Easing fears, tightening
    (date(2020,  2,  1), 0.0032),   # Pre-COVID tight
    (date(2020,  3, 23), 0.0140),   # COVID spike peak
    (date(2020,  6,  1), 0.0065),   # Rapid recovery on QE
    (date(2021,  1,  1), 0.0038),   # QE fully in, very tight
    (date(2021,  6,  1), 0.0032),   # Tightest post-GFC
    (date(2022,  3,  1), 0.0048),   # Rate shock widening
    (date(2022,  6,  1), 0.0062),   # Peak 2022 widening
    (date(2023,  3,  1), 0.0075),   # SVB/regional bank stress
    (date(2023,  6,  1), 0.0060),   # Recovery
    (date(2024,  1,  1), 0.0048),   # Soft landing expectations
    (date(2024,  6,  1), 0.0042),   # Tightening further
    (END_DATE,           0.0045),
]

# EUR credit spread level (5Y anchor)
_EUR_CS_LEVEL: list[tuple[date, float]] = [
    (date(2014,  1,  1), 0.0055),   # 55bp — periphery risk premium
    (date(2015,  1,  1), 0.0045),   # ECB QE compression
    (date(2016,  2,  1), 0.0075),   # Deutsche Bank fears, Italy
    (date(2016,  6,  1), 0.0085),   # Brexit shock
    (date(2017,  1,  1), 0.0060),   # Recovery, Macron elected
    (date(2018,  6,  1), 0.0050),   # Tight on growth
    (date(2018, 12,  1), 0.0070),   # Italy budget crisis + global selloff
    (date(2019,  6,  1), 0.0052),   # ECB restart QE
    (date(2020,  2,  1), 0.0040),   # Pre-COVID tight
    (date(2020,  3, 23), 0.0160),   # COVID spike (wider than USD)
    (date(2020,  6,  1), 0.0080),   # PEPP recovery
    (date(2021,  1,  1), 0.0050),   # Vaccines, tight
    (date(2021,  6,  1), 0.0040),   # Very tight, ample liquidity
    (date(2022,  3,  1), 0.0060),   # Ukraine + rate shock
    (date(2022,  7,  1), 0.0080),   # ECB hiking, fragmentation
    (date(2022, 10,  1), 0.0095),   # Energy crisis peak
    (date(2023,  3,  1), 0.0085),   # CS/AT1 concerns
    (date(2023,  7,  1), 0.0065),   # Recovery
    (date(2024,  1,  1), 0.0055),   # ECB cutting expectations
    (date(2024,  6,  1), 0.0048),
    (END_DATE,           0.0050),
]


def _cs_tenor_factor(T: float) -> float:
    """Scale credit spread from 5Y anchor. Upward sloping, flattens long end."""
    # 5Y anchor = 1.0; short end < 1.0; long end > 1.0
    # e.g. 1Y = 0.55, 5Y = 1.0, 10Y = 1.35, 30Y = 1.65
    if T <= 0:   return 0.50
    raw = 0.40 + 0.60 * math.log1p(T) / math.log1p(5)
    return min(raw, 1.80)


def _cs_vol(T: float) -> float:
    """Daily vol for credit spread (mean-reverting)."""
    if T < 2:   return 0.0004
    if T < 5:   return 0.0003
    return 0.0002


# ══════════════════════════════════════════════════════════════════════════════
#  COMPONENT 3 — TERM LIQUIDITY PREMIUM (TLP)
#  FHLB advance spread over SOFR OIS (USD) / covered bond spread (EUR).
#  Always positive, upward-sloping, widens during stress / QT cycles.
# ══════════════════════════════════════════════════════════════════════════════

# USD TLP level (5Y anchor, in decimal)
_USD_TLP_LEVEL: list[tuple[date, float]] = [
    (date(2014,  1,  1), 0.0030),   # 30bp — post-GFC elevated
    (date(2015,  6,  1), 0.0025),   # Steady
    (date(2016,  2,  1), 0.0040),   # Oil/HY stress widens TLP
    (date(2016, 10,  1), 0.0030),   # Normalises
    (date(2017,  6,  1), 0.0025),   # Low vol, tight
    (date(2018,  6,  1), 0.0028),   # Modest widening QT begins
    (date(2019,  6,  1), 0.0022),   # Cuts / easing
    (date(2020,  3, 16), 0.0060),   # COVID liquidity spike
    (date(2020,  6,  1), 0.0030),   # Fed backstop, tight
    (date(2021,  1,  1), 0.0015),   # QE peak, extremely compressed
    (date(2021,  6,  1), 0.0012),   # Tightest in modern history
    (date(2022,  3,  1), 0.0018),   # Hiking starts
    (date(2022,  9,  1), 0.0035),   # QT accelerates
    (date(2023,  3,  1), 0.0050),   # Regional bank stress (FHLB demand spike)
    (date(2023,  9,  1), 0.0042),   # Moderate
    (date(2024,  3,  1), 0.0038),   # Cuts begin
    (date(2024,  9,  1), 0.0032),
    (END_DATE,           0.0030),
]

# EUR TLP level (5Y anchor — covered bond / SNP spread over ESTR)
_EUR_TLP_LEVEL: list[tuple[date, float]] = [
    (date(2014,  1,  1), 0.0040),   # 40bp — covered bond spread
    (date(2015,  3,  1), 0.0030),   # ECB CBPP3 compresses covered bond spreads
    (date(2016,  2,  1), 0.0045),   # DB / Italy stress
    (date(2016,  9,  1), 0.0035),   # CSPP includes corporates, further compression
    (date(2018,  6,  1), 0.0030),   # QE tapering, modest
    (date(2019,  6,  1), 0.0025),   # ECB restarts easing
    (date(2020,  3, 20), 0.0070),   # COVID spike
    (date(2020,  7,  1), 0.0035),   # PEPP recovery
    (date(2021,  1,  1), 0.0018),   # Very compressed
    (date(2021,  6,  1), 0.0015),   # Tightest
    (date(2022,  3,  1), 0.0022),   # ECB hiking signals
    (date(2022,  6,  1), 0.0040),   # QT + fragmentation risk
    (date(2022, 10,  1), 0.0055),   # Energy crisis peak
    (date(2023,  6,  1), 0.0042),   # Recovery
    (date(2024,  1,  1), 0.0035),
    (date(2024,  6,  1), 0.0030),
    (END_DATE,           0.0028),
]


def _tlp_tenor_shape(T: float) -> float:
    """
    TLP always monotonically increasing with tenor (no inversion).
    3M anchor = 0.35 × 5Y; 5Y = 1.0; 10Y = 1.55; 30Y = 2.20
    Uses log-linear shape reflecting liquidity premium convexity.
    """
    if T <= 0.25:  return 0.35
    # log-linear from 3M → 30Y
    log_ref = math.log(5)                      # 5Y = reference (1.0)
    log_t   = math.log(max(T, 0.25))
    factor  = 0.35 + 0.65 * log_t / log_ref
    return max(0.30, min(factor, 2.30))


def _tlp_vol(T: float) -> float:
    if T < 1:   return 0.0002
    if T < 5:   return 0.00015
    return 0.0001


# ══════════════════════════════════════════════════════════════════════════════
#  COMPONENT 7B — EUR/USD CROSS-CURRENCY BASIS
#  The XCCY basis = EUR/USD swap spread. Negative means EUR is expensive to
#  access with USD funding. Tenor-structured, daily.
# ══════════════════════════════════════════════════════════════════════════════

# 5Y EUR/USD XCCY basis (decimal, typically negative)
_XCCY_LEVEL: list[tuple[date, float]] = [
    (date(2014,  1,  1), -0.0015),   # -15bp — mild basis
    (date(2015,  6,  1), -0.0018),   # Dollar strength
    (date(2016,  6,  1), -0.0035),   # Brexit shock
    (date(2016, 12,  1), -0.0050),   # EUR demand spike end of year
    (date(2017,  6,  1), -0.0040),   # Some normalisation
    (date(2018,  6,  1), -0.0030),   # Fed hiking, USD abundant
    (date(2019,  9,  1), -0.0025),   # Repo stress in USD
    (date(2020,  3, 23), -0.0075),   # COVID USD squeeze (worst)
    (date(2020,  6,  1), -0.0025),   # Fed swap lines normalize
    (date(2021,  1,  1), -0.0015),   # Very compressed, abundant USD
    (date(2021,  6,  1), -0.0010),   # Tightest (post-GFC)
    (date(2022,  3,  1), -0.0020),   # ECB/Fed divergence
    (date(2022,  9,  1), -0.0035),   # Parity EUR/USD, EUR cheap
    (date(2023,  3,  1), -0.0028),   # Normalise
    (date(2023,  9,  1), -0.0025),
    (date(2024,  3,  1), -0.0022),
    (date(2024,  9,  1), -0.0020),
    (END_DATE,           -0.0018),
]


def _xccy_tenor_factor(T: float) -> float:
    """
    XCCY basis tends to be larger (more negative) at longer tenors.
    1Y = 0.55 × 5Y anchor; 5Y = 1.0; 10Y = 1.35; 30Y = 1.70
    """
    if T <= 0:   return 0.50
    return 0.40 + 0.60 * math.log1p(T) / math.log1p(5)


def _xccy_vol(T: float) -> float:
    if T < 2:   return 0.0003
    if T < 5:   return 0.00025
    return 0.0002


# ══════════════════════════════════════════════════════════════════════════════
#  Generic spread series generator
# ══════════════════════════════════════════════════════════════════════════════

def generate_spread_series(
    name: str,
    tenors: list[str],
    level_knots: list[tuple[date, float]],
    tenor_factor_fn,     # f(T_years) → multiplier on 5Y-anchor level
    vol_fn,              # f(T_years) → daily vol (decimal)
    floor: float = 0.0,  # Hard floor (credit spreads / TLP always ≥ 0)
    allow_negative: bool = False,
) -> None:
    filepath = path.join(OUTPUT_DIR, f"historical_{name.lower()}.csv")

    with open(filepath, "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["date", "tenor", "rate"])

        noise = {t: 0.0 for t in tenors}
        cur = START_DATE

        while cur <= END_DATE:
            if cur.weekday() >= 5:
                cur += timedelta(days=1)
                continue

            level_5y = _lerp(cur, level_knots)

            for tenor in tenors:
                T = tenor_years(tenor)
                base = level_5y * tenor_factor_fn(T)

                vol = vol_fn(T)
                noise[tenor] = 0.93 * noise[tenor] + random.gauss(0, vol)
                rate = base + noise[tenor]

                if not allow_negative:
                    rate = max(floor, rate)
                else:
                    rate = max(floor, rate)   # floor applies even for negative series

                writer.writerow([cur.isoformat(), tenor, round(rate, 6)])

            cur += timedelta(days=1)

    n_rows = sum(1 for _ in open(filepath, encoding="utf-8")) - 1
    print(f"  {filepath}  →  {n_rows:,} rows")


# ══════════════════════════════════════════════════════════════════════════════
#  Entry point
# ══════════════════════════════════════════════════════════════════════════════

TENORS_CREDIT = ["1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "15Y", "20Y", "30Y"]
TENORS_TLP    = ["3M", "6M", "1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "15Y", "20Y", "30Y"]
TENORS_XCCY   = ["1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "15Y", "20Y", "30Y"]


def main() -> None:
    print("=" * 60)
    print(f"FTP spread series generator  {START_DATE} → {END_DATE}")
    print("=" * 60)

    print("\n[Component 2] USD Credit Spread (G-SIB senior unsecured z-spread over SOFR)…")
    generate_spread_series(
        "USD_CREDIT_SPREAD",
        TENORS_CREDIT,
        _USD_CS_LEVEL,
        _cs_tenor_factor,
        _cs_vol,
        floor=0.0005,    # floor at 5bp — credit spread never truly zero
    )

    print("[Component 2] EUR Credit Spread (EU G-SIB senior unsecured z-spread over €STR)…")
    generate_spread_series(
        "EUR_CREDIT_SPREAD",
        TENORS_CREDIT,
        _EUR_CS_LEVEL,
        _cs_tenor_factor,
        _cs_vol,
        floor=0.0005,
    )

    print("[Component 3] USD TLP (FHLB advance spread over SOFR OIS)…")
    generate_spread_series(
        "USD_TLP",
        TENORS_TLP,
        _USD_TLP_LEVEL,
        _tlp_tenor_shape,
        _tlp_vol,
        floor=0.0002,    # TLP always positive (collateralized funding premium)
    )

    print("[Component 3] EUR TLP (covered bond / SNP spread over €STR)…")
    generate_spread_series(
        "EUR_TLP",
        TENORS_TLP,
        _EUR_TLP_LEVEL,
        _tlp_tenor_shape,
        _tlp_vol,
        floor=0.0002,
    )

    print("[Component 7B] EUR/USD Cross-Currency Basis (XCCY swap spread)…")
    generate_spread_series(
        "XCCY_EUR_USD",
        TENORS_XCCY,
        _XCCY_LEVEL,
        _xccy_tenor_factor,
        _xccy_vol,
        floor=-0.0150,   # floor at -150bp (extreme stress boundary)
        allow_negative=True,
    )

    print("\nDone.")


if __name__ == "__main__":
    main()

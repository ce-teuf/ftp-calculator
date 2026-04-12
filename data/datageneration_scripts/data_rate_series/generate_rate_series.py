#!/usr/bin/env python3
"""
generate_rate_series.py
Generates realistic synthetic historical rate series for SOFR, €STR and EURIBOR
covering 2014-01-01 → today (~11 years of daily data).

All rates stored as DECIMAL fractions (0.045 = 4.50%).

Rate regimes modelled:
  USD (SOFR):  ZIRP 2014-2015, hiking 2015-2018, cuts 2019, COVID ZLB 2020-2021,
               aggressive hiking 2022-2023 (0% → 5.33%), cuts 2024-2025
  EUR (€STR):  Negative rates 2014-2021 (-0.5% trough), rapid hiking 2022-2023
               (→ 4.0%), cuts 2024-2025 (~2.5%)
  EUR (EURIBOR): €STR + credit/term premium spread (~15-25bp on 3M)

Term structure: Nelson-Siegel parametrisation, regime-aware slope & curvature.
"""

from __future__ import annotations

import csv
import math
import random
from datetime import date, timedelta
from os import path

OUTPUT_DIR = path.dirname(path.abspath(__file__))
random.seed(42)

# FTP-aligned tenors (decimal years used internally; stored as string labels)
TENORS_USD  = ["1D", "1W", "2W", "1M", "2M", "3M", "6M", "9M",
               "1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "15Y", "20Y", "30Y"]
TENORS_EUR  = ["1D", "1W", "2W", "1M", "2M", "3M", "6M", "9M",
               "1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "15Y", "20Y", "30Y"]
TENORS_EURIBOR = ["1W", "2W", "1M", "2M", "3M", "6M", "9M",
                  "1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "15Y", "20Y", "30Y"]

START_DATE = date(2014, 1, 1)
END_DATE   = date.today()


# ── Tenor → years mapping ─────────────────────────────────────────────────────

def tenor_years(t: str) -> float:
    if t.endswith("D"):  return int(t[:-1]) / 365
    if t.endswith("W"):  return int(t[:-1]) / 52
    if t.endswith("M"):  return int(t[:-1]) / 12
    if t.endswith("Y"):  return float(t[:-1])
    return 0.0


# ── Nelson-Siegel term structure ──────────────────────────────────────────────

def nelson_siegel(T: float, beta0: float, beta1: float, beta2: float, tau: float = 1.5) -> float:
    """
    NS spot rate for maturity T (years).
    beta0 = long-run level (T→∞)
    beta1 = r_on − beta0  (T→0 gives beta0 + beta1 = r_on)
    beta2 = curvature (hump)
    tau   = decay factor (~1.5Y is typical)

    Convention: beta1 < 0 → upward sloping; beta1 > 0 → inverted.
    """
    if T < 1e-6:
        return beta0 + beta1
    lam = T / tau
    decay = (1 - math.exp(-lam)) / lam
    return beta0 + beta1 * decay + beta2 * (decay - math.exp(-lam))


def ns_from_anchors(r_on: float, r_long: float, curvature: float, tau: float = 1.5) -> tuple[float, float, float]:
    """Build NS params from overnight and long-end anchors.
    beta0 = r_long (T→∞ limit)
    beta1 = r_on − r_long (T→0 limit = r_on)
    beta2 = curvature (hump in the belly)
    """
    return r_long, r_on - r_long, curvature


# ── Overnight rate pathways ───────────────────────────────────────────────────

def _lerp(t: date, segments: list[tuple[date, float]]) -> float:
    """Piecewise linear interpolation over (date, rate) knots."""
    if t <= segments[0][0]:  return segments[0][1]
    if t >= segments[-1][0]: return segments[-1][1]
    for i in range(1, len(segments)):
        d0, r0 = segments[i - 1]
        d1, r1 = segments[i]
        if d0 <= t <= d1:
            frac = (t - d0).days / max((d1 - d0).days, 1)
            return r0 + frac * (r1 - r0)
    return segments[-1][1]


# USD SOFR overnight path (decimal)
_SOFR_KNOTS: list[tuple[date, float]] = [
    (date(2014,  1,  1), 0.0008),  # near-zero (post-GFC)
    (date(2015, 12, 16), 0.0025),  # Dec-2015 liftoff
    (date(2018, 12,  1), 0.0240),  # peak 2018 tightening
    (date(2019,  7,  1), 0.0225),  # insurance cuts start
    (date(2019, 12,  1), 0.0175),
    (date(2020,  3, 16), 0.0008),  # COVID emergency cut
    (date(2022,  3, 16), 0.0033),  # first hike of new cycle
    (date(2022,  6,  1), 0.0158),
    (date(2022, 11,  1), 0.0383),
    (date(2023,  2,  1), 0.0458),
    (date(2023,  7, 26), 0.0533),  # peak
    (date(2024,  9, 18), 0.0508),  # first cut
    (date(2024, 11, 14), 0.0458),
    (date(2024, 12, 18), 0.0433),
    (END_DATE,           0.0433),
]

# EUR €STR overnight path (decimal)
_ESTR_KNOTS: list[tuple[date, float]] = [
    (date(2014,  1,  1), -0.0008),  # ECB deposit rate close to 0
    (date(2014,  6,  5), -0.0010),  # June 2014 first negative
    (date(2015,  1,  1), -0.0020),
    (date(2016,  3, 16), -0.0040),  # trough -40bp
    (date(2022,  7, 27),  0.0000),  # ECB liftoff
    (date(2022, 10, 27),  0.0150),
    (date(2023,  3, 22),  0.0300),
    (date(2023,  9, 20),  0.0400),  # peak
    (date(2024,  6,  6),  0.0375),  # first cut
    (date(2024,  9, 12),  0.0350),
    (date(2024, 10, 17),  0.0325),
    (date(2024, 12, 12),  0.0300),
    (date(2025,  1,  1),  0.0265),
    (END_DATE,            0.0240),
]


# ── Regime-aware long-end anchor ──────────────────────────────────────────────

def _long_end_usd(r_on: float, t: date) -> float:
    """
    Estimate of the long-run equilibrium / 10Y rate for USD.
    Historically the 10Y tracks the Fed Funds + term premium:
    - ZIRP era: 10Y ≈ 2.0–2.5% even with O/N near 0  (market expecting normalisation)
    - Hiking:   10Y lags behind O/N (premium compresses → inversion)
    - Post-peak: 10Y reprices lower as cuts are priced in
    """
    year = t.year + t.month / 12
    if year < 2016:   return 0.022           # 2.2% – gradual hike expectations
    if year < 2019:   return 0.027 + r_on * 0.15  # modest term premium
    if year < 2020:   return 0.020           # insurance cut environment
    if year < 2022:   return 0.018 + 0.007 * (year - 2020)  # COVID: 1.8% rising
    if year < 2023:   return min(r_on + 0.003, 0.042)  # inversion: 10Y < O/N
    if year < 2024.5: return min(r_on - 0.008, 0.043)  # deep inversion peak
    return r_on - 0.006                      # cuts: 10Y reprices down slowly


def _long_end_eur(r_on: float, t: date) -> float:
    """
    Long-end anchor for EUR (10Y Bund proxy).
    Pre-2022: modestly positive even when O/N is negative (ECB credibility anchor)
    Hiking: long end rises but stays below O/N at peak
    """
    year = t.year + t.month / 12
    if year < 2016:   return 0.008           # 0.8% – pre-QE residual premium
    if year < 2021:   return max(-0.002, r_on + 0.005)  # negative but not too deep
    if year < 2022.5: return 0.005 + 0.015 * (year - 2021)  # gradual re-pricing
    if year < 2024:   return min(r_on - 0.005, 0.030)  # slight inversion at peak
    return r_on - 0.004                      # cuts: 10Y < O/N by ~40bp then re-steepens


def _curvature_usd(r_on: float, t: date) -> float:
    """Hump curvature (beta2): belly rises relative to endpoints."""
    year = t.year + t.month / 12
    # Positive = belly above chord (hump) — typical in ZIRP/early-hiking
    # Negative = belly below chord — typical in inverted markets
    if year < 2022:   return -0.002
    if year < 2024:   return  0.004   # slight hump during normalisation
    return  0.001


def _curvature_eur(r_on: float, t: date) -> float:
    year = t.year + t.month / 12
    if year < 2016:   return -0.001
    if year < 2023:   return -0.002
    return  0.002


# ── NS parameter builders ─────────────────────────────────────────────────────

def _ns_params_usd(r_on: float, t: date) -> tuple[float, float, float]:
    r_long = _long_end_usd(r_on, t)
    curv   = _curvature_usd(r_on, t)
    return ns_from_anchors(r_on, r_long, curv)


def _ns_params_eur(r_on: float, t: date) -> tuple[float, float, float]:
    r_long = _long_end_eur(r_on, t)
    curv   = _curvature_eur(r_on, t)
    return ns_from_anchors(r_on, r_long, curv)


# ── Noise model: tenor-dependent daily vol (in bp) ───────────────────────────

def _daily_vol(tenor: str) -> float:
    """Daily rate volatility in decimal (not bp)."""
    T = tenor_years(tenor)
    if T < 0.1:   return 0.0008   # 8bp  — short end more volatile in crisis
    if T < 0.5:   return 0.0006   # 6bp
    if T < 1:     return 0.0005   # 5bp
    if T < 3:     return 0.0004   # 4bp
    if T < 7:     return 0.0003   # 3bp
    return 0.0002                  # 2bp  — long end more anchored


# ── Main series generator ─────────────────────────────────────────────────────

def generate_series(
    name: str,
    tenors: list[str],
    knots: list[tuple[date, float]],
    ns_params_fn,
    spread_fn=None,          # optional: f(tenor_str, r_on) → extra spread
) -> None:
    """Generate and write one CSV file."""
    filepath = path.join(OUTPUT_DIR, f"historical_{name.lower()}.csv")

    with open(filepath, "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["date", "tenor", "rate"])

        cur = START_DATE
        # Persistent noise state per tenor (mean-reverting random walk)
        noise = {t: 0.0 for t in tenors}

        while cur <= END_DATE:
            # Skip weekends
            if cur.weekday() >= 5:
                cur += timedelta(days=1)
                continue

            r_on = _lerp(cur, knots)
            b0, b1, b2 = ns_params_fn(r_on, cur)

            for tenor in tenors:
                T = max(tenor_years(tenor), 1/365)
                base = nelson_siegel(T, b0, b1, b2)

                # Optional additional spread (EURIBOR vs ESTR)
                if spread_fn:
                    base += spread_fn(tenor, r_on)

                # Mean-reverting noise (AR(1) with daily shock)
                vol = _daily_vol(tenor)
                noise[tenor] = 0.95 * noise[tenor] + random.gauss(0, vol)
                rate = base + noise[tenor]

                # Hard floor: no rate below -1.5% for EUR, -0.1% for USD
                floor = -0.015 if name in ("ESTR", "EURIBOR") else -0.001
                rate = max(floor, rate)

                writer.writerow([cur.isoformat(), tenor, round(rate, 6)])

            cur += timedelta(days=1)

    n_rows = sum(1 for _ in open(filepath, encoding="utf-8")) - 1  # exclude header
    print(f"  {filepath}  →  {n_rows:,} rows")


# ── EURIBOR spread model ──────────────────────────────────────────────────────

def _euribor_spread(tenor: str, r_on: float) -> float:
    """
    EURIBOR ≈ ESTR + credit/term premium.
    In stressed / hiking environments spread widens slightly.
    Short end ≈ +10-25bp; long end ≈ +5-15bp.
    """
    T = tenor_years(tenor)
    base_spread = 0.0015 + 0.0010 / (1 + T)  # ~15bp short, ~10bp long
    # Slight widening when rates are very negative (pre-2022)
    if r_on < 0:
        base_spread += abs(r_on) * 0.20
    return base_spread


# ── Entry point ───────────────────────────────────────────────────────────────

def main() -> None:
    print("=" * 60)
    print(f"Rate series generator  {START_DATE} → {END_DATE}")
    print("=" * 60)

    print("\nGenerating SOFR (USD) …")
    generate_series("SOFR",    TENORS_USD,    _SOFR_KNOTS, _ns_params_usd)

    print("Generating ESTR (EUR) …")
    generate_series("ESTR",    TENORS_EUR,    _ESTR_KNOTS, _ns_params_eur)

    print("Generating EURIBOR (EUR) …")
    generate_series("EURIBOR", TENORS_EURIBOR, _ESTR_KNOTS, _ns_params_eur,
                    spread_fn=_euribor_spread)

    print("\nDone.")


if __name__ == "__main__":
    main()

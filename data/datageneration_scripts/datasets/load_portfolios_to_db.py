#!/usr/bin/env python3
"""
load_portfolios_to_db.py
Generates 20 diverse portfolio examples and inserts them into
portfolios_v3 + portfolio_rows (format expected by the Rust backend).

Schedule format  : [{"date": "MM-YYYY", "buckets": [12 float values]}]
Buckets align with FTP_TENORS: 1M 3M 6M 12M 24M 36M 60M 84M 120M 180M 240M 360M

Usage:
    python load_portfolios_to_db.py
    DATABASE_URL=... python load_portfolios_to_db.py
"""

from __future__ import annotations

import json
import math
import os
import random
import uuid
from datetime import datetime
from typing import NamedTuple

import psycopg2
import psycopg2.extras

# ── Config ────────────────────────────────────────────────────────────────────

DB_URL = os.getenv(
    "DATABASE_URL",
    "postgresql://ftp_dev:ftp_dev@127.0.0.1:5432/ftp_simulator_dev",
)

GLOBAL_START = "2015-01"
GLOBAL_END   = "2024-12"

# FTP standard tenor buckets (in months)
FTP_TENORS = [1, 3, 6, 12, 24, 36, 60, 84, 120, 180, 240, 360]

random.seed(42)


# ── Date helpers ──────────────────────────────────────────────────────────────

def parse_period(s: str) -> datetime:
    return datetime.strptime(s, "%Y-%m")

def fmt_period(dt: datetime) -> str:
    """Returns first-of-month in YYYY-MM-DD format, as expected by the Rust backend."""
    return dt.strftime("%Y-%m-01")

def monthly_dates(start: str, end: str) -> list[str]:
    cur = parse_period(start)
    end_dt = parse_period(end)
    out = []
    while cur <= end_dt:
        out.append(fmt_period(cur))
        if cur.month == 12:
            cur = cur.replace(year=cur.year + 1, month=1)
        else:
            cur = cur.replace(month=cur.month + 1)
    return out


# ── Amortization profile helpers ──────────────────────────────────────────────

def amort_profile(maturity_months: int, curvature: float) -> list[float]:
    """Normalized amortization profile: profile[t-1] = remaining fraction at month t."""
    if maturity_months <= 1:
        return [0.0]
    denom = maturity_months - 1
    return [round((1.0 - (t / denom)) ** curvature, 6) for t in range(maturity_months)]

def profile_at_tenor(profile: list[float], tenor_m: int) -> float:
    """Get profile value at a given tenor (in months)."""
    idx = tenor_m - 1
    if idx < 0 or idx >= len(profile):
        return 0.0
    return profile[idx]

def schedule_buckets(profile: list[float]) -> list[float]:
    """Extract the 12 FTP tenor bucket values from a full monthly profile."""
    return [profile_at_tenor(profile, t) for t in FTP_TENORS]


# ── Outstanding trend helpers ─────────────────────────────────────────────────

def trend_linear(n: int, start: float, end: float, noise_std: float = 0.0) -> list[float]:
    vals = [start + (end - start) * i / max(n - 1, 1) for i in range(n)]
    return [max(0.0, v + random.gauss(0, noise_std)) for v in vals]

def trend_exponential(n: int, start: float, end: float, noise_std: float = 0.0) -> list[float]:
    if start <= 0 or end <= 0:
        return trend_linear(n, start, end, noise_std)
    vals = [start * (end / start) ** (i / max(n - 1, 1)) for i in range(n)]
    return [max(0.0, v + random.gauss(0, noise_std)) for v in vals]

def trend_convex(n: int, start: float, end: float, curvature: float = 2.0, noise_std: float = 0.0) -> list[float]:
    vals = [start + (end - start) * (i / max(n - 1, 1)) ** curvature for i in range(n)]
    return [max(0.0, v + random.gauss(0, noise_std)) for v in vals]

def trend_logistic(n: int, start: float, end: float, midpoint: float = 0.5, steepness: float = 10.0, noise_std: float = 0.0) -> list[float]:
    def logit(x: float) -> float:
        return 1.0 / (1.0 + math.exp(-steepness * (x - midpoint)))
    vals = [start + (end - start) * logit(i / max(n - 1, 1)) for i in range(n)]
    return [max(0.0, v + random.gauss(0, noise_std)) for v in vals]

def trend_nmd_decay(n: int, base: float, lam: float, noise_std: float = 0.0) -> list[float]:
    """Slowly decaying outstanding (NMD behavioral)."""
    vals = [base * math.exp(-lam * i / 12) for i in range(n)]  # lam per year
    return [max(0.0, v + random.gauss(0, noise_std * base)) for v in vals]


# ── Portfolio definitions ─────────────────────────────────────────────────────

class PortfolioDef(NamedTuple):
    name: str
    description: str
    schedule_type: str          # 'stock_amort' | 'flux'
    rows: list[dict]             # list of row defs


class RowDef(NamedTuple):
    label: str
    maturity_months: int
    curvature: float
    curvature_std: float
    outstanding_fn: str          # 'linear' | 'exponential' | 'convex' | 'logistic' | 'nmd'
    outstanding_params: dict


def build_row_data(row: RowDef, dates: list[str]) -> tuple[str, str]:
    """Build schedule_json and outstanding_json strings for a portfolio_rows entry."""
    n = len(dates)

    # Compute per-date schedule buckets (vary curvature slightly over time)
    schedule = []
    for i, date in enumerate(dates):
        c = max(0.1, row.curvature + random.gauss(0, row.curvature_std))
        profile = amort_profile(row.maturity_months, c)
        buckets = schedule_buckets(profile)
        schedule.append({"date": date, "buckets": buckets})

    # Compute outstanding values
    fn = row.outstanding_fn
    p = row.outstanding_params
    if fn == "linear":
        vals = trend_linear(n, p["start"], p["end"], p.get("noise", 0))
    elif fn == "exponential":
        vals = trend_exponential(n, p["start"], p["end"], p.get("noise", 0))
    elif fn == "convex":
        vals = trend_convex(n, p["start"], p["end"], p.get("curvature", 2.0), p.get("noise", 0))
    elif fn == "logistic":
        vals = trend_logistic(n, p["start"], p["end"], p.get("midpoint", 0.5), p.get("steepness", 10.0), p.get("noise", 0))
    elif fn == "nmd":
        vals = trend_nmd_decay(n, p["base"], p["lam"], p.get("noise", 0))
    else:
        vals = trend_linear(n, p.get("start", 1e6), p.get("end", 1e6), 0)

    outstanding = [{"date": d, "outstanding": round(v, 2)} for d, v in zip(dates, vals)]

    return json.dumps(schedule), json.dumps(outstanding)


# ── The 20 portfolio definitions ──────────────────────────────────────────────

def make_portfolios() -> list[tuple[str, str, str, list[RowDef]]]:
    """Returns list of (name, description, schedule_type, [RowDef...])."""
    return [
        # 1. Retail mortgage book — long duration, steady growth
        ("Retail Mortgages 20Y", "Residential mortgage portfolio, fixed rate, 20-year maturity", "stock_amort", [
            RowDef("Fixed 20Y — Main book", 240, 1.5, 0.2,
                   "linear", {"start": 450e6, "end": 720e6, "noise": 8e6}),
        ]),
        # 2. Retail mortgage — 15Y
        ("Retail Mortgages 15Y", "Residential mortgage portfolio, fixed rate, 15-year maturity", "stock_amort", [
            RowDef("Fixed 15Y — Main book", 180, 1.5, 0.15,
                   "convex", {"start": 200e6, "end": 380e6, "curvature": 1.8, "noise": 4e6}),
        ]),
        # 3. Retail mortgage — 10Y
        ("Retail Mortgages 10Y", "Medium-term residential mortgage portfolio", "stock_amort", [
            RowDef("Fixed 10Y — Main book", 120, 1.5, 0.15,
                   "linear", {"start": 150e6, "end": 280e6, "noise": 3e6}),
        ]),
        # 4. Corporate loans 5Y — declining (portfolio being wound down)
        ("Corporate Loans 5Y", "Corporate term loans, 5-year maturity, declining book", "stock_amort", [
            RowDef("5Y Corporate — Senior", 60, 2.0, 0.1,
                   "linear", {"start": 300e6, "end": 180e6, "noise": 5e6}),
        ]),
        # 5. Corporate loans 7Y — stable
        ("Corporate Loans 7Y", "Corporate term loans, 7-year maturity, stable book", "stock_amort", [
            RowDef("7Y Corporate — Main", 84, 2.0, 0.1,
                   "linear", {"start": 250e6, "end": 260e6, "noise": 6e6}),
        ]),
        # 6. SME loans 5Y — strong growth
        ("SME Loans 5Y", "Small and medium enterprise loans, 5-year maturity", "stock_amort", [
            RowDef("5Y SME — Main book", 60, 1.8, 0.2,
                   "exponential", {"start": 80e6, "end": 190e6, "noise": 2e6}),
        ]),
        # 7. SME loans 7Y — logistic growth (S-curve)
        ("SME Loans 7Y", "SME loans 7Y, S-curve growth trajectory", "stock_amort", [
            RowDef("7Y SME — Main book", 84, 1.8, 0.2,
                   "logistic", {"start": 40e6, "end": 120e6, "midpoint": 0.5, "steepness": 8.0, "noise": 1e6}),
        ]),
        # 8. Consumer credit 2Y — fast growing
        ("Consumer Credit 2Y", "Unsecured consumer loans, 2-year maturity", "stock_amort", [
            RowDef("2Y Consumer — Main", 24, 1.0, 0.3,
                   "exponential", {"start": 50e6, "end": 130e6, "noise": 2e6}),
        ]),
        # 9. Consumer credit 3Y
        ("Consumer Credit 3Y", "Unsecured consumer loans, 3-year maturity", "stock_amort", [
            RowDef("3Y Consumer — Main", 36, 1.0, 0.3,
                   "linear", {"start": 60e6, "end": 110e6, "noise": 2.5e6}),
        ]),
        # 10. Commercial RE 20Y — long term stable
        ("Commercial Real Estate 20Y", "Commercial real estate loans, 20-year maturity", "stock_amort", [
            RowDef("20Y CRE — Paris CBD", 240, 2.0, 0.1,
                   "linear", {"start": 600e6, "end": 750e6, "noise": 10e6}),
        ]),
        # 11. Commercial RE 15Y
        ("Commercial Real Estate 15Y", "Commercial real estate, 15Y, two cohorts", "stock_amort", [
            RowDef("15Y CRE — Office", 180, 2.0, 0.15,
                   "linear", {"start": 200e6, "end": 280e6, "noise": 5e6}),
            RowDef("15Y CRE — Retail", 180, 1.8, 0.15,
                   "linear", {"start": 80e6, "end": 120e6, "noise": 3e6}),
        ]),
        # 12. NMD Retail demand deposits — exponential decay
        ("NMD Retail Demand Deposits", "Retail current accounts — behavioral exponential decay model", "stock_amort", [
            RowDef("Demand Deposits — Core (75%)", 120, 0.3, 0.05,
                   "nmd", {"base": 2_000e6, "lam": 0.05, "noise": 0.005}),
            RowDef("Demand Deposits — Non-core (25%)", 24, 1.0, 0.1,
                   "nmd", {"base": 650e6, "lam": 0.3, "noise": 0.01}),
        ]),
        # 13. NMD Retail savings — slower decay
        ("NMD Retail Savings Accounts", "Retail savings accounts — behavioral model, core/non-core split", "stock_amort", [
            RowDef("Savings — Core (60%)", 180, 0.25, 0.05,
                   "nmd", {"base": 1_500e6, "lam": 0.08, "noise": 0.005}),
            RowDef("Savings — Non-core (40%)", 36, 1.0, 0.1,
                   "nmd", {"base": 1_000e6, "lam": 0.25, "noise": 0.01}),
        ]),
        # 14. Corporate deposits — fast decay (more volatile behavior)
        ("Corporate Current Accounts", "Corporate demand deposits, high volatility behavioral model", "stock_amort", [
            RowDef("Corporate Deposits — Core (50%)", 60, 0.5, 0.1,
                   "nmd", {"base": 800e6, "lam": 0.15, "noise": 0.02}),
            RowDef("Corporate Deposits — Non-core (50%)", 12, 1.0, 0.2,
                   "nmd", {"base": 800e6, "lam": 0.5, "noise": 0.03}),
        ]),
        # 15. Term deposit 1Y — bullet profile
        ("Term Deposits 1Y", "Fixed-term retail deposits, 1-year maturity (bullet)", "stock_amort", [
            RowDef("1Y Term Deposits — Bullet", 12, 50.0, 0.5,
                   "linear", {"start": 500e6, "end": 600e6, "noise": 15e6}),
        ]),
        # 16. Term deposit 2Y — bullet
        ("Term Deposits 2Y", "Fixed-term retail deposits, 2-year maturity", "stock_amort", [
            RowDef("2Y Term Deposits — Bullet", 24, 50.0, 0.5,
                   "linear", {"start": 300e6, "end": 420e6, "noise": 10e6}),
        ]),
        # 17. Auto loans 5Y — linear amortization
        ("Auto Loans 5Y", "Retail auto loans, 5-year amortizing", "stock_amort", [
            RowDef("5Y Auto — Main book", 60, 1.0, 0.2,
                   "convex", {"start": 120e6, "end": 210e6, "curvature": 1.5, "noise": 3e6}),
        ]),
        # 18. Revolving credit facilities 3Y — flat until maturity (cliff)
        ("Revolving Credit Facilities", "Corporate revolving credit lines, drawn/undrawn mix", "stock_amort", [
            RowDef("Revolving — Drawn portion", 36, 10.0, 1.0,
                   "linear", {"start": 180e6, "end": 200e6, "noise": 15e6}),
        ]),
        # 19. Infrastructure / project finance 30Y — very long duration
        ("Infrastructure Loans 30Y", "Project finance loans, 30-year tenor, linear amortization", "stock_amort", [
            RowDef("30Y Infrastructure — Main", 360, 1.0, 0.05,
                   "linear", {"start": 1_200e6, "end": 1_500e6, "noise": 20e6}),
        ]),
        # 20. Student loans 10Y — concave amortization (back-loaded)
        ("Student Loans 10Y", "Retail student loans, 10-year maturity, back-loaded repayment", "stock_amort", [
            RowDef("10Y Student — Main", 120, 0.5, 0.1,
                   "logistic", {"start": 90e6, "end": 160e6, "midpoint": 0.6, "steepness": 6.0, "noise": 2e6}),
        ]),
        # 21. Trade finance 1Y — very short term, high turnover
        ("Trade Finance 1Y", "Short-term trade finance loans, 12-month bullet", "stock_amort", [
            RowDef("1Y Trade Finance — Main", 12, 20.0, 1.0,
                   "linear", {"start": 400e6, "end": 550e6, "noise": 20e6}),
        ]),
        # 22. Syndicated loans 7Y — two tranches
        ("Syndicated Loans 7Y", "Senior secured syndicated loans, two-tranche structure", "flux", [
            RowDef("Tranche A — Amortizing (60%)", 84, 1.5, 0.1,
                   "linear", {"start": 600e6, "end": 700e6, "noise": 12e6}),
            RowDef("Tranche B — Bullet (40%)", 84, 30.0, 0.5,
                   "linear", {"start": 400e6, "end": 470e6, "noise": 8e6}),
        ]),
    ]


# ── DB insertion ──────────────────────────────────────────────────────────────

def load_to_db(conn: psycopg2.extensions.connection) -> None:
    dates = monthly_dates(GLOBAL_START, GLOBAL_END)
    portfolios = make_portfolios()

    with conn.cursor() as cur:
        inserted_portfolios = 0
        inserted_rows = 0

        for name, description, schedule_type, row_defs in portfolios:
            portfolio_id = str(uuid.uuid4())

            cur.execute(
                """
                INSERT INTO portfolios_v3 (id, name, description, schedule_type)
                VALUES (%s, %s, %s, %s)
                """,
                (portfolio_id, name, description, schedule_type),
            )
            inserted_portfolios += 1

            for order, row_def in enumerate(row_defs):
                row_id = str(uuid.uuid4())
                schedule_json, outstanding_json = build_row_data(row_def, dates)

                cur.execute(
                    """
                    INSERT INTO portfolio_rows
                        (id, portfolio_id, label, schedule_json, outstanding_json, row_order)
                    VALUES (%s, %s, %s, %s, %s, %s)
                    """,
                    (row_id, portfolio_id, row_def.label, schedule_json, outstanding_json, order),
                )
                inserted_rows += 1

        conn.commit()
        print(f"Inserted {inserted_portfolios} portfolios, {inserted_rows} rows ({len(dates)} dates each).")


# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    print("=" * 60)
    print("Portfolio loader — portfolios_v3 + portfolio_rows")
    print(f"Period: {GLOBAL_START} → {GLOBAL_END}")
    print("=" * 60)

    conn = psycopg2.connect(DB_URL)
    try:
        load_to_db(conn)
    finally:
        conn.close()

    print("Done.")

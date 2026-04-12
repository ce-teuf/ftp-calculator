#!/usr/bin/env python3
"""
load_rate_series_to_db.py
Loads historical rate series (SOFR, €STR, EURIBOR) into the rate_series_data table.

If the CSV files don't exist yet, it regenerates them first by calling generate_rate_series.py.

Usage:
    python load_rate_series_to_db.py
    DATABASE_URL=... python load_rate_series_to_db.py
"""

from __future__ import annotations

import csv
import os
import sys
import uuid
from pathlib import Path

import psycopg2
import psycopg2.extras

# ── Config ────────────────────────────────────────────────────────────────────

DB_URL = os.getenv(
    "DATABASE_URL",
    "postgresql://ftp_dev:ftp_dev@127.0.0.1:5432/ftp_simulator_dev",
)

THIS_DIR = Path(__file__).parent


# ── Mapping: CSV file → (series_name, component, currency) ───────────────────

SERIES = [
    # Component 1 — Base Rate
    {
        "file":       THIS_DIR / "historical_sofr.csv",
        "name":       "SOFR",
        "component":  "base_rate",
        "currency":   "USD",
    },
    {
        "file":       THIS_DIR / "historical_estr.csv",
        "name":       "ESTR",
        "component":  "base_rate",
        "currency":   "EUR",
    },
    # IBOR reference (Component 7C basis)
    {
        "file":       THIS_DIR / "historical_euribor.csv",
        "name":       "EURIBOR",
        "component":  "ibor",
        "currency":   "EUR",
    },
    # Component 2 — Credit Spread (bank senior unsecured z-spread)
    {
        "file":       THIS_DIR / "historical_usd_credit_spread.csv",
        "name":       "USD_CREDIT_SPREAD",
        "component":  "credit_spread",
        "currency":   "USD",
    },
    {
        "file":       THIS_DIR / "historical_eur_credit_spread.csv",
        "name":       "EUR_CREDIT_SPREAD",
        "component":  "credit_spread",
        "currency":   "EUR",
    },
    # Component 3 — Term Liquidity Premium
    {
        "file":       THIS_DIR / "historical_usd_tlp.csv",
        "name":       "USD_TLP",
        "component":  "tlp",
        "currency":   "USD",
    },
    {
        "file":       THIS_DIR / "historical_eur_tlp.csv",
        "name":       "EUR_TLP",
        "component":  "tlp",
        "currency":   "EUR",
    },
    # Component 7B — Cross-Currency Basis
    {
        "file":       THIS_DIR / "historical_xccy_eur_usd.csv",
        "name":       "XCCY_EUR_USD",
        "component":  "basis_risk",
        "currency":   "EUR",
    },
]


# ── Regenerate CSVs if missing ────────────────────────────────────────────────

def ensure_csvs_exist() -> None:
    import subprocess
    missing = [s for s in SERIES if not s["file"].exists()]
    if not missing:
        return

    # Determine which generators to run based on which files are missing
    missing_names = {s["name"] for s in missing}
    base_series   = {"SOFR", "ESTR", "EURIBOR"}
    spread_series = {"USD_CREDIT_SPREAD", "EUR_CREDIT_SPREAD", "USD_TLP", "EUR_TLP", "XCCY_EUR_USD"}

    for gen_name, needed_set, gen_file in [
        ("base rate", base_series, THIS_DIR / "generate_rate_series.py"),
        ("FTP spread", spread_series, THIS_DIR / "generate_ftp_spreads.py"),
    ]:
        if missing_names & needed_set:
            print(f"[info] Regenerating {gen_name} series from {gen_file.name}...")
            if not gen_file.exists():
                print(f"[error] Generator not found: {gen_file}", file=sys.stderr)
                sys.exit(1)
            result = subprocess.run([sys.executable, str(gen_file)], capture_output=True, text=True)
            if result.returncode != 0:
                print(result.stderr, file=sys.stderr)
                sys.exit(1)
            print(result.stdout)


# ── Load CSV into rate_series_data ────────────────────────────────────────────

def load_series(conn: psycopg2.extensions.connection, series_def: dict) -> int:
    filepath = series_def["file"]
    name     = series_def["name"]
    comp     = series_def["component"]
    currency = series_def["currency"]

    rows_inserted = 0
    rows_skipped  = 0

    with open(filepath, newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)

        # Expected columns: date, tenor, rate
        batch: list[tuple] = []

        for row in reader:
            obs_date = row.get("date", "").strip()
            tenor    = row.get("tenor", "").strip() or None
            raw_rate = row.get("rate", "").strip()

            if not obs_date or not raw_rate:
                continue
            try:
                rate = float(raw_rate)
            except ValueError:
                continue

            batch.append((
                str(uuid.uuid4()),  # id
                name,               # series_name
                comp,               # component
                currency,           # currency
                obs_date,           # obs_date
                tenor,              # tenor (nullable)
                rate,               # rate
            ))

            # Flush in batches of 5000
            if len(batch) >= 5000:
                rows_inserted += _flush(conn, batch)
                batch = []

        if batch:
            rows_inserted += _flush(conn, batch)

    return rows_inserted


def _flush(conn: psycopg2.extensions.connection, batch: list[tuple]) -> int:
    with conn.cursor() as cur:
        psycopg2.extras.execute_values(
            cur,
            """
            INSERT INTO rate_series_data
                (id, series_name, component, currency, obs_date, tenor, rate)
            VALUES %s
            ON CONFLICT (series_name, obs_date, COALESCE(tenor, ''))
            DO NOTHING
            """,
            batch,
        )
    conn.commit()
    return len(batch)


# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    print("=" * 60)
    print("Rate series loader — rate_series_data")
    print("=" * 60)

    ensure_csvs_exist()

    conn = psycopg2.connect(DB_URL)
    conn.cursor().execute("SET search_path TO sc_series, sc_curves, sc_portfolios, sc_studies, public")
    conn.commit()
    total = 0
    try:
        for s in SERIES:
            print(f"\n[loading] {s['name']} ({s['currency']}) from {s['file'].name}...")
            n = load_series(conn, s)
            print(f"  → {n:,} rows inserted")
            total += n
    finally:
        conn.close()

    print(f"\nTotal: {total:,} rate observations loaded.")
    print("Done.")

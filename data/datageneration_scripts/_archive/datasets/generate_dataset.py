#!/usr/bin/env python3
"""
Dataset Generator — assembles FTP-ready datasets from source CSVs/JSONs.

Reads:
  ../data_entities/   branches.csv, business_units.csv, sales.csv
  ../data_contracts/  contracts.csv
  ../data_rate_series/ current_curves.json, historical_*.csv
  ../data_schedules/   nmd_behavioral_profiles.json, schedule_*.csv

Produces (in this directory):
  {name}/
    dataset_meta.json         — dataset header (id, name, source, as_of_date)
    contracts.csv             — contracts in original format (QuantLib-compatible)
    rate_curves.csv           — rate curves (app import format)
    runoff_models.json        — NMD behavioral runoff models
    portfolio_positions.csv   — FTP-ready positions derived from contracts

Usage:
    python generate_dataset.py [--name "My Dataset"] [--max-contracts 500] [--seed 42]
    python generate_dataset.py --list    # list existing datasets
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import os
import sys
import uuid
from datetime import date, datetime
from pathlib import Path
from typing import Any

# ── Paths ─────────────────────────────────────────────────────────────────────

SCRIPT_DIR   = Path(__file__).parent.resolve()
ROOT_DIR     = SCRIPT_DIR.parent
ENTITIES_DIR = ROOT_DIR / "data_entities"
CONTRACTS_DIR = ROOT_DIR / "data_contracts"
RATES_DIR    = ROOT_DIR / "data_rate_series"
SCHEDULES_DIR = ROOT_DIR / "data_schedules"

# Tenor labels (months) used by the FTP engine
FTP_TENORS_MONTHS = [1, 3, 6, 12, 24, 36, 60, 84, 120, 180, 240, 360]
FTP_TENOR_LABELS  = ["1M", "3M", "6M", "1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "15Y", "20Y", "30Y"]


# ── Helpers ───────────────────────────────────────────────────────────────────

def parse_tenor_months(label: str) -> int:
    """Convert tenor label to months: '1M'→1, '1Y'→12, '10Y'→120."""
    label = label.strip()
    if label.endswith("D"):
        return max(1, int(int(label[:-1]) / 30))
    if label.endswith("W"):
        return max(1, int(int(label[:-1]) / 4))
    if label.endswith("M"):
        return int(label[:-1])
    if label.endswith("Y"):
        return int(label[:-1]) * 12
    return 12


def interpolate_rate(spot_rates: dict[str, float], target_months: int) -> float:
    """Linear interpolation of a rate curve to a target tenor in months."""
    points = sorted(
        [(parse_tenor_months(k), v / 100.0) for k, v in spot_rates.items()]
    )
    if target_months <= points[0][0]:
        return points[0][1]
    if target_months >= points[-1][0]:
        return points[-1][1]
    for i in range(len(points) - 1):
        t0, r0 = points[i]
        t1, r1 = points[i + 1]
        if t0 <= target_months <= t1:
            frac = (target_months - t0) / (t1 - t0)
            return r0 + frac * (r1 - r0)
    return points[-1][1]


def build_amortisation_profile(
    amortization_type: str,
    tenor_months: int,
    profile_months: list[int],
) -> list[float]:
    """
    Build a normalised amortisation profile [0..1] at each FTP tenor bucket.

    amortization_type: linear | constant_installment | bullet | behavioral
    tenor_months: contract remaining life in months
    profile_months: list of FTP tenor bucket boundaries in months
    """
    if tenor_months <= 0:
        return [0.0] * len(profile_months)

    n = len(profile_months)

    if amortization_type == "bullet":
        # Full notional remains until maturity
        return [1.0 if m < tenor_months else 0.0 for m in profile_months]

    if amortization_type in ("linear", "constant_installment"):
        # Linearly declining profile
        profile = []
        for m in profile_months:
            remaining = max(0.0, 1.0 - m / tenor_months)
            profile.append(round(remaining, 6))
        return profile

    # Default: flat then drop (for unknown types)
    return [1.0 if m < tenor_months else 0.0 for m in profile_months]


def map_currency_to_curve(currency: str) -> str:
    """Map currency code to the closest benchmark curve."""
    mapping = {"EUR": "ESTR", "USD": "SOFR", "GBP": "SONIA"}
    return mapping.get(currency.upper(), "ESTR")


# ── Loaders ───────────────────────────────────────────────────────────────────

def load_contracts(max_contracts: int | None = None) -> list[dict]:
    path = CONTRACTS_DIR / "contracts.csv"
    if not path.exists():
        print(f"  Warning: {path} not found — run data_contracts/generate_contracts.py first")
        return []
    with open(path) as f:
        reader = csv.DictReader(f)
        rows = list(reader)
    if max_contracts:
        rows = rows[:max_contracts]
    return rows


def load_current_curves() -> dict[str, Any]:
    path = RATES_DIR / "current_curves.json"
    if not path.exists():
        print(f"  Warning: {path} not found — using default rates")
        return {}
    with open(path) as f:
        return json.load(f)


def load_nmd_profiles() -> list[dict]:
    path = SCHEDULES_DIR / "nmd_behavioral_profiles.json"
    if not path.exists():
        return []
    with open(path) as f:
        return json.load(f)


def load_sales_index() -> dict[str, dict]:
    """Build seller_id → {branch, bu} lookup."""
    path = ENTITIES_DIR / "sales.csv"
    if not path.exists():
        return {}
    with open(path) as f:
        return {row["seller_id"]: row for row in csv.DictReader(f)}


def load_entity_csv(filename: str) -> list[dict]:
    path = ENTITIES_DIR / filename
    if not path.exists():
        return []
    with open(path) as f:
        return list(csv.DictReader(f))


def load_historical_series() -> list[dict]:
    """Load all historical rate CSVs into a combined list with series_name."""
    series_files = {
        "ESTR":    "historical_estr.csv",
        "EURIBOR": "historical_euribor.csv",
        "SOFR":    "historical_sofr.csv",
    }
    result = []
    for name, fname in series_files.items():
        path = RATES_DIR / fname
        if not path.exists():
            continue
        with open(path) as f:
            for row in csv.DictReader(f):
                result.append({
                    "series_name": name,
                    "obs_date":    row.get("date", ""),
                    "tenor":       row.get("tenor", ""),
                    "rate":        row.get("rate", ""),
                })
    return result


def filter_entities(
    contracts: list[dict],
    branches: list[dict],
    business_units: list[dict],
    departments: list[dict],
    sellers: list[dict],
    treasuries: list[dict],
) -> dict[str, list[dict]]:
    """Keep only entities referenced by the contracts in this dataset."""
    branch_codes = {c.get("branch_code", "") for c in contracts if c.get("branch_code")}
    seller_ids   = {c.get("seller_id", "")   for c in contracts if c.get("seller_id")}

    filt_branches = [b for b in branches   if b.get("branch_code") in branch_codes]
    branch_ids    = {b["branch_id"] for b in filt_branches}

    filt_sellers  = [s for s in sellers    if s.get("seller_id") in seller_ids]
    bu_ids        = {s["bu_id"] for s in filt_sellers if s.get("bu_id")}

    filt_bus      = [b for b in business_units if b.get("bu_id") in bu_ids or b.get("branch_id") in branch_ids]
    all_bu_ids    = bu_ids | {b["bu_id"] for b in filt_bus}

    filt_depts    = [d for d in departments  if d.get("bu_id") in all_bu_ids]
    filt_treas    = [t for t in treasuries   if t.get("branch_id") in branch_ids]

    return {
        "branches":       filt_branches,
        "business_units": filt_bus,
        "departments":    filt_depts,
        "sellers":        filt_sellers,
        "treasuries":     filt_treas,
    }


# ── Builders ──────────────────────────────────────────────────────────────────

def build_rate_curves(curves_data: dict) -> list[dict]:
    """Convert current_curves.json into app rate_curves format."""
    result = []
    component_map = {
        "SOFR": "base_rate",
        "ESTR": "base_rate",
        "SONIA": "base_rate",
        "EURIBOR": "ibor",
    }
    currency_map = {
        "SOFR": "USD",
        "ESTR": "EUR",
        "SONIA": "GBP",
        "EURIBOR": "EUR",
    }
    as_of = date.today().isoformat()

    for curve_name, data in curves_data.items():
        spot = data.get("spot_rates", {})
        # Filter to FTP tenors only
        tenors = [t for t in FTP_TENOR_LABELS if t in spot]
        values = [spot[t] / 100.0 for t in tenors]

        if not tenors:
            continue

        result.append({
            "id": f"curve-{curve_name.lower()}-{as_of}",
            "name": f"{curve_name} Spot {as_of}",
            "component": component_map.get(curve_name, "base_rate"),
            "currency": currency_map.get(curve_name, "EUR"),
            "version": 1,
            "status": "draft",
            "tenors_json": json.dumps(tenors),
            "values_json": json.dumps([round(v, 6) for v in values]),
            "source": "generated",
            "created_at": datetime.now().isoformat(),
        })

    return result


def build_runoff_models(nmd_profiles: list[dict]) -> list[dict]:
    """Convert nmd_behavioral_profiles.json into app runoff_models format."""
    result = []
    for p in nmd_profiles:
        profile_values = [entry["remaining_ratio"] for entry in p.get("profile", [])]
        result.append({
            "id": f"rm-{p['profile_name'].lower().replace(' ', '-')[:30]}",
            "name": p["profile_name"],
            "product_type": "nmd",
            "category": "retail",
            "version": 1,
            "status": "draft",
            "method": "behavioral_exponential",
            "profile_json": json.dumps([round(v, 6) for v in profile_values]),
            "parameters_json": json.dumps({
                "lambda": p.get("lambda_decay"),
                "wal": p.get("wal_months"),
                "core_ratio": p.get("core_ratio"),
                "eba_capped": p.get("wal_months", 0) >= 60,
            }),
            "created_at": datetime.now().isoformat(),
        })
    return result


def build_ftp_ready_contracts(
    contracts: list[dict],
    curves_data: dict,
    sales_index: dict,
) -> list[dict]:
    """
    For each contract, compute profiles_json and rates_json for the FTP engine.
    Stores all QuantLib fields for future contractual schedule computation.
    """
    result = []
    as_of = date.today()

    for c in contracts:
        # Remaining tenor in months
        tenor = int(c.get("tenor_months") or 0)
        if tenor <= 0 and c.get("maturity_date"):
            try:
                mat = date.fromisoformat(c["maturity_date"])
                tenor = max(1, (mat.year - as_of.year) * 12 + (mat.month - as_of.month))
            except ValueError:
                tenor = 12

        # Buckets: FTP tenors up to contract maturity (+ 1 terminal bucket)
        buckets = [m for m in FTP_TENORS_MONTHS if m <= tenor]
        if not buckets:
            buckets = [min(FTP_TENORS_MONTHS[0], tenor)]
        buckets.append(tenor)  # terminal point = 0

        # Amortisation profile
        amt_type = c.get("amortization_type", "linear")
        is_deposit = c.get("side", "ASSET").upper() == "PASSIF"
        if is_deposit:
            amt_type = "bullet"  # deposits: full notional until maturity
        profile = build_amortisation_profile(amt_type, tenor, buckets)

        # Market rates: interpolate from benchmark curve matching currency
        currency = c.get("currency", "EUR")
        curve_name = map_currency_to_curve(currency)
        curve_data = curves_data.get(curve_name, {})
        spot = curve_data.get("spot_rates", {})

        # Rate row: one per inter-bucket gap (len(buckets) - 1 values)
        rates_row = []
        for m in buckets[:-1]:
            rates_row.append(round(interpolate_rate(spot, m), 6))

        result.append({
            # Identity
            "id": str(uuid.uuid4()),
            "contract_id": c["contract_id"],
            "contract_type": c["contract_type"],
            "side": c.get("side", "ASSET"),
            # Organisation
            "seller_id": c.get("seller_id", ""),
            "branch_code": c.get("branch_code", ""),
            "currency": currency,
            "rating": c.get("rating", ""),
            # Financial terms
            "notional": float(c.get("notional", 0)),
            "rate_type": c.get("rate_type", "fixed"),
            "interest_rate": float(c.get("interest_rate", 0)),
            "spread_over_index": float(c.get("spread_over_index", 0) or 0),
            # QuantLib schedule terms
            "settlement_date": c.get("settlement_date", ""),
            "maturity_date": c.get("maturity_date", ""),
            "tenor_months": tenor,
            "payment_frequency": c.get("payment_frequency", ""),
            "day_count": c.get("day_count", ""),
            "business_day_convention": c.get("business_day_convention", ""),
            "amortization_type": c.get("amortization_type", ""),
            "prepayment_allowed": c.get("prepayment_allowed", "False").lower() == "true",
            "prepayment_penalty": float(c.get("prepayment_penalty", 0) or 0),
            "guarantee_type": c.get("guarantee_type", ""),
            # FTP computation
            "profiles_json": json.dumps(profile),
            "rates_json": json.dumps(rates_row),
            "risk_weight": 1.0,
            "created_at": datetime.now().isoformat(),
        })

    return result


def build_portfolio_positions(ftp_contracts: list[dict]) -> list[dict]:
    """
    Derive portfolio_positions rows from FTP-ready contracts.
    Maps contract fields to the app's portfolio import CSV format.
    """
    return [
        {
            "position_ref": c["contract_id"],
            "product_type": c["contract_type"].lower(),
            "branch":       c.get("branch_code", ""),
            "seller":       c.get("seller_id", ""),
            "currency":     c.get("currency", "EUR"),
            "outstanding":  c.get("notional", 0),
            "origination_date": c.get("settlement_date", ""),
            "maturity_date":    c.get("maturity_date", ""),
            "client_rate":  c.get("interest_rate", 0),
            "risk_weight":  c.get("risk_weight", 1.0),
            "profiles_json": c.get("profiles_json", ""),
            "rates_json":   c.get("rates_json", ""),
        }
        for c in ftp_contracts
    ]


# ── Writers ───────────────────────────────────────────────────────────────────

def write_csv(rows: list[dict], path: Path) -> int:
    if not rows:
        return 0
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)
    return len(rows)


def write_json(data: Any, path: Path) -> None:
    with open(path, "w") as f:
        json.dump(data, f, indent=2, default=str)


def list_datasets() -> None:
    """List existing generated datasets."""
    found = False
    for d in sorted(SCRIPT_DIR.iterdir()):
        meta = d / "dataset_meta.json"
        if d.is_dir() and meta.exists():
            with open(meta) as f:
                m = json.load(f)
            print(f"  {m['id'][:8]}  {m['name']:<30}  {m['as_of_date']}  "
                  f"{m.get('contracts_count', '?')} contracts")
            found = True
    if not found:
        print("  (aucun dataset généré — lancez sans --list)")


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(description="FTP Dataset Generator")
    parser.add_argument("--name", default=f"Dataset {date.today().isoformat()}",
                        help="Nom du dataset")
    parser.add_argument("--description", default="",
                        help="Description du dataset")
    parser.add_argument("--max-contracts", type=int, default=None,
                        help="Nombre maximum de contrats (défaut: tous)")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--filter-side", default=None,
                        help="Filtrer par côté: ASSET ou PASSIF")
    parser.add_argument("--filter-type", default=None,
                        help="Filtrer par type(s), virgule-séparé: MORTGAGE,PAM,ANNUITE")
    parser.add_argument("--filter-branch", default=None,
                        help="Filtrer par branche(s): FR,ES ou US ou DE")
    parser.add_argument("--filter-currency", default=None,
                        help="Filtrer par devise(s): EUR,USD,GBP")
    parser.add_argument("--filter-max-tenor", type=int, default=None,
                        help="Exclure les contrats avec tenor > N mois")
    parser.add_argument("--list", action="store_true", help="Lister les datasets existants")
    args = parser.parse_args()

    if args.list:
        list_datasets()
        return

    dataset_id = str(uuid.uuid4())
    out_dir = SCRIPT_DIR / dataset_id[:8]
    out_dir.mkdir(exist_ok=True)

    print(f"\n==> Génération du dataset : {args.name}")
    print(f"    ID     : {dataset_id}")
    print(f"    Dossier: {out_dir}\n")

    # ── Load source data ──────────────────────────────────────────────────────
    print("  Chargement des sources...")
    contracts_raw = load_contracts(None)  # load all, filter below

    # Apply filters
    if args.filter_side:
        side = args.filter_side.upper()
        contracts_raw = [c for c in contracts_raw if c.get("side", "ASSET").upper() == side]
    if args.filter_type:
        types = {t.strip().upper() for t in args.filter_type.split(",")}
        contracts_raw = [c for c in contracts_raw if c.get("contract_type", "").upper() in types]
    if args.filter_branch:
        branches_filter = {b.strip().upper() for b in args.filter_branch.split(",")}
        contracts_raw = [c for c in contracts_raw if c.get("branch_code", "").upper() in branches_filter]
    if args.filter_currency:
        currencies = {c.strip().upper() for c in args.filter_currency.split(",")}
        contracts_raw = [c for c in contracts_raw if c.get("currency", "").upper() in currencies]
    if args.filter_max_tenor:
        contracts_raw = [c for c in contracts_raw
                         if int(c.get("tenor_months") or 999) <= args.filter_max_tenor]
    if args.max_contracts:
        contracts_raw = contracts_raw[:args.max_contracts]
    curves_data   = load_current_curves()
    nmd_profiles  = load_nmd_profiles()
    sales_index   = load_sales_index()

    # Load entity data
    branches_all      = load_entity_csv("branches.csv")
    business_units_all = load_entity_csv("business_units.csv")
    departments_all   = load_entity_csv("departments.csv")
    sellers_all       = load_entity_csv("sales.csv")
    treasuries_all    = load_entity_csv("treasuries.csv")
    historical_series = load_historical_series()

    print(f"    {len(contracts_raw)} contrats, "
          f"{len(curves_data)} courbes de taux, "
          f"{len(nmd_profiles)} profils NMD")

    # ── Build outputs ─────────────────────────────────────────────────────────
    print("\n  Construction des outputs...")

    rate_curves    = build_rate_curves(curves_data)
    runoff_models  = build_runoff_models(nmd_profiles)
    ftp_contracts  = build_ftp_ready_contracts(contracts_raw, curves_data, sales_index)
    positions      = build_portfolio_positions(ftp_contracts)

    # Filter entities to those referenced by this dataset's contracts
    entities = filter_entities(
        contracts_raw,
        branches_all, business_units_all, departments_all,
        sellers_all, treasuries_all,
    )

    # ── Write outputs ─────────────────────────────────────────────────────────
    print("\n  Écriture des fichiers...")

    n_pos = write_csv(positions, out_dir / "portfolio_positions.csv")
    print(f"    portfolio_positions.csv    : {n_pos} lignes")

    n_ctr = write_csv(ftp_contracts, out_dir / "contracts.csv")
    print(f"    contracts.csv              : {n_ctr} lignes")

    n_rc = write_csv(rate_curves, out_dir / "rate_curves.csv")
    print(f"    rate_curves.csv            : {n_rc} lignes")

    write_json(runoff_models, out_dir / "runoff_models.json")
    print(f"    runoff_models.json         : {len(runoff_models)} modèles")

    # Entities
    n_br = write_csv(entities["branches"],       out_dir / "entities_branches.csv")
    n_bu = write_csv(entities["business_units"], out_dir / "entities_business_units.csv")
    n_dp = write_csv(entities["departments"],    out_dir / "entities_departments.csv")
    n_sl = write_csv(entities["sellers"],        out_dir / "entities_sellers.csv")
    n_tr = write_csv(entities["treasuries"],     out_dir / "entities_treasuries.csv")
    print(f"    entities_branches.csv      : {n_br} lignes")
    print(f"    entities_business_units.csv: {n_bu} lignes")
    print(f"    entities_departments.csv   : {n_dp} lignes")
    print(f"    entities_sellers.csv       : {n_sl} lignes")
    print(f"    entities_treasuries.csv    : {n_tr} lignes")

    # Historical rate series (all, not filtered — global reference data)
    n_rs = write_csv(historical_series, out_dir / "rate_series.csv")
    print(f"    rate_series.csv            : {n_rs} lignes")

    # ── Dataset metadata ──────────────────────────────────────────────────────
    meta = {
        "id":               dataset_id,
        "name":             args.name,
        "description":      args.description or f"Généré depuis les données sources ({date.today().isoformat()})",
        "status":           "active",
        "source":           "generated",
        "as_of_date":       date.today().isoformat(),
        "contracts_count":  n_ctr,
        "rate_curves_count": n_rc,
        "runoff_models_count": len(runoff_models),
        "rate_series_count": n_rs,
        "entities": {
            "branches": n_br, "business_units": n_bu,
            "departments": n_dp, "sellers": n_sl, "treasuries": n_tr,
        },
        "files": {
            "contracts":             "contracts.csv",
            "portfolio_positions":   "portfolio_positions.csv",
            "rate_curves":           "rate_curves.csv",
            "runoff_models":         "runoff_models.json",
            "rate_series":           "rate_series.csv",
            "entities_branches":     "entities_branches.csv",
            "entities_business_units": "entities_business_units.csv",
            "entities_departments":  "entities_departments.csv",
            "entities_sellers":      "entities_sellers.csv",
            "entities_treasuries":   "entities_treasuries.csv",
        },
        "generated_at": datetime.now().isoformat(),
    }
    write_json(meta, out_dir / "dataset_meta.json")

    print(f"\n==> Dataset prêt dans {out_dir}")
    print(f"    Charger via l'app : POST /api/datasets/fs/{out_dir.name}/load")


if __name__ == "__main__":
    main()

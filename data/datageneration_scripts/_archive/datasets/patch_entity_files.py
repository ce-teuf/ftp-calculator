#!/usr/bin/env python3
"""
patch_entity_files.py — Adds entity CSVs and rate_series.csv to existing dataset folders.
Safe to run multiple times (idempotent).
"""
import csv, json
from pathlib import Path

SCRIPT_DIR   = Path(__file__).parent.resolve()
ROOT_DIR     = SCRIPT_DIR.parent
ENTITIES_DIR = ROOT_DIR / "data_entities"
RATES_DIR    = ROOT_DIR / "data_rate_series"


def load_csv(path: Path) -> list[dict]:
    if not path.exists():
        return []
    with open(path) as f:
        return list(csv.DictReader(f))


def write_csv(rows: list[dict], path: Path) -> int:
    if not rows:
        return 0
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)
    return len(rows)


def load_historical() -> list[dict]:
    result = []
    for name, fname in [("ESTR","historical_estr.csv"), ("EURIBOR","historical_euribor.csv"), ("SOFR","historical_sofr.csv")]:
        path = RATES_DIR / fname
        if not path.exists():
            continue
        with open(path) as f:
            for row in csv.DictReader(f):
                result.append({"series_name": name, "obs_date": row.get("date",""), "tenor": row.get("tenor",""), "rate": row.get("rate","")})
    return result


def patch_dataset(folder: Path, all_entities: dict, historical: list[dict]) -> None:
    meta_path = folder / "dataset_meta.json"
    if not meta_path.exists():
        return

    with open(meta_path) as f:
        meta = json.load(f)

    # Read contracts to get referenced entity IDs
    contracts_path = folder / "contracts.csv"
    if not contracts_path.exists():
        print(f"  {folder.name}: no contracts.csv, skip")
        return

    contracts = load_csv(contracts_path)
    branch_codes = {c.get("branch_code","") for c in contracts if c.get("branch_code")}
    seller_ids   = {c.get("seller_id","")   for c in contracts if c.get("seller_id")}

    branches = [b for b in all_entities["branches"] if b.get("branch_code") in branch_codes]
    branch_ids = {b["branch_id"] for b in branches}

    # Sellers: match by seller_id if found, else fall back to branch_code match
    sellers  = [s for s in all_entities["sellers"]  if s.get("seller_id") in seller_ids]
    if not sellers:
        sellers = [s for s in all_entities["sellers"] if s.get("branch_code") in branch_codes]
    bu_ids   = {s["bu_id"] for s in sellers if s.get("bu_id")}

    bus      = [b for b in all_entities["business_units"] if b.get("bu_id") in bu_ids or b.get("branch_id") in branch_ids]
    all_bu   = bu_ids | {b["bu_id"] for b in bus}

    depts    = [d for d in all_entities["departments"] if d.get("bu_id") in all_bu]
    treas    = [t for t in all_entities["treasuries"]  if t.get("branch_id") in branch_ids]

    n_br = write_csv(branches, folder / "entities_branches.csv")
    n_bu = write_csv(bus,      folder / "entities_business_units.csv")
    n_dp = write_csv(depts,    folder / "entities_departments.csv")
    n_sl = write_csv(sellers,  folder / "entities_sellers.csv")
    n_tr = write_csv(treas,    folder / "entities_treasuries.csv")
    n_rs = write_csv(historical, folder / "rate_series.csv")

    # Update meta
    meta.setdefault("entities", {})
    meta["entities"] = {"branches": n_br, "business_units": n_bu, "departments": n_dp, "sellers": n_sl, "treasuries": n_tr}
    meta["rate_series_count"] = n_rs
    meta.setdefault("files", {})
    meta["files"].update({
        "rate_series": "rate_series.csv",
        "entities_branches": "entities_branches.csv",
        "entities_business_units": "entities_business_units.csv",
        "entities_departments": "entities_departments.csv",
        "entities_sellers": "entities_sellers.csv",
        "entities_treasuries": "entities_treasuries.csv",
    })

    with open(meta_path, "w") as f:
        json.dump(meta, f, indent=2, default=str)

    print(f"  {folder.name} ({meta['name'][:30]}): "
          f"{n_br} branches, {n_bu} BUs, {n_dp} depts, {n_sl} sellers, {n_tr} treas, {n_rs} rate series")


def main():
    print("Loading entity source files...")
    all_entities = {
        "branches":       load_csv(ENTITIES_DIR / "branches.csv"),
        "business_units": load_csv(ENTITIES_DIR / "business_units.csv"),
        "departments":    load_csv(ENTITIES_DIR / "departments.csv"),
        "sellers":        load_csv(ENTITIES_DIR / "sales.csv"),
        "treasuries":     load_csv(ENTITIES_DIR / "treasuries.csv"),
    }
    historical = load_historical()
    print(f"  {sum(len(v) for v in all_entities.values())} entity rows, {len(historical)} rate series rows")

    print("\nPatching dataset folders...")
    for d in sorted(SCRIPT_DIR.iterdir()):
        if d.is_dir() and (d / "dataset_meta.json").exists():
            patch_dataset(d, all_entities, historical)

    print("\nDone.")


if __name__ == "__main__":
    main()

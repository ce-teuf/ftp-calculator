#!/usr/bin/env python3
"""
load_to_db.py — Génère 5 datasets FTP et les charge directement en PostgreSQL.

Usage:
    python load_to_db.py [--db-url postgresql://ftp:ftp_local@127.0.0.1:5432/ftp_simulator]
"""
import csv, json, sys, uuid, math, argparse
from datetime import datetime, date
from pathlib import Path

# ── Install psycopg2 si absent ────────────────────────────────────────────────
try:
    import psycopg2
    from psycopg2.extras import execute_values
except ImportError:
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "psycopg2-binary", "-q",
                           "--break-system-packages"])
    import psycopg2
    from psycopg2.extras import execute_values

# ── Chemins ───────────────────────────────────────────────────────────────────
BASE = Path(__file__).parent.parent
CONTRACTS_CSV    = BASE / "data_contracts"  / "contracts.csv"
CURVES_JSON      = BASE / "data_rate_series" / "current_curves.json"
NMD_JSON         = BASE / "data_schedules"  / "nmd_behavioral_profiles.json"
SALES_CSV        = BASE / "data_entities"   / "sales.csv"

# ── Tenors standard FTP ───────────────────────────────────────────────────────
FTP_TENOR_LABELS  = ["1M","3M","6M","1Y","2Y","3Y","5Y","7Y","10Y","15Y","20Y","30Y"]
FTP_TENOR_MONTHS  = [1,   3,   6,   12,  24,  36,  60,  84,  120,  180,  240,  360 ]

# Conversion label → mois pour les courbes source
SOURCE_LABEL_MONTHS = {
    "1D": 1/30, "1W": 7/30, "2W": 14/30,
    "1M": 1,  "2M": 2,  "3M": 3,  "6M": 6,  "9M": 9,
    "1Y": 12, "2Y": 24, "3Y": 36, "5Y": 60, "7Y": 84,
    "10Y": 120, "15Y": 180, "20Y": 240, "30Y": 360,
}

# Devise → nom de courbe principale
CCY_TO_CURVE = {"EUR": "ESTR", "USD": "SOFR", "GBP": "SONIA"}

# ── Risk weights Basel ────────────────────────────────────────────────────────
RISK_WEIGHTS = {
    "MORTGAGE": 0.5, "PAM": 1.0, "ANNUITE": 1.0,
    "BULLET": 1.0, "COMMERCIAL_LOAN": 1.0, "REVOLVER": 0.75,
    "DEMAND_DEPOSIT": 0.0, "SAVINGS": 0.0,
    "TERM_DEPOSIT": 0.0, "CERTIFICATE_OF_DEPOSIT": 0.0,
}


# ─────────────────────────────────────────────────────────────────────────────
# HELPERS
# ─────────────────────────────────────────────────────────────────────────────

def interp(x: float, xs: list, ys: list) -> float:
    """Interpolation linéaire (extrapolation plate aux bords)."""
    if x <= xs[0]:  return ys[0]
    if x >= xs[-1]: return ys[-1]
    for i in range(len(xs) - 1):
        if xs[i] <= x <= xs[i+1]:
            t = (x - xs[i]) / (xs[i+1] - xs[i])
            return ys[i] + t * (ys[i+1] - ys[i])
    return ys[-1]


def build_curve_lookup(spot_rates: dict) -> tuple[list, list]:
    """Retourne (months_list, rates_list) triés depuis spot_rates dict."""
    pairs = [(SOURCE_LABEL_MONTHS[k], v / 100.0)   # ÷100 : % → décimal
             for k, v in spot_rates.items()
             if k in SOURCE_LABEL_MONTHS]
    pairs.sort(key=lambda p: p[0])
    return [p[0] for p in pairs], [p[1] for p in pairs]


def make_rates_json(currency: str, tenor_months: int, curves_lookup: dict) -> str:
    """Vecteur de 12 taux FTP interpolés (décimal) aux tenors standard."""
    curve_name = CCY_TO_CURVE.get(currency, "ESTR")
    xs, ys = curves_lookup.get(curve_name, curves_lookup["ESTR"])
    t = max(1, min(tenor_months, 360))
    return json.dumps([round(interp(min(b, t), xs, ys), 6) for b in FTP_TENOR_MONTHS])


def make_profile_json(amort_type: str, side: str, tenor_months: int) -> str:
    """Vecteur de 12 valeurs [0..1] de profil d'amortissement aux tenors standard."""
    t = max(1, tenor_months)
    profile = []
    for b in FTP_TENOR_MONTHS:
        if b >= t:
            v = 0.0
        elif side == "PASSIF" or amort_type == "bullet":
            v = 1.0
        elif amort_type in ("linear", "constant_installment"):
            v = max(0.0, 1.0 - b / t)
        else:
            v = max(0.0, 1.0 - b / t)
        profile.append(round(v, 6))
    return json.dumps(profile)


def new_id() -> str:
    return str(uuid.uuid4())


# ─────────────────────────────────────────────────────────────────────────────
# CHARGEMENT DES SOURCES
# ─────────────────────────────────────────────────────────────────────────────

def load_contracts() -> list[dict]:
    print("  Lecture contracts.csv …", end=" ", flush=True)
    rows = []
    with open(CONTRACTS_CSV, newline="", encoding="utf-8") as f:
        for r in csv.DictReader(f):
            r["tenor_months"] = int(r["tenor_months"]) if r["tenor_months"] else 12
            r["notional"]     = float(r["notional"])
            r["interest_rate"]= float(r["interest_rate"]) if r["interest_rate"] else None
            r["prepayment_allowed"] = r["prepayment_allowed"].lower() == "true"
            r["prepayment_penalty"] = float(r["prepayment_penalty"]) if r["prepayment_penalty"] else 0.0
            rows.append(r)
    print(f"{len(rows)} contrats chargés.")
    return rows


def load_curves() -> dict:
    print("  Lecture current_curves.json …", end=" ", flush=True)
    with open(CURVES_JSON, encoding="utf-8") as f:
        raw = json.load(f)
    lookup = {}
    for name, data in raw.items():
        lookup[name] = build_curve_lookup(data["spot_rates"])
    print(f"{list(lookup.keys())} OK.")
    return lookup


def load_nmd_profiles() -> list[dict]:
    print("  Lecture nmd_behavioral_profiles.json …", end=" ", flush=True)
    with open(NMD_JSON, encoding="utf-8") as f:
        profiles = json.load(f)
    print(f"{len(profiles)} profils NMD chargés.")
    return profiles


# ─────────────────────────────────────────────────────────────────────────────
# INSERTIONS DB
# ─────────────────────────────────────────────────────────────────────────────

def insert_dataset(cur, name: str, description: str, source: str, as_of_date: str) -> str:
    did = new_id()
    cur.execute("""
        INSERT INTO datasets (id, name, description, status, source, as_of_date)
        VALUES (%s, %s, %s, 'active', %s, %s)
        ON CONFLICT (id) DO NOTHING
    """, (did, name, description, source, as_of_date))
    return did


def insert_rate_curves(cur, dataset_id: str, curves_raw: dict,
                       curve_names: list[str]) -> list[str]:
    """Insère les courbes et renvoie leurs IDs."""
    ids = []
    for name in curve_names:
        if name not in curves_raw:
            continue
        data    = curves_raw[name]
        xs, ys  = data  # (months_list, rates_list)
        # On retient les 12 tenors standard
        rates   = [round(interp(b, xs, ys), 6) for b in FTP_TENOR_MONTHS]
        ccy     = {"SOFR": "USD", "ESTR": "EUR", "SONIA": "GBP", "EURIBOR": "EUR"}[name]
        comp    = "base_rate" if name != "EURIBOR" else "ibor"
        cid     = new_id()
        cur.execute("""
            INSERT INTO rate_curves
                (id, name, component, currency, version, status,
                 tenors_json, values_json, source)
            VALUES (%s, %s, %s, %s, 1, 'approved', %s, %s, %s)
        """, (cid, name, comp, ccy,
              json.dumps(FTP_TENOR_LABELS),
              json.dumps(rates),
              "generated (realistic) — 2026-04-09"))
        cur.execute("""
            INSERT INTO dataset_items (dataset_id, entity_type, entity_id)
            VALUES (%s, 'rate_curve', %s) ON CONFLICT DO NOTHING
        """, (dataset_id, cid))
        ids.append(cid)
    return ids


def insert_nmd_models(cur, dataset_id: str, profiles: list[dict]) -> list[str]:
    """Insère les modèles NMD et renvoie leurs IDs."""
    ids = []
    NMD_PRODUCT_MAP = {
        "Retail Demand Deposits Core":    "DEMAND_DEPOSIT",
        "Retail Demand Deposits Volatile":"DEMAND_DEPOSIT",
        "Retail Savings Account":         "SAVINGS",
        "Corporate Operating Account":    "DEMAND_DEPOSIT",
        "Corporate Cash Pool":            "DEMAND_DEPOSIT",
        "SME Deposits":                   "SAVINGS",
        "Payroll Account":                "DEMAND_DEPOSIT",
    }
    for p in profiles:
        # Profil mensuel → vecteur aux 12 tenors standard
        monthly = [row["remaining_ratio"] for row in p["profile"]]  # 120 valeurs
        profile_at_tenors = [
            round(monthly[min(b - 1, len(monthly) - 1)], 6)
            for b in FTP_TENOR_MONTHS
        ]
        mid = new_id()
        cur.execute("""
            INSERT INTO runoff_models
                (id, name, product_type, category, version, status,
                 method, profile_json, parameters_json)
            VALUES (%s, %s, %s, 'NMD', 1, 'approved', 'Behavioral', %s, %s)
        """, (mid,
              p["profile_name"],
              NMD_PRODUCT_MAP.get(p["profile_name"], "SAVINGS"),
              json.dumps(profile_at_tenors),
              json.dumps({"lambda": p["lambda_decay"],
                          "core_ratio": p["core_ratio"],
                          "wal_months": p["wal_months"],
                          "eba_capped": True})))
        cur.execute("""
            INSERT INTO dataset_items (dataset_id, entity_type, entity_id)
            VALUES (%s, 'runoff_model', %s) ON CONFLICT DO NOTHING
        """, (dataset_id, mid))
        ids.append(mid)
    return ids


def insert_contracts(cur, dataset_id: str, contracts: list[dict],
                     curves_lookup: dict) -> int:
    """Insère les contrats en batch et renvoie le nombre inséré."""
    rows = []
    item_rows = []
    for c in contracts:
        cid   = new_id()
        tenor = c["tenor_months"]
        profiles_json = make_profile_json(c["amortization_type"], c["side"], tenor)
        rates_json    = make_rates_json(c["currency"], tenor, curves_lookup)
        risk_w        = RISK_WEIGHTS.get(c["contract_type"], 1.0)

        rows.append((
            cid,
            c["contract_id"],
            c["contract_type"],
            c["side"],
            c.get("seller_id") or None,
            c.get("branch_code") or None,
            c.get("currency", "EUR"),
            c.get("rating") or None,
            c["notional"],
            c.get("rate_type") or None,
            c["interest_rate"],
            None,                           # spread_over_index
            c.get("settlement_date") or None,
            c.get("maturity_date") or None,
            tenor,
            c.get("payment_frequency") or None,
            c.get("day_count") or None,
            c.get("business_day_convention") or None,
            c.get("amortization_type") or None,
            c["prepayment_allowed"],
            c["prepayment_penalty"],
            c.get("guarantee_type") or None,
            profiles_json,
            rates_json,
            risk_w,
            dataset_id,
        ))
        item_rows.append((dataset_id, "contract", cid))

    execute_values(cur, """
        INSERT INTO contracts (
            id, contract_id, contract_type, side,
            seller_id, branch_code, currency, rating,
            notional, rate_type, interest_rate, spread_over_index,
            settlement_date, maturity_date, tenor_months,
            payment_frequency, day_count, business_day_convention,
            amortization_type, prepayment_allowed, prepayment_penalty,
            guarantee_type, profiles_json, rates_json, risk_weight,
            source_dataset_id
        ) VALUES %s
        ON CONFLICT (contract_id) DO NOTHING
    """, rows)

    execute_values(cur, """
        INSERT INTO dataset_items (dataset_id, entity_type, entity_id)
        VALUES %s ON CONFLICT DO NOTHING
    """, item_rows)

    return len(rows)


# ─────────────────────────────────────────────────────────────────────────────
# DÉFINITION DES 5 DATASETS
# ─────────────────────────────────────────────────────────────────────────────

def dataset_definitions():
    """Retourne la liste des 5 datasets avec leurs filtres et métadonnées."""
    return [
        {
            "name":        "Retail Mortgages EUR — Branches FR & ES",
            "description": "Prêts immobiliers et amortissables retail, devise EUR, branches France et Espagne. "
                           "Base pour calibration courbe FTP Retail EUR.",
            "as_of_date":  "2026-04-09",
            "curves":      ["ESTR", "EURIBOR"],
            "nmd":         False,
            "filter": lambda c: (
                c["contract_type"] in {"MORTGAGE", "ANNUITE", "PAM"}
                and c["currency"] == "EUR"
                and c["branch_code"] in {"FR", "ES"}
            ),
            "max": 500,
        },
        {
            "name":        "Corporate USD Book — Branch US",
            "description": "Portefeuille corporate et structuré en USD, Branch New York. "
                           "Inclut bullets, revolvers et commercial loans.",
            "as_of_date":  "2026-04-09",
            "curves":      ["SOFR"],
            "nmd":         False,
            "filter": lambda c: (
                c["contract_type"] in {"BULLET", "COMMERCIAL_LOAN", "REVOLVER"}
                and c["currency"] == "USD"
                and c["branch_code"] == "US"
            ),
            "max": 300,
        },
        {
            "name":        "Deposit Book — Passifs Toutes Branches",
            "description": "Ensemble des dépôts et passifs (demand deposits, épargne, term deposits, CDs) "
                           "sur toutes les branches. Inclut les 7 modèles NMD comportementaux.",
            "as_of_date":  "2026-04-09",
            "curves":      ["ESTR", "SOFR", "SONIA"],
            "nmd":         True,
            "filter": lambda c: c["side"] == "PASSIF",
            "max": 2000,    # prend tous les passifs (~1500)
        },
        {
            "name":        "Portefeuille Complet Multi-devise Q1 2026",
            "description": "Dataset de référence complet : tous types de contrats, toutes branches, "
                           "toutes devises (EUR/USD/GBP). Sert de base aux scénarios et backtests.",
            "as_of_date":  "2026-04-09",
            "curves":      ["ESTR", "SOFR", "SONIA", "EURIBOR"],
            "nmd":         True,
            "filter": lambda c: True,
            "max": 2000,
        },
        {
            "name":        "Short-term Revolving & Bullets ≤ 36 mois",
            "description": "Portefeuille court terme : revolvers et bullets de maturité ≤ 3 ans. "
                           "Sensibilité élevée aux taux courts. Toutes branches et devises.",
            "as_of_date":  "2026-04-09",
            "curves":      ["ESTR", "SOFR", "SONIA"],
            "nmd":         False,
            "filter": lambda c: (
                c["contract_type"] in {"REVOLVER", "BULLET"}
                and c["tenor_months"] <= 36
            ),
            "max": 300,
        },
    ]


# ─────────────────────────────────────────────────────────────────────────────
# MAIN
# ─────────────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Charge 5 datasets FTP en PostgreSQL")
    parser.add_argument("--db-url", default="postgresql://ftp:ftp_local@127.0.0.1:5432/ftp_simulator")
    args = parser.parse_args()

    print("=" * 64)
    print("  FTP Dataset Loader — 5 datasets → PostgreSQL")
    print("=" * 64)

    # ── Chargement des sources ──────────────────────────────────────────────
    print("\n[1/3] Chargement des fichiers sources …")
    all_contracts  = load_contracts()
    curves_raw     = load_curves()
    nmd_profiles   = load_nmd_profiles()

    # Courbes brutes (toutes)
    curves_lookup = curves_raw  # dict name → (xs, ys)

    # ── Connexion PostgreSQL ────────────────────────────────────────────────
    print(f"\n[2/3] Connexion à {args.db_url} …")
    try:
        conn = psycopg2.connect(args.db_url)
        conn.autocommit = False
        cur  = conn.cursor()
        print("  Connecté.")
    except Exception as e:
        print(f"  ERREUR connexion: {e}")
        sys.exit(1)

    # ── Insertion des 5 datasets ────────────────────────────────────────────
    print("\n[3/3] Génération et chargement des datasets …\n")
    definitions = dataset_definitions()

    for i, defn in enumerate(definitions, 1):
        print(f"  ─── Dataset {i}/5 : {defn['name']}")

        # Filtrage des contrats
        filtered = [c for c in all_contracts if defn["filter"](c)]
        sampled  = filtered[:defn["max"]]
        print(f"       Contrats filtrés : {len(filtered)} → échantillon : {len(sampled)}")

        try:
            # Créer le dataset
            did = insert_dataset(
                cur,
                name=defn["name"],
                description=defn["description"],
                source="generated",
                as_of_date=defn["as_of_date"],
            )

            # Insérer les courbes de taux
            rc_ids = insert_rate_curves(cur, did, curves_lookup, defn["curves"])
            print(f"       Courbes insérées : {len(rc_ids)} ({defn['curves']})")

            # Insérer les modèles NMD si demandé
            nmd_ids = []
            if defn["nmd"]:
                nmd_ids = insert_nmd_models(cur, did, nmd_profiles)
                print(f"       Modèles NMD insérés : {len(nmd_ids)}")

            # Insérer les contrats
            n = insert_contracts(cur, did, sampled, curves_lookup)
            print(f"       Contrats chargés en DB : {n}")

            conn.commit()
            print(f"       ✓ Dataset ID: {did}\n")

        except Exception as e:
            conn.rollback()
            print(f"       ✗ ERREUR: {e}\n")
            import traceback; traceback.print_exc()

    cur.close()
    conn.close()
    print("=" * 64)
    print("  Chargement terminé.")
    print("=" * 64)


if __name__ == "__main__":
    main()

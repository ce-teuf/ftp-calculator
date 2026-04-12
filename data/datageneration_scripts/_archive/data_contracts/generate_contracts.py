#!/usr/bin/env python3
"""
Data Contracts Generator
Generates CSV with millions of contracts using QuantLib parameters:
- PAM (Plain Amortizing Mortgage)
- ANNUITE (Fixed rate annuity loan)
- MORTGAGE (Residential mortgage)
- bullet loans
- revolving facilities
- deposits (passif)
"""

from __future__ import annotations

import csv
import uuid
import random
from datetime import date, timedelta
from os import path
from typing import TypedDict, Optional

OUTPUT_DIR = path.dirname(path.abspath(__file__))

random.seed(42)

QUANTLIB_AVAILABLE: bool = False

try:
    import QuantLib as ql
    QUANTLIB_AVAILABLE = True
    # ql.setGlobalEvaluationDate(ql.Date(9, 4, 2026))
    # ql.Settings.instance().setEvaluationDate(ql.Date(9, 4, 2026))
    ql.Settings.instance().evaluationDate = ql.Date(9, 4, 2026)
except ImportError:
    print("Warning: QuantLib not available. Using simplified contract generation.")


class ContractTypeConfig(TypedDict):
    weight: float
    min_tenor: int
    max_tenor: int


class MarketRatesFixed(TypedDict):
    fixed: tuple[float, float]


class MarketRatesFloating(TypedDict):
    spread: tuple[float, float]
    margin: float


class MarketRates(TypedDict):
    fixed: tuple[float, float]
    floating: MarketRatesFloating


class LoanContract(TypedDict):
    contract_id: str
    contract_type: str
    side: str
    seller_id: str
    branch_code: str
    currency: str
    notional: int
    rate_type: str
    interest_rate: float
    settlement_date: str
    maturity_date: str
    tenor_months: int
    payment_frequency: str
    day_count: str
    business_day_convention: str
    amortization_type: str
    prepayment_allowed: bool
    prepayment_penalty: float
    guarantee_type: str
    rating: str


class DepositContract(TypedDict):
    contract_id: str
    contract_type: str
    side: str
    treasury_id: str
    branch_code: str
    currency: str
    notional: int
    rate_type: str
    interest_rate: float
    settlement_date: str
    maturity_date: Optional[str]
    tenor_months: Optional[int]
    payment_frequency: str
    day_count: str
    early_withdrawal_penalty: float
    deposit_category: str


CONTRACT_TYPES: dict[str, ContractTypeConfig] = {
    "PAM": {"weight": 0.25, "min_tenor": 12, "max_tenor": 360},
    "ANNUITE": {"weight": 0.20, "min_tenor": 12, "max_tenor": 240},
    "MORTGAGE": {"weight": 0.25, "min_tenor": 60, "max_tenor": 360},
    "BULLET": {"weight": 0.15, "min_tenor": 3, "max_tenor": 60},
    "REVOLVER": {"weight": 0.10, "min_tenor": 12, "max_tenor": 60},
    "COMMERCIAL_LOAN": {"weight": 0.05, "min_tenor": 12, "max_tenor": 180}
}

ASSET_TYPES: list[str] = ["PAM", "ANNUITE", "MORTGAGE", "BULLET", "REVOLVER", "COMMERCIAL_LOAN"]
PASSIF_TYPES: list[str] = ["DEMAND_DEPOSIT", "SAVINGS", "TERM_DEPOSIT", "CERTIFICATE_OF_DEPOSIT"]
CURRENCIES: list[str] = ["EUR", "USD", "GBP"]
RATE_TYPES: list[str] = ["fixed", "floating"]

MARKET_RATES: dict[str, MarketRates] = {
    "EUR": {"fixed": (0.02, 0.06), "floating": {"spread": (0.01, 0.03), "margin": 0.04}},
    "USD": {"fixed": (0.04, 0.08), "floating": {"spread": (0.01, 0.025), "margin": 0.055}},
    "GBP": {"fixed": (0.03, 0.07), "floating": {"spread": (0.01, 0.02), "margin": 0.05}}
}


def generate_contract_id() -> str:
    return f"CNT-{uuid.uuid4().hex[:12].upper()}"


def generate_loan_contract(contract_type: str, seller_ids: list[str], branch_codes: list[str]) -> LoanContract:
    """Generate a loan contract (asset)."""
    currency = random.choice(CURRENCIES)
    market = MARKET_RATES[currency]
    rate_type = random.choice(RATE_TYPES)
    
    config = CONTRACT_TYPES[contract_type]
    tenor_months = random.randint(config["min_tenor"], config["max_tenor"])
    
    if rate_type == "fixed":
        rate = random.uniform(market["fixed"][0], market["fixed"][1])
    else:
        rate = market["floating"]["margin"] + random.uniform(
            market["floating"]["spread"][0], market["floating"]["spread"][1]
        )
    
    notional = random.randint(10000, 10000000)
    
    settlement_date = date.today() - timedelta(days=random.randint(30, 1825))
    maturity_date = settlement_date + timedelta(days=tenor_months * 30)
    
    contract: LoanContract = {
        "contract_id": generate_contract_id(),
        "contract_type": contract_type,
        "side": "ASSET",
        "seller_id": random.choice(seller_ids),
        "branch_code": random.choice(branch_codes),
        "currency": currency,
        "notional": notional,
        "rate_type": rate_type,
        "interest_rate": round(rate, 5),
        "settlement_date": settlement_date.isoformat(),
        "maturity_date": maturity_date.isoformat(),
        "tenor_months": tenor_months,
        "payment_frequency": random.choice(["monthly", "quarterly", "semiannual", "annual"]),
        "day_count": random.choice(["30/360", "ACT/360", "ACT/365", "ACT/ACT"]),
        "business_day_convention": random.choice(["Following", "ModifiedFollowing", "Preceding"]),
        "amortization_type": random.choice(["linear", "constant_installment", "bullet"]),
        "prepayment_allowed": random.choice([True, False]),
        "prepayment_penalty": round(random.uniform(0, 0.02), 4) if random.random() > 0.5 else 0,
        "guarantee_type": random.choice(["none", "personal", "mortgage", "commercial"]),
        "rating": random.choice(["AAA", "AA", "A", "BBB", "BB", "B", "NR"])
    }
    
    return contract


def generate_deposit_contract(contract_type: str, branch_codes: list[str], treasury_ids: list[str]) -> DepositContract:
    """Generate a deposit contract (passif)."""
    currency = random.choice(CURRENCIES)
    
    tenor_months = random.randint(1, 60) if contract_type != "DEMAND_DEPOSIT" else 1
    
    rate = random.uniform(0.001, 0.04)
    
    settlement_date = date.today() - timedelta(days=random.randint(30, 1095))
    maturity_date = settlement_date + timedelta(days=tenor_months * 30) if contract_type != "DEMAND_DEPOSIT" else settlement_date
    
    contract: DepositContract = {
        "contract_id": generate_contract_id(),
        "contract_type": contract_type,
        "side": "PASSIF",
        "treasury_id": random.choice(treasury_ids),
        "branch_code": random.choice(branch_codes),
        "currency": currency,
        "notional": random.randint(10000, 50000000),
        "rate_type": "fixed",
        "interest_rate": round(rate, 5),
        "settlement_date": settlement_date.isoformat(),
        "maturity_date": maturity_date.isoformat() if contract_type != "DEMAND_DEPOSIT" else None,
        "tenor_months": tenor_months if contract_type != "DEMAND_DEPOSIT" else None,
        "payment_frequency": "monthly" if contract_type in ["DEMAND_DEPOSIT", "SAVINGS"] else random.choice(["monthly", "quarterly", "annual"]),
        "day_count": "ACT/365",
        "early_withdrawal_penalty": round(random.uniform(0, 0.01), 4) if contract_type != "DEMAND_DEPOSIT" else 0,
        "deposit_category": random.choice(["core", "non_core"]) if contract_type == "DEMAND_DEPOSIT" else "core"
    }
    
    return contract


def generate_contracts(num_contracts: int, seller_ids: list[str], branch_codes: list[str], treasury_ids: list[str]) -> list[LoanContract | DepositContract]:
    """Generate all contracts."""
    contracts: list[LoanContract | DepositContract] = []
    
    asset_weights = [CONTRACT_TYPES[t]["weight"] for t in ASSET_TYPES]
    
    for _ in range(num_contracts):
        if random.random() < 0.85:
            contract_type = random.choices(ASSET_TYPES, weights=asset_weights)[0]
            contract = generate_loan_contract(contract_type, seller_ids, branch_codes)
        else:
            contract_type = random.choice(PASSIF_TYPES)
            contract = generate_deposit_contract(contract_type, branch_codes, treasury_ids)
        
        contracts.append(contract)
    
    return contracts


def write_csv(data: list[LoanContract | DepositContract], filename: str) -> str:
    """Write data to CSV file."""
    if not data:
        return ""
    
    keys = list(data[0].keys())
    
    filepath = path.join(OUTPUT_DIR, filename)
    with open(filepath, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=keys, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(data)
    
    print(f"Generated: {filepath} ({len(data)} records)")
    return filepath


def main(num_contracts: int = 10000) -> None:
    print("=" * 60)
    print("Data Contracts Generator")
    print(f"Generating {num_contracts} contracts (QuantLib: {'Yes' if QUANTLIB_AVAILABLE else 'No'})")
    print("=" * 60)
    
    seller_ids: list[str] = [f"SLR-{code}-{i:03d}" for code in ["US", "FR", "ES", "DE"] for i in range(1, 16)]
    branch_codes: list[str] = ["US", "FR", "ES", "DE"]
    treasury_ids: list[str] = [f"TR-{code}-001" for code in branch_codes]
    
    contracts = generate_contracts(num_contracts, seller_ids, branch_codes, treasury_ids)
    write_csv(contracts, "contracts.csv")
    
    stats: dict[str, int] = {"ASSET": 0, "PASSIF": 0}
    for c in contracts:
        stats[c["side"]] += 1
    
    print(f"\nStatistics:")
    print(f"  Total contracts: {len(contracts)}")
    print(f"  Assets (Loans): {stats['ASSET']}")
    print(f"  Passif (Deposits): {stats['PASSIF']}")
    print("=" * 60)


if __name__ == "__main__":
    import sys
    num = int(sys.argv[1]) if len(sys.argv) > 1 else 10000
    main(num)
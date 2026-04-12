#!/usr/bin/env python3
"""
FTP Curve Data Generator - Main Entry Point

Generates fake data for inaccessible FTP curve components:
- Credit Spread
- TLP (Treasury Liquidity Premium)
- CLP (Customer Liquidity Premium)
- OAS (Option Adjusted Spread)
- Basis Risk
- Operational Risk
- NMD behavioral model data
- Risk weights and RAROC parameters

Usage:
    python generate_all.py
"""

import os
import json
from datetime import date, timedelta
import numpy as np

# Set random seed for reproducibility
np.random.seed(42)

# Configuration
OUTPUT_DIR = os.path.dirname(os.path.abspath(__file__))
TENORS = ["1M", "3M", "6M", "1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "15Y", "20Y", "30Y"]


def generate_base_rate_curve():
    """Generate realistic SOFR OIS curve (base rate)"""
    base_rates = {
        "1M": 5.35, "3M": 5.33, "6M": 5.28, "1Y": 5.00,
        "2Y": 4.75, "3Y": 4.60, "5Y": 4.45, "7Y": 4.50,
        "10Y": 4.55, "15Y": 4.65, "20Y": 4.70, "30Y": 4.75
    }
    return base_rates


def generate_credit_spread_curve():
    """Generate credit spread curve by tenor (realistic bank internal data)"""
    # Credit spreads typically increase with tenor for corporates
    spreads = {
        "1M": 0.80, "3M": 0.90, "6M": 1.00, "1Y": 1.20,
        "2Y": 1.40, "3Y": 1.60, "5Y": 1.80, "7Y": 2.00,
        "10Y": 2.20, "15Y": 2.40, "20Y": 2.50, "30Y": 2.60
    }
    return spreads


def generate_tlp_curve():
    """Generate Treasury Liquidity Premium (TLP) curve - FHLB based"""
    # TLP varies by tenor, typically higher for longer maturities
    tlp = {
        "1M": 0.15, "3M": 0.18, "6M": 0.20, "1Y": 0.22,
        "2Y": 0.25, "3Y": 0.28, "5Y": 0.32, "7Y": 0.35,
        "10Y": 0.38, "15Y": 0.40, "20Y": 0.42, "30Y": 0.45
    }
    return tlp


def generate_clp_curve():
    """Generate Customer Liquidity Premium (CLP) curve"""
    # CLP depends on product type and customer segment
    clp_retail = {
        "1M": 0.10, "3M": 0.12, "6M": 0.15, "1Y": 0.18,
        "2Y": 0.20, "3Y": 0.22, "5Y": 0.25, "7Y": 0.28,
        "10Y": 0.30, "15Y": 0.32, "20Y": 0.34, "30Y": 0.35
    }
    clp_corporate = {
        "1M": 0.20, "3M": 0.25, "6M": 0.30, "1Y": 0.35,
        "2Y": 0.40, "3Y": 0.45, "5Y": 0.50, "7Y": 0.55,
        "10Y": 0.60, "15Y": 0.65, "20Y": 0.70, "30Y": 0.75
    }
    return {"retail": clp_retail, "corporate": clp_corporate}


def generate_oas_curve():
    """Generate Option Adjusted Spread curve"""
    # OAS by rating and tenor
    oas_bbb = {
        "1M": 1.20, "3M": 1.30, "6M": 1.40, "1Y": 1.50,
        "2Y": 1.60, "3Y": 1.70, "5Y": 1.85, "7Y": 2.00,
        "10Y": 2.15, "15Y": 2.30, "20Y": 2.40, "30Y": 2.50
    }
    oas_bb = {
        "1M": 3.00, "3M": 3.20, "6M": 3.50, "1Y": 3.80,
        "2Y": 4.20, "3Y": 4.50, "5Y": 5.00, "7Y": 5.50,
        "10Y": 6.00, "15Y": 6.50, "20Y": 7.00, "30Y": 7.50
    }
    return {"BBB": oas_bbb, "BB": oas_bb}


def generate_basis_risk():
    """Generate basis risk by product type"""
    basis_risk = {
        "fixed_rate_mortgage": 0.08,
        "floating_rate_loan": 0.15,
        "commercial_loan": 0.12,
        "consumer_loan": 0.10,
        "revolving_credit": 0.20
    }
    return basis_risk


def generate_operational_risk():
    """Generate operational risk premium by product type"""
    # Typically 5-15 bps depending on complexity
    op_risk = {
        "retail_mortgage": 0.08,
        "consumer_loan": 0.10,
        "commercial_loan": 0.12,
        "corporate_loan": 0.15,
        "trade_finance": 0.18,
        "derivatives": 0.20
    }
    return op_risk


def generate_nmd_behavioral_models():
    """Generate NMD behavioral model data"""
    models = [
        {
            "name": "Retail Demand Deposits Core",
            "type": "nmd",
            "category": "retail",
            "method": "behavioral_exponential",
            "lambda": 0.05,
            "core_ratio": 0.75,
            "profile": generate_decay_profile(0.05, 0.75, 120)
        },
        {
            "name": "Retail Savings Account",
            "type": "nmd",
            "category": "retail",
            "method": "behavioral_exponential",
            "lambda": 0.08,
            "core_ratio": 0.60,
            "profile": generate_decay_profile(0.08, 0.60, 120)
        },
        {
            "name": "Corporate Checking",
            "type": "nmd",
            "category": "corporate",
            "method": "behavioral_exponential",
            "lambda": 0.12,
            "core_ratio": 0.50,
            "profile": generate_decay_profile(0.12, 0.50, 120)
        },
        {
            "name": "SME Deposits",
            "type": "nmd",
            "category": "sme",
            "method": "behavioral_exponential",
            "lambda": 0.10,
            "core_ratio": 0.55,
            "profile": generate_decay_profile(0.10, 0.55, 120)
        }
    ]
    return models


def generate_decay_profile(lambda_decay, core_ratio, months=120):
    """Generate exponential decay profile for NMD modeling"""
    profile = []
    for t in range(1, months + 1):
        decay = np.exp(-lambda_decay * t)
        value = core_ratio + (1 - core_ratio) * decay
        profile.append(round(value, 4))
    return profile


def generate_risk_weights():
    """Generate Basel risk weights by product type"""
    risk_weights = {
        "residential_mortgage": 0.50,
        "commercial_mortgage": 0.50,
        "corporate_exposure": 0.75,
        "retail_exposure": 0.75,
        "sovereign_exposure": 0.00,
        "bank_exposure": 0.20,
        " SME_exposure": 0.75,
        "consumer_credit": 1.00,
        "unsecured_personal": 1.00,
        "revolving": 1.00,
        "trade_receivables": 1.00,
        "commercial_real_estate": 1.00
    }
    return risk_weights


def generate_raroc_parameters():
    """Generate RAROC parameters (Cost of Equity, hurdle rates)"""
    return {
        "coe_by_segment": {
            "retail": 0.12,
            "corporate": 0.11,
            "sme": 0.13,
            "commercial_real_estate": 0.10,
            "financial_institutions": 0.09,
            "sovereign": 0.08
        },
        "hurdle_rates": {
            "retail": 0.12,
            "corporate": 0.11,
            "sme": 0.14,
            "commercial_real_estate": 0.10,
            "commercial_banking": 0.09
        },
        "capital_ratio": 0.08,
        "tier1_ratio": 0.06,
        "capital_conservation_buffer": 0.025,
        "countercyclical_buffer": 0.00
    }


def generate_historical_rates(days=365 * 3):
    """Generate historical SOFR rates (fake but realistic)"""
    start_date = date.today() - timedelta(days=days)
    dates = []
    values = []
    
    base_rate = 0.05
    for i in range(days):
        current_date = start_date + timedelta(days=i)
        
        # Add some realistic variation (volatility + trend)
        if current_date.year == 2022 and current_date.month >= 3:
            # Rate hike period
            trend = 0.0003 * i
        elif current_date.year == 2023 and current_date.month <= 9:
            # High rates plateau
            trend = 0.001
        else:
            trend = 0
        
        noise = np.random.normal(0, 0.001)
        rate = base_rate + trend + noise
        rate = max(0.01, min(0.08, rate))  # Bound between 1% and 8%
        
        dates.append(current_date.isoformat())
        values.append(round(rate * 100, 4))  # Convert to percentage
    
    return {"dates": dates, "values": values}


def generate_all_curves():
    """Generate all curve components and save to JSON"""
    curves = {
        "base_rate": generate_base_rate_curve(),
        "credit_spread": generate_credit_spread_curve(),
        "tlp": generate_tlp_curve(),
        "clp": generate_clp_curve(),
        "oas": generate_oas_curve(),
        "basis_risk": generate_basis_risk(),
        "operational_risk": generate_operational_risk(),
        "risk_weights": generate_risk_weights(),
        "raroc_parameters": generate_raroc_parameters(),
        "historical_sofr": generate_historical_rates(),
        "nmd_models": generate_nmd_behavioral_models()
    }
    
    # Save to JSON
    output_path = os.path.join(OUTPUT_DIR, "generated_curves.json")
    with open(output_path, "w") as f:
        json.dump(curves, f, indent=2)
    
    print(f"Generated curves saved to: {output_path}")
    return curves


def generate_portfolio_sample():
    """Generate sample portfolio data for testing"""
    import random
    
    products = [
        ("RET_MORTGAGE_001", "residential_mortgage", "Paris Centre", "Seller_A", 2500000, 0.045),
        ("RET_MORTGAGE_002", "residential_mortgage", "Lyon Nord", "Seller_B", 1800000, 0.042),
        ("RET_MORTGAGE_003", "residential_mortgage", "Marseille", "Seller_A", 3200000, 0.048),
        ("CORP_LOAN_001", "corporate_exposure", "Corporate Paris", "Seller_C", 5000000, 0.055),
        ("CORP_LOAN_002", "corporate_exposure", "Corporate Lyon", "Seller_D", 3500000, 0.052),
        ("SME_LOAN_001", "SME_exposure", "SME Nice", "Seller_E", 800000, 0.065),
        ("SME_LOAN_002", "SME_exposure", "SME Bordeaux", "Seller_E", 650000, 0.068),
        ("CONSUMER_001", "consumer_credit", "Retail", "Seller_A", 50000, 0.120),
        ("CONSUMER_002", "consumer_credit", "Retail", "Seller_B", 35000, 0.115),
        ("COMMERCIAL_RE_001", "commercial_real_estate", "Paris CBD", "Seller_C", 8000000, 0.055),
    ]
    
    portfolio = []
    for i, (prod_id, prod_type, branch, seller, outstanding, rate) in enumerate(products):
        portfolio.append({
            "product_id": prod_id,
            "product_type": prod_type,
            "branch": branch,
            "seller": seller,
            "outstanding": outstanding,
            "client_rate": rate,
            "currency": "EUR",
            "origination_date": (date.today() - timedelta(days=random.randint(180, 1095))).isoformat(),
            "maturity_date": (date.today() + timedelta(days=random.randint(365, 3650))).isoformat(),
            "risk_weight": generate_risk_weights().get(prod_type, 0.75)
        })
    
    output_path = os.path.join(OUTPUT_DIR, "sample_portfolio.json")
    with open(output_path, "w") as f:
        json.dump({"portfolio": portfolio, "as_of_date": date.today().isoformat()}, f, indent=2)
    
    print(f"Sample portfolio saved to: {output_path}")
    return portfolio


if __name__ == "__main__":
    print("=" * 60)
    print("FTP Curve Data Generator")
    print("Generating fake data for inaccessible FTP components")
    print("=" * 60)
    
    generate_all_curves()
    generate_portfolio_sample()
    
    print("\n" + "=" * 60)
    print("Generation complete!")
    print("=" * 60)
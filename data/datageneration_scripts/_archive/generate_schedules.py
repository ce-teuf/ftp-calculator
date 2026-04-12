#!/usr/bin/env python3
"""
Data Schedules Generator
Generates amortization schedules and behavioral profiles for:
- NMD (Non-Maturity Deposits): demand deposits, savings
- Revolving facilities
- Credit lines
- Custom behavioral profiles
"""

from __future__ import annotations

import csv
import json
import random
import math
from datetime import date, timedelta
from os import path
from typing import TypedDict

OUTPUT_DIR = path.dirname(path.abspath(__file__))
random.seed(42)

MONTHS_PROJECTION = 120


class ScheduleRow(TypedDict):
    period: int
    date: str
    payment: float
    principal: float
    interest: float
    outstanding: float


class NMDProfileRow(TypedDict):
    month: int
    remaining_ratio: float
    core_component: float
    volatile_component: float


class NMDModel(TypedDict):
    profile_name: str
    lambda_decay: float
    core_ratio: float
    projection_months: int
    wal_months: float
    profile: list[NMDProfileRow]


class RevolvingProfileRow(TypedDict):
    month: int
    outstanding: float
    commitment: float
    utilization_ratio: float
    available: float


class RevolvingProfile(TypedDict):
    profile_name: str
    commitment: float
    avg_utilization: float
    profile: list[RevolvingProfileRow]


class CreditLineProfileRow(TypedDict):
    month: int
    outstanding: float
    commitment: float
    drawdown_ratio: float


class CreditLineProfile(TypedDict):
    profile_name: str
    commitment: float
    avg_drawdown: float
    profile: list[CreditLineProfileRow]


class DecayingPoolProfileRow(TypedDict):
    month: int
    outstanding: float
    remaining_ratio: float
    amortized: float


class DecayingPoolProfile(TypedDict):
    profile_name: str
    initial_amount: float
    half_life_months: int
    profile: list[DecayingPoolProfileRow]


def generate_linear_amortization(notional: float, start_date: date, months: int, payment_freq: str = "monthly") -> list[ScheduleRow]:
    """Generate linear amortization schedule."""
    payments_per_year = {"monthly": 12, "quarterly": 4, "semiannual": 2, "annual": 1}[payment_freq]
    total_payments = months * 12 // (12 // payments_per_year)
    payment_amount = notional / total_payments
    
    schedule: list[ScheduleRow] = []
    current_date = start_date
    remaining = notional
    
    while remaining > 0:
        interest = remaining * 0.04 / payments_per_year
        principal = min(payment_amount, remaining)
        
        schedule.append({
            "period": len(schedule) + 1,
            "date": current_date.isoformat(),
            "payment": round(principal + interest, 2),
            "principal": round(principal, 2),
            "interest": round(interest, 2),
            "outstanding": round(remaining - principal, 2)
        })
        
        remaining -= principal
        current_date = _increment_date(current_date, payment_freq)
        
        if len(schedule) >= total_payments:
            break
    
    return schedule


def generate_constant_installment_amortization(notional: float, start_date: date, months: int, rate: float, payment_freq: str = "monthly") -> list[ScheduleRow]:
    """Generate constant installment (annuity) schedule."""
    payments_per_year = {"monthly": 12, "quarterly": 4, "semiannual": 2, "annual": 1}[payment_freq]
    
    r = rate / payments_per_year
    n = months * payments_per_year / 12
    
    if r > 0:
        installment = notional * (r * (1 + r)**n) / ((1 + r)**n - 1)
    else:
        installment = notional / n
    
    schedule: list[ScheduleRow] = []
    current_date = start_date
    remaining = notional
    
    while remaining > 0 and len(schedule) < n:
        interest = remaining * r
        principal = installment - interest
        
        if remaining - principal < 0:
            principal = remaining
            installment = principal + interest
        
        schedule.append({
            "period": len(schedule) + 1,
            "date": current_date.isoformat(),
            "payment": round(installment, 2),
            "principal": round(principal, 2),
            "interest": round(interest, 2),
            "outstanding": round(remaining - principal, 2)
        })
        
        remaining -= principal
        current_date = _increment_date(current_date, payment_freq)
    
    return schedule


def generate_bullet_schedule(notional: float, start_date: date, months: int, rate: float) -> list[ScheduleRow]:
    """Generate bullet loan schedule (interest only, principal at maturity)."""
    payments_per_year = 12
    monthly_rate = rate / payments_per_year
    
    schedule: list[ScheduleRow] = []
    
    for period in range(1, months + 1):
        current_date = start_date + timedelta(days=period * 30)
        interest = notional * monthly_rate
        
        schedule.append({
            "period": period,
            "date": current_date.isoformat(),
            "payment": round(interest, 2),
            "principal": 0,
            "interest": round(interest, 2),
            "outstanding": notional
        })
    
    final_date = start_date + timedelta(days=months * 30)
    schedule.append({
        "period": months + 1,
        "date": final_date.isoformat(),
        "payment": round(notional, 2),
        "principal": round(notional, 2),
        "interest": 0,
        "outstanding": 0
    })
    
    return schedule


def _increment_date(d: date, freq: str) -> date:
    """Increment date by payment frequency."""
    if freq == "monthly":
        return d + timedelta(days=30)
    elif freq == "quarterly":
        return d + timedelta(days=91)
    elif freq == "semiannual":
        return d + timedelta(days=182)
    elif freq == "annual":
        return d + timedelta(days=365)
    return d


def generate_nmd_behavioral_profile(name: str, lambda_decay: float, core_ratio: float, months: int = MONTHS_PROJECTION) -> NMDModel:
    """
    Generate NMD behavioral profile using exponential decay model.
    
    profile[t] = core_ratio + (1 - core_ratio) * exp(-lambda * t)
    
    Parameters:
    - lambda_decay: decay rate (higher = more volatile)
    - core_ratio: stable core portion (0-1)
    - months: projection horizon
    """
    profile: list[NMDProfileRow] = []
    
    for t in range(1, months + 1):
        decay_factor = math.exp(-lambda_decay * t)
        value = core_ratio + (1 - core_ratio) * decay_factor
        profile.append({
            "month": t,
            "remaining_ratio": round(value, 6),
            "core_component": round(core_ratio, 6),
            "volatile_component": round((1 - core_ratio) * decay_factor, 6)
        })
    
    return {
        "profile_name": name,
        "lambda_decay": lambda_decay,
        "core_ratio": core_ratio,
        "projection_months": months,
        "wal_months": round(sum(p["remaining_ratio"] for p in profile) / len(profile), 2),
        "profile": profile
    }


def generate_nmd_models() -> list[NMDModel]:
    """Generate various NMD behavioral models."""
    models: list[tuple[str, float, float]] = [
        ("Retail Demand Deposits Core", 0.05, 0.75),
        ("Retail Demand Deposits Volatile", 0.15, 0.40),
        ("Retail Savings Account", 0.08, 0.60),
        ("Corporate Operating Account", 0.12, 0.55),
        ("Corporate Cash Pool", 0.06, 0.70),
        ("SME Deposits", 0.10, 0.50),
        ("Payroll Account", 0.04, 0.80),
    ]
    
    return [generate_nmd_behavioral_profile(name, lambda_val, core) for name, lambda_val, core in models]


def generate_revolving_profile(commitment: float, utilization_rate: float, tenor_months: int, months: int = MONTHS_PROJECTION) -> RevolvingProfile:
    """Generate revolving facility utilization profile."""
    profile: list[RevolvingProfileRow] = []
    
    for t in range(1, months + 1):
        base_util = utilization_rate
        seasonal = 0.1 * math.sin(2 * math.pi * t / 12)
        noise = random.gauss(0, 0.05)
        
        util = max(0.05, min(0.95, base_util + seasonal + noise))
        
        profile.append({
            "month": t,
            "outstanding": round(commitment * util, 2),
            "commitment": commitment,
            "utilization_ratio": round(util, 4),
            "available": round(commitment * (1 - util), 2)
        })
    
    return {
        "profile_name": "Revolving Facility",
        "commitment": commitment,
        "avg_utilization": round(sum(p["utilization_ratio"] for p in profile) / len(profile), 4),
        "profile": profile
    }


def generate_credit_line_profile(commitment: float, avg_drawdown: float, months: int = MONTHS_PROJECTION) -> CreditLineProfile:
    """Generate credit line drawdown profile."""
    profile: list[CreditLineProfileRow] = []
    
    for t in range(1, months + 1):
        trend = min(t / 60, 1.0) * 0.3
        seasonal = 0.05 * math.cos(2 * math.pi * t / 12)
        noise = random.gauss(0, 0.03)
        
        drawdown = max(0.1, min(0.9, avg_drawdown + trend + seasonal + noise))
        
        profile.append({
            "month": t,
            "outstanding": round(commitment * drawdown, 2),
            "commitment": commitment,
            "drawdown_ratio": round(drawdown, 4)
        })
    
    return {
        "profile_name": "Credit Line",
        "commitment": commitment,
        "avg_drawdown": round(sum(p["drawdown_ratio"] for p in profile) / len(profile), 4),
        "profile": profile
    }


def generate_decaying_pool_profile(initial_amount: float, decay_rate: float, half_life_months: int, months: int = MONTHS_PROJECTION) -> DecayingPoolProfile:
    """Generate decaying pool profile (e.g., for securitization)."""
    profile: list[DecayingPoolProfileRow] = []
    decay_constant = math.log(2) / half_life_months
    
    for t in range(1, months + 1):
        remaining = initial_amount * math.exp(-decay_constant * t)
        profile.append({
            "month": t,
            "outstanding": round(remaining, 2),
            "remaining_ratio": round(remaining / initial_amount, 6),
            "amortized": round(initial_amount - remaining, 2)
        })
    
    return {
        "profile_name": "Decaying Pool",
        "initial_amount": initial_amount,
        "half_life_months": half_life_months,
        "profile": profile
    }


def write_json(data: list[NMDModel] | NMDModel | RevolvingProfile | CreditLineProfile | DecayingPoolProfile, filename: str) -> str:
    filepath = path.join(OUTPUT_DIR, filename)
    with open(filepath, 'w') as f:
        json.dump(data, f, indent=2, default=str)
    print(f"Generated: {filepath}")
    return filepath


def write_csv_schedule(schedule: list[ScheduleRow], filename: str) -> str:
    filepath = path.join(OUTPUT_DIR, filename)
    if not schedule:
        return ""
    
    keys = schedule[0].keys()
    with open(filepath, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=keys)
        writer.writeheader()
        writer.writerows(schedule)
    print(f"Generated: {filepath}")
    return filepath


def main() -> None:
    print("=" * 60)
    print("Data Schedules Generator")
    print("=" * 60)
    
    print("\n[1/5] Generating NMD behavioral models...")
    nmd_models = generate_nmd_models()
    write_json(nmd_models, "nmd_behavioral_profiles.json")
    
    print("\n[2/5] Generating loan amortization schedules...")
    start = date.today() - timedelta(days=365)
    
    linear = generate_linear_amortization(1000000, start, 60, "monthly")
    write_csv_schedule(linear, "schedule_linear_60m.csv")
    
    annuity = generate_constant_installment_amortization(500000, start, 120, 0.045, "monthly")
    write_csv_schedule(annuity, "schedule_annuity_120m.csv")
    
    bullet = generate_bullet_schedule(2000000, start, 24, 0.05)
    write_csv_schedule(bullet, "schedule_bullet_24m.csv")
    
    print("\n[3/5] Generating revolving facility profiles...")
    revol = generate_revolving_profile(1000000, 0.65, 36)
    write_json(revol, "revolving_profile.json")
    
    print("\n[4/5] Generating credit line profiles...")
    cl = generate_credit_line_profile(500000, 0.50)
    write_json(cl, "credit_line_profile.json")
    
    print("\n[5/5] Generating decaying pool profiles...")
    pool = generate_decaying_pool_profile(10000000, 0.5, 24)
    write_json(pool, "decaying_pool_profile.json")
    
    print("\n" + "=" * 60)
    print("Schedules generation complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()
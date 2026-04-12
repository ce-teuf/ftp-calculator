#!/usr/bin/env python3
"""
Data Entities Generator
Generates CSV files for organizational structure:
- Branches (bank branches in US, FR, ES, GER)
- Business Units (per branch)
- Departments (per business unit)
- Sales (per business unit)
- Treasury (one per branch)
"""

from __future__ import annotations

import csv
import uuid
import random
from datetime import date, timedelta
from os import path
from typing import TypedDict

OUTPUT_DIR = path.dirname(path.abspath(__file__))

random.seed(42)


class BranchConfig(TypedDict):
    country: str
    currency: str
    city: str
    business_units: list[str]
    departments_per_bu: list[str]


class BranchData(TypedDict):
    branch_id: str
    branch_code: str
    branch_name: str
    country: str
    currency: str
    city: str
    address: str
    phone: str
    status: str
    created_date: str


class BusinessUnitData(TypedDict):
    bu_id: str
    bu_name: str
    branch_id: str
    branch_code: str
    currency: str
    status: str
    created_date: str


class DepartmentData(TypedDict):
    dept_id: str
    dept_name: str
    bu_id: str
    bu_name: str
    branch_id: str
    branch_code: str
    status: str


class TreasuryData(TypedDict):
    treasury_id: str
    branch_id: str
    branch_code: str
    treasury_name: str
    currency: str
    status: str


class SalesData(TypedDict):
    seller_id: str
    seller_code: str
    first_name: str
    last_name: str
    email: str
    bu_id: str
    bu_name: str
    branch_id: str
    branch_code: str
    hire_date: str
    status: str
    target_volume: int
    seniority: str


BRANCHES_CONFIG: dict[str, BranchConfig] = {
    "US": {
        "country": "United States",
        "currency": "USD",
        "city": "New York",
        "business_units": ["Retail Banking", "Commercial Banking", "Corporate Banking", "Wealth Management"],
        "departments_per_bu": ["Sales", "Operations", "Risk", "Compliance"]
    },
    "FR": {
        "country": "France",
        "currency": "EUR",
        "city": "Paris",
        "business_units": ["Retail Banking", "Commercial Banking", "Corporate Banking", "PME-PMI", "Private Banking"],
        "departments_per_bu": ["Commercial", "Opérations", "Risques", "Conformité"]
    },
    "ES": {
        "country": "Spain",
        "currency": "EUR",
        "city": "Madrid",
        "business_units": ["Retail Banking", "Commercial Banking", "Corporate Banking", "SME"],
        "departments_per_bu": ["Ventas", "Operaciones", "Riesgos", "Cumplimiento"]
    },
    "DE": {
        "country": "Germany",
        "currency": "EUR",
        "city": "Frankfurt",
        "business_units": ["Retail Banking", "Commercial Banking", "Corporate Banking", "Private Banking"],
        "departments_per_bu": ["Vertrieb", "Operations", "Risiko", "Compliance"]
    }
}

SALES_FIRST_NAMES: dict[str, list[str]] = {
    "US": ["John", "Michael", "David", "James", "Robert", "William", "Mary", "Patricia", "Jennifer", "Linda"],
    "FR": ["Jean", "Pierre", "Michel", "François", "Nicolas", "Marie", "Sophie", "Catherine", "Julie", "Emma"],
    "ES": ["José", "Antonio", "Juan", "Carlos", "Miguel", "María", "Carmen", "Ana", "Laura", "Sofia"],
    "DE": ["Hans", "Peter", "Klaus", "Michael", "Thomas", "Anna", "Maria", "Christine", "Julia", "Stefanie"]
}

SALES_LAST_NAMES: list[str] = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Martin", "Wilson"]


def generate_branches() -> list[BranchData]:
    """Generate branch entities."""
    branches: list[BranchData] = []
    for code, config in BRANCHES_CONFIG.items():
        branch: BranchData = {
            "branch_id": f"BR-{code}-001",
            "branch_code": code,
            "branch_name": f"{config['city']} Branch",
            "country": config["country"],
            "currency": config["currency"],
            "city": config["city"],
            "address": f"{random.randint(1, 999)} {random.choice(['Main', 'Park', 'Liberty', 'Grand'])} Street",
            "phone": f"+1-{random.randint(200,999)}-{random.randint(100,999)}-{random.randint(1000,9999)}",
            "status": "active",
            "created_date": (date.today() - timedelta(days=random.randint(365, 3650))).isoformat()
        }
        branches.append(branch)
    return branches


def generate_business_units(branches: list[BranchData]) -> list[BusinessUnitData]:
    """Generate business units per branch."""
    bus: list[BusinessUnitData] = []
    for branch in branches:
        config = BRANCHES_CONFIG[branch["branch_code"]]
        for bu_name in config["business_units"]:
            bu: BusinessUnitData = {
                "bu_id": f"BU-{branch['branch_code']}-{uuid.uuid4().hex[:6].upper()}",
                "bu_name": bu_name,
                "branch_id": branch["branch_id"],
                "branch_code": branch["branch_code"],
                "currency": branch["currency"],
                "status": "active",
                "created_date": (date.today() - timedelta(days=random.randint(180, 2000))).isoformat()
            }
            bus.append(bu)
    return bus


def generate_departments(business_units: list[BusinessUnitData]) -> list[DepartmentData]:
    """Generate departments per business unit."""
    departments: list[DepartmentData] = []
    for bu in business_units:
        config = BRANCHES_CONFIG[bu["branch_code"]]
        for dept_name in config["departments_per_bu"]:
            dept: DepartmentData = {
                "dept_id": f"DEPT-{bu['branch_code']}-{uuid.uuid4().hex[:6].upper()}",
                "dept_name": dept_name,
                "bu_id": bu["bu_id"],
                "bu_name": bu["bu_name"],
                "branch_id": bu["branch_id"],
                "branch_code": bu["branch_code"],
                "status": "active"
            }
            departments.append(dept)
    return departments


def generate_treasury(branches: list[BranchData]) -> list[TreasuryData]:
    """Generate treasury per branch."""
    treasuries: list[TreasuryData] = []
    for branch in branches:
        treasury: TreasuryData = {
            "treasury_id": f"TR-{branch['branch_code']}-001",
            "branch_id": branch["branch_id"],
            "branch_code": branch["branch_code"],
            "treasury_name": f"{branch['city']} Treasury",
            "currency": branch["currency"],
            "status": "active"
        }
        treasuries.append(treasury)
    return treasuries


def generate_sales(business_units: list[BusinessUnitData], num_sales_per_bu: int = 15) -> list[SalesData]:
    """Generate sales per business unit."""
    sales: list[SalesData] = []
    for bu in business_units:
        country_code = bu["branch_code"]
        first_names = SALES_FIRST_NAMES.get(country_code, SALES_FIRST_NAMES["US"])
        
        for i in range(num_sales_per_bu):
            first_name = random.choice(first_names)
            last_name = random.choice(SALES_LAST_NAMES)
            
            seller: SalesData = {
                "seller_id": f"SLR-{bu['branch_code']}-{uuid.uuid4().hex[:8].upper()}",
                "seller_code": f"{bu['branch_code']}{i+1:03d}",
                "first_name": first_name,
                "last_name": last_name,
                "email": f"{first_name.lower()}.{last_name.lower()}@{bu['branch_code'].lower()}bank.com",
                "bu_id": bu["bu_id"],
                "bu_name": bu["bu_name"],
                "branch_id": bu["branch_id"],
                "branch_code": bu["branch_code"],
                "hire_date": (date.today() - timedelta(days=random.randint(90, 3650))).isoformat(),
                "status": "active",
                "target_volume": random.randint(500000, 5000000),
                "seniority": random.choice(["Junior", "Senior", "Lead", "Manager"])
            }
            sales.append(seller)
    return sales


def write_csv(data: list[BranchData] | list[BusinessUnitData] | list[DepartmentData] | list[TreasuryData] | list[SalesData], filename: str) -> None:
    """Write data to CSV file."""
    if not data:
        return
    keys = data[0].keys()
    filepath = path.join(OUTPUT_DIR, filename)
    with open(filepath, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=keys)
        writer.writeheader()
        writer.writerows(data)
    print(f"Generated: {filepath} ({len(data)} records)")


def main() -> None:
    print("=" * 60)
    print("Data Entities Generator")
    print("=" * 60)
    
    branches = generate_branches()
    write_csv(branches, "branches.csv")
    
    business_units = generate_business_units(branches)
    write_csv(business_units, "business_units.csv")
    
    departments = generate_departments(business_units)
    write_csv(departments, "departments.csv")
    
    treasuries = generate_treasury(branches)
    write_csv(treasuries, "treasuries.csv")
    
    sales = generate_sales(business_units, num_sales_per_bu=15)
    write_csv(sales, "sales.csv")
    
    print("\n" + "=" * 60)
    print("Entity relationships:")
    print(f"  - {len(branches)} branches")
    print(f"  - {len(business_units)} business units")
    print(f"  - {len(departments)} departments")
    print(f"  - {len(treasuries)} treasuries")
    print(f"  - {len(sales)} sales")
    print("=" * 60)


if __name__ == "__main__":
    main()
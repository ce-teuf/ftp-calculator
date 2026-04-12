#!/usr/bin/env python3
"""
Orchestrator for Amortization Matrix and Outstanding Balance generators.
Generates a ZIP archive containing both CSV files, each named with a UUIDv7.

Usage:
    python run_generators_zip.py --config config.yaml [--zip-output archive.zip]
"""

import argparse
import csv
import json
import math
import random
import sys
import os
import zipfile
import time
from datetime import datetime
from typing import List, Tuple, Dict, Any

try:
    import yaml
except ImportError:
    print("PyYAML is required. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(1)

# ----------------------------------------------------------------------
# UUIDv7 generator (based on timestamp + random bytes)
# ----------------------------------------------------------------------
def generate_uuid7() -> str:
    """
    Generate a UUID version 7 (time-ordered) as a string.
    Format: 8hex-4hex-4hex-4hex-12hex
    """
    # Get current Unix timestamp in milliseconds
    timestamp_ms = int(time.time() * 1000)
    # Timestamp fits in 48 bits (2^48-1 = 281474976710655)
    # UUID v7: first 48 bits = timestamp, next 4 bits = version (7), next 12 bits = random (or counter)
    # We'll use random for the remaining bits
    rand_a = random.getrandbits(12)  # 12 bits for the "ver" field's lower part
    rand_b = random.getrandbits(62)  # 62 bits for the rest
    
    # Build the 128-bit integer
    uuid_int = (timestamp_ms << 80) | (0x7 << 76) | (rand_a << 64) | rand_b
    # Format as hex with dashes
    hex_str = f"{uuid_int:032x}"
    return f"{hex_str[:8]}-{hex_str[8:12]}-{hex_str[12:16]}-{hex_str[16:20]}-{hex_str[20:]}"

# ----------------------------------------------------------------------
# Functions from amortization_matrix.py and outstanding_generator.py
# (copied here for standalone execution; you can also import if modules are present)
# ----------------------------------------------------------------------
def generate_monthly_dates(start_str: str, end_str: str) -> List[Tuple[datetime, str]]:
    start = datetime.strptime(start_str, "%m-%Y")
    end = datetime.strptime(end_str, "%m-%Y")
    dates = []
    current = start
    while current <= end:
        dates.append((current, current.strftime("%m-%Y")))
        if current.month == 12:
            current = current.replace(year=current.year + 1, month=1)
        else:
            current = current.replace(month=current.month + 1)
    return dates

def compute_amortization_profile(maturity_months: int, curvature: float) -> List[float]:
    if maturity_months < 1:
        return []
    if maturity_months == 1:
        return [0.0]
    denominator = maturity_months - 1
    profile = []
    for t in range(1, maturity_months + 1):
        ratio = (1.0 - (t - 1) / denominator) ** curvature
        profile.append(round(ratio, 6))
    return profile

def write_matrix_csv(output_file: str, start_date_str: str, end_date_str: str,
                     periods_config: List[Dict[str, Any]]) -> None:
    dates = generate_monthly_dates(start_date_str, end_date_str)
    if not dates:
        print("No dates generated. Check start and end.")
        return
    sorted_periods = []
    for p in periods_config:
        sp = p.get('start_period')
        if sp is None:
            sorted_periods.append((None, p))
        else:
            sp_date = datetime.strptime(sp, "%m-%Y")
            sorted_periods.append((sp_date, p))
    sorted_periods.sort(key=lambda x: (x[0] is not None, x[0]))
    first_sp, _ = sorted_periods[0]
    if first_sp is None:
        print("Warning: First period has no 'start_period' - applying to all earlier dates.")
    else:
        if first_sp > dates[0][0]:
            raise ValueError(f"First period starts at {first_sp.strftime('%m-%Y')} but global start is {start_date_str}.")
    for i, (sp_date, _) in enumerate(sorted_periods[1:], start=1):
        if sp_date is None:
            raise ValueError(f"Period {i+1} must have 'start_period'.")
    max_maturity = max(p['maturity'] for _, p in sorted_periods)
    header = ["date"] + [str(m) for m in range(1, max_maturity + 1)]
    def get_period_for_date(date: datetime) -> Dict[str, Any]:
        for sp_date, period in reversed(sorted_periods):
            if sp_date is None or sp_date <= date:
                return period
        return sorted_periods[0][1]
    with open(output_file, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(header)
        for dt, date_str in dates:
            period = get_period_for_date(dt)
            maturity = period['maturity']
            base_curvature = period['curvature']
            curvature_std = period.get('curvature_std', 0.0)
            if curvature_std > 0:
                curvature = random.gauss(base_curvature, curvature_std)
                curvature = max(0.01, curvature)
                profile = compute_amortization_profile(maturity, curvature)
            else:
                profile = compute_amortization_profile(maturity, base_curvature)
            full_profile = profile + [0.0] * (max_maturity - maturity)
            row = [date_str] + full_profile
            writer.writerow(row)
    print(f"Generated: {output_file}")

def compute_trend_values(n_months: int, trend_config: Dict[str, Any]) -> List[float]:
    trend_type = trend_config.get("type", "linear")
    start_val = trend_config.get("start_value", 0.0)
    end_val = trend_config.get("end_value", 1.0)
    curvature = trend_config.get("curvature", 1.0)
    midpoint = trend_config.get("logistic_midpoint", 0.5)
    steepness = trend_config.get("logistic_steepness", 10.0)
    if n_months == 1:
        return [start_val]
    values = []
    for t in range(n_months):
        progress = t / (n_months - 1) if n_months > 1 else 0.0
        if trend_type == "linear":
            weight = progress
        elif trend_type == "exponential":
            if start_val <= 0 or end_val <= 0:
                raise ValueError("Exponential trend requires positive start and end values")
            weight = start_val * (end_val / start_val) ** progress
            values.append(weight)
            continue
        elif trend_type == "convex":
            weight = progress ** curvature
        elif trend_type == "concave":
            weight = progress ** curvature
        elif trend_type == "logistic":
            logistic = 1.0 / (1.0 + math.exp(-steepness * (progress - midpoint)))
            weight = logistic
        else:
            raise ValueError(f"Unknown trend type: {trend_type}")
        if trend_type != "exponential":
            value = start_val + (end_val - start_val) * weight
            values.append(value)
    return values

def add_noise(values: List[float], noise_config: Dict[str, Any]) -> List[float]:
    if not noise_config:
        return values
    noise_type = noise_config.get("type", "absolute")
    if noise_type == "absolute":
        std = noise_config.get("std", 0.0)
        if std <= 0:
            return values
        noise = [random.gauss(0, std) for _ in values]
    elif noise_type == "relative":
        rel = noise_config.get("rel", 0.0)
        if rel <= 0:
            return values
        noise = [random.gauss(0, rel * abs(v)) for v in values]
    else:
        raise ValueError("noise.type must be 'absolute' or 'relative'")
    noisy = [max(0.0, v + n) for v, n in zip(values, noise)]
    return noisy

def write_outstanding_csv(output_file: str, dates: List[Tuple[datetime, str]], values: List[float]) -> None:
    with open(output_file, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(["date", "outstanding"])
        for (_, date_str), val in zip(dates, values):
            writer.writerow([date_str, round(val, 2)])
    print(f"Generated: {output_file}")

# ----------------------------------------------------------------------
# Configuration loading
# ----------------------------------------------------------------------
def load_combined_config(config_file: str) -> dict:
    with open(config_file, 'r', encoding='utf-8') as f:
        full_config = yaml.safe_load(f)
    global_cfg = full_config.get('global', {})
    global_start = global_cfg.get('start')
    global_end = global_cfg.get('end')
    
    if 'amortization' not in full_config:
        raise ValueError("YAML must contain 'amortization' section")
    amort_cfg = full_config['amortization'].copy()
    if global_start and global_end:
        amort_cfg['start'] = global_start
        amort_cfg['end'] = global_end
    else:
        if 'start' not in amort_cfg or 'end' not in amort_cfg:
            raise ValueError("Amortization needs start/end or global")
    
    if 'periods' not in amort_cfg:
        if 'maturity' not in amort_cfg:
            raise ValueError("Amortization must have 'periods' or 'maturity'")
        amort_cfg['periods'] = [{
            'maturity': amort_cfg['maturity'],
            'curvature': amort_cfg.get('curvature', 1.0),
            'curvature_std': amort_cfg.get('curvature_std', 0.0)
        }]
        for k in ['maturity', 'curvature', 'curvature_std']:
            amort_cfg.pop(k, None)
    else:
        for idx, p in enumerate(amort_cfg['periods']):
            if 'maturity' not in p:
                raise ValueError(f"Period {idx} missing 'maturity'")
            if 'curvature' not in p:
                raise ValueError(f"Period {idx} missing 'curvature'")
            p.setdefault('curvature_std', 0.0)
            if idx > 0 and 'start_period' not in p:
                raise ValueError(f"Period {idx} must have 'start_period'")
        def period_sort_key(p):
            sp = p.get('start_period')
            if sp is None:
                return (0, None)
            return (1, datetime.strptime(sp, "%m-%Y"))
        amort_cfg['periods'].sort(key=period_sort_key)
    
    if 'outstanding' not in full_config:
        raise ValueError("YAML must contain 'outstanding' section")
    out_cfg = full_config['outstanding'].copy()
    if global_start and global_end:
        out_cfg['start'] = global_start
        out_cfg['end'] = global_end
    else:
        if 'start' not in out_cfg or 'end' not in out_cfg:
            raise ValueError("Outstanding needs start/end or global")
    
    if 'trend' not in out_cfg:
        raise ValueError("Outstanding missing 'trend'")
    trend = out_cfg['trend']
    if 'type' not in trend:
        raise ValueError("trend must have 'type'")
    if 'start_value' not in trend or 'end_value' not in trend:
        raise ValueError("trend requires start_value and end_value")
    if trend['type'] in ['convex', 'concave'] and 'curvature' not in trend:
        raise ValueError(f"trend type {trend['type']} requires curvature")
    if trend['type'] == 'logistic':
        trend.setdefault('logistic_midpoint', 0.5)
        trend.setdefault('logistic_steepness', 10.0)
    out_cfg.setdefault('noise', {})
    noise = out_cfg['noise']
    if noise:
        if 'type' not in noise:
            noise['type'] = 'absolute'
        if noise['type'] == 'absolute' and 'std' not in noise:
            raise ValueError("absolute noise requires std")
        if noise['type'] == 'relative' and 'rel' not in noise:
            raise ValueError("relative noise requires rel")
    out_cfg.setdefault('output', 'outstanding.csv')
    
    return {'amortization': amort_cfg, 'outstanding': out_cfg}

# ----------------------------------------------------------------------
# Main orchestration with ZIP
# ----------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description="Generate amortization matrix and outstanding CSV, then pack into ZIP with UUIDv7 names.")
    parser.add_argument("--config", required=True, help="YAML configuration file")
    parser.add_argument("--zip-output", default=None, help="Output ZIP file name (default: <uuid>.zip)")
    args = parser.parse_args()
    
    configs = load_combined_config(args.config)
    amort_cfg = configs['amortization']
    out_cfg = configs['outstanding']
    
    # Generate a UUIDv7 for the file names
    uuid_str = generate_uuid7()
    # Create temporary CSV file names
    amort_csv = f"{uuid_str}_amortization.csv"
    out_csv = f"{uuid_str}_outstanding.csv"
    
    # Override outputs in configs
    amort_cfg['output'] = amort_csv
    out_cfg['output'] = out_csv
    
    # Generate amortization matrix
    print("Generating amortization matrix...")
    write_matrix_csv(amort_cfg['output'], amort_cfg['start'], amort_cfg['end'], amort_cfg['periods'])
    
    # Generate outstanding balance
    print("Generating outstanding balance...")
    dates = generate_monthly_dates(out_cfg['start'], out_cfg['end'])
    trend_vals = compute_trend_values(len(dates), out_cfg['trend'])
    final_vals = add_noise(trend_vals, out_cfg.get('noise', {}))
    write_outstanding_csv(out_cfg['output'], dates, final_vals)
    
    # Create ZIP archive
    zip_filename = args.zip_output if args.zip_output else f"{uuid_str}.zip"
    with zipfile.ZipFile(zip_filename, 'w', zipfile.ZIP_DEFLATED) as zipf:
        zipf.write(amort_csv, arcname=amort_csv)
        zipf.write(out_csv, arcname=out_csv)
    
    print(f"\nZIP archive created: {zip_filename}")
    print(f"Contents: {amort_csv}, {out_csv}")
    
    # Optionally delete the individual CSV files
    os.remove(amort_csv)
    os.remove(out_csv)
    print("Temporary CSV files removed.")

if __name__ == "__main__":
    random.seed()  # don't fix seed for UUID randomness
    main()
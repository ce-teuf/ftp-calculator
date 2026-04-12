#!/usr/bin/env python3
"""
Amortization Matrix Generator with Multi-Period YAML Configuration

Supports multiple amortization profiles over time, each starting at a given month.
"""

import argparse
import csv
import random
import sys
from datetime import datetime
from typing import List, Tuple, Dict, Any, Optional

try:
    import yaml
except ImportError:
    yaml = None
    print("Warning: PyYAML not installed. YAML config files will not work.", file=sys.stderr)


def generate_monthly_dates(start_str: str, end_str: str) -> List[Tuple[datetime, str]]:
    """Generate a list of monthly dates from start to end inclusive."""
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
    """Compute remaining ratio for each month from 1 to maturity."""
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
    """
    Generate CSV matrix using multiple periods.
    Each period defines its own maturity, curvature, curvature_std, and optional start_period.
    """
    dates = generate_monthly_dates(start_date_str, end_date_str)
    if not dates:
        print("No dates generated. Check start and end.")
        return

    # Prepare periods sorted by start_period (None first)
    sorted_periods = []
    for p in periods_config:
        # Convert start_period string to datetime if present
        sp = p.get('start_period')
        if sp is None:
            sorted_periods.append((None, p))
        else:
            sp_date = datetime.strptime(sp, "%m-%Y")
            sorted_periods.append((sp_date, p))
    # Sort: None first, then by date
    sorted_periods.sort(key=lambda x: (x[0] is not None, x[0]))
    
    # Validate: first period may have no start_period (warning), others must have
    first_sp, first_period = sorted_periods[0]
    if first_sp is None:
        print("Warning: First period has no 'start_period' - it will apply to all dates before the next period's start (or entire range if only one period).")
    else:
        # If first period has a start_period, then dates before that start would have no period -> we raise error
        if first_sp > dates[0][0]:
            raise ValueError(f"First period starts at {first_sp.strftime('%m-%Y')} but global start is {start_date_str}. No period defined for earlier dates.")
    
    # For each period after the first, ensure start_period exists
    for i, (sp_date, _) in enumerate(sorted_periods[1:], start=1):
        if sp_date is None:
            raise ValueError(f"Period {i+1} (0-index) must have 'start_period' because it's not the first period.")
    
    # Determine the maximum maturity among all periods to know the number of columns
    max_maturity = max(p['maturity'] for _, p in sorted_periods)
    header = ["date"] + [str(m) for m in range(1, max_maturity + 1)]
    
    # Function to find period for a given date
    def get_period_for_date(date: datetime) -> Dict[str, Any]:
        # Iterate from last to first to find the latest period with start_period <= date
        for sp_date, period in reversed(sorted_periods):
            if sp_date is None or sp_date <= date:
                return period
        # Should never reach here because first period has no start or first start <= earliest date?
        return sorted_periods[0][1]  # fallback
    
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
            
            # Pad with zeros if maturity < max_maturity
            full_profile = profile + [0.0] * (max_maturity - maturity)
            row = [date_str] + full_profile
            writer.writerow(row)
    
    print(f"Generated matrix: {output_file}")
    print(f"Rows: {len(dates)} (from {start_date_str} to {end_date_str})")
    print(f"Columns: {max_maturity} months (max maturity among periods)")
    for i, (sp_date, period) in enumerate(sorted_periods):
        sp_str = sp_date.strftime("%m-%Y") if sp_date else "default"
        print(f"  Period {i+1}: start={sp_str}, maturity={period['maturity']}, curvature={period['curvature']}, std={period.get('curvature_std',0)}")


def load_config_from_yaml(config_file: str) -> Dict[str, Any]:
    """Load configuration from YAML file, supporting multiple periods."""
    if yaml is None:
        raise ImportError("PyYAML required. Install: pip install pyyaml")
    with open(config_file, 'r', encoding='utf-8') as f:
        config = yaml.safe_load(f)
    
    # Required global keys
    if 'start' not in config or 'end' not in config:
        raise ValueError("YAML must contain 'start' and 'end' keys")
    
    # If 'periods' key exists, use multi-period mode
    if 'periods' in config:
        periods = config['periods']
        if not isinstance(periods, list) or len(periods) == 0:
            raise ValueError("'periods' must be a non-empty list")
        # Validate each period
        for idx, p in enumerate(periods):
            if 'maturity' not in p:
                raise ValueError(f"Period {idx} missing 'maturity'")
            if 'curvature' not in p:
                raise ValueError(f"Period {idx} missing 'curvature'")
            # curvature_std is optional, default 0
            p.setdefault('curvature_std', 0.0)
            # start_period is optional only for first period
            if idx > 0 and 'start_period' not in p:
                raise ValueError(f"Period {idx} (not first) must have 'start_period'")
        # Sort periods by start_period (None first)
        def period_sort_key(p):
            sp = p.get('start_period')
            if sp is None:
                return (0, None)  # None comes first
            return (1, datetime.strptime(sp, "%m-%Y"))
        periods.sort(key=period_sort_key)
        config['periods'] = periods
    else:
        # Single period mode: create one period with no start_period using global maturity, curvature, curvature_std
        if 'maturity' not in config:
            raise ValueError("YAML must contain 'maturity' (or 'periods')")
        config['periods'] = [{
            'maturity': config['maturity'],
            'curvature': config.get('curvature', 1.0),
            'curvature_std': config.get('curvature_std', 0.0),
            # no start_period
        }]
    
    # Output file
    config.setdefault('output', 'amortization_matrix.csv')
    return config


def main():
    parser = argparse.ArgumentParser(description="Generate amortization matrix with optional multi-period YAML config.")
    parser.add_argument("--config", help="YAML configuration file (supports multiple periods)")
    # For backward compatibility, still allow command line, but recommend YAML
    parser.add_argument("--start", help="Start date (MM-YYYY)")
    parser.add_argument("--end", help="End date (MM-YYYY)")
    parser.add_argument("--maturity", type=int, help="Maturity in months")
    parser.add_argument("--curvature", type=float, default=1.0, help="Base curvature")
    parser.add_argument("--curvature-std", type=float, default=0.0, help="Randomness std dev")
    parser.add_argument("--output", default="amortization_matrix.csv", help="Output CSV")
    
    args = parser.parse_args()
    
    if args.config:
        config = load_config_from_yaml(args.config)
        start = config['start']
        end = config['end']
        output = config['output']
        periods = config['periods']
        write_matrix_csv(output, start, end, periods)
    else:
        # Command line mode (single period)
        if not args.start or not args.end or not args.maturity:
            parser.error("When --config is not used, --start, --end, and --maturity are required")
        # Build a single period config
        periods = [{
            'maturity': args.maturity,
            'curvature': args.curvature,
            'curvature_std': args.curvature_std,
        }]
        write_matrix_csv(args.output, args.start, args.end, periods)


if __name__ == "__main__":
    random.seed(42)  # for reproducibility
    main()
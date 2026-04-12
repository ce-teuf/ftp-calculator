#!/usr/bin/env python3
"""
Outstanding Balance Generator

Generates a CSV with monthly dates and outstanding balance values that follow
a deterministic trend (linear, exponential, convex, concave, logistic) plus
additive Gaussian noise.

Usage:
    python outstanding_generator.py --config config.yaml

YAML Configuration Example:
    start: "01-2020"
    end: "12-2024"
    output: "outstanding.csv"
    trend:
      type: "linear"          # linear, exponential, convex, concave, logistic
      start_value: 1000.0
      end_value: 5000.0
      curvature: 1.5         # for convex/concave (exponent for power law)
      logistic_midpoint: 0.5  # for logistic (fraction of time range)
      logistic_steepness: 10  # for logistic
    noise:
      std: 50.0               # absolute standard deviation of additive noise
      # alternatively, relative noise: rel: 0.05 (5% of current value)
    random_seed: 42
"""

import argparse
import csv
import math
import random
import sys
from datetime import datetime
from typing import List, Tuple, Dict, Any

try:
    import yaml
except ImportError:
    yaml = None
    print("Warning: PyYAML not installed. Install with: pip install pyyaml", file=sys.stderr)


def generate_monthly_dates(start_str: str, end_str: str) -> List[Tuple[datetime, str]]:
    """Generate monthly dates from start to end inclusive."""
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


def compute_trend_values(n_months: int, trend_config: Dict[str, Any]) -> List[float]:
    """
    Compute deterministic trend values for each month (0-indexed).
    trend_config keys:
        type: "linear", "exponential", "convex", "concave", "logistic"
        start_value: float
        end_value: float
        curvature: float (for convex/concave, exponent >0; convex: >1, concave: 0<curvature<1)
        logistic_midpoint: float (0-1, fraction of time where growth is fastest)
        logistic_steepness: float (higher = steeper transition)
    """
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
        # progress from 0 to 1 over months
        progress = t / (n_months - 1) if n_months > 1 else 0.0

        if trend_type == "linear":
            weight = progress
        elif trend_type == "exponential":
            # exponential from start to end: value = start * (end/start)^progress
            # avoid division by zero
            if start_val <= 0 or end_val <= 0:
                raise ValueError("Exponential trend requires positive start and end values")
            weight = start_val * (end_val / start_val) ** progress
            values.append(weight)
            continue
        elif trend_type == "convex":
            # power law with exponent >1: accelerates
            weight = progress ** curvature
        elif trend_type == "concave":
            # power law with exponent <1: decelerates
            weight = progress ** curvature
        elif trend_type == "logistic":
            # logistic S-curve: L / (1 + exp(-k*(x - x0)))
            # map progress to logistic: value = start + (end-start) * logistic(progress)
            # logistic function: 1/(1+exp(-steepness*(progress - midpoint)))
            logistic = 1.0 / (1.0 + math.exp(-steepness * (progress - midpoint)))
            weight = logistic
        else:
            raise ValueError(f"Unknown trend type: {trend_type}")

        if trend_type != "exponential":
            value = start_val + (end_val - start_val) * weight
            values.append(value)

    return values


def add_noise(values: List[float], noise_config: Dict[str, Any]) -> List[float]:
    """Add Gaussian noise (absolute or relative) to values."""
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

    # Ensure no negative outstanding (clip to zero)
    noisy = [max(0.0, v + n) for v, n in zip(values, noise)]
    return noisy


def write_csv(output_file: str, dates: List[Tuple[datetime, str]], values: List[float]) -> None:
    """Write date and value columns to CSV."""
    with open(output_file, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(["date", "outstanding"])
        for (_, date_str), val in zip(dates, values):
            writer.writerow([date_str, round(val, 2)])
    print(f"Generated: {output_file}")
    print(f"Rows: {len(dates)}")


def load_config_from_yaml(config_file: str) -> Dict[str, Any]:
    """Load and validate YAML configuration."""
    if yaml is None:
        raise ImportError("PyYAML is required. Install with: pip install pyyaml")
    with open(config_file, 'r', encoding='utf-8') as f:
        config = yaml.safe_load(f)

    required = ['start', 'end', 'trend']
    for key in required:
        if key not in config:
            raise ValueError(f"Missing required key: '{key}'")

    # Validate trend
    trend = config['trend']
    if 'type' not in trend:
        raise ValueError("trend must have 'type' (linear, exponential, convex, concave, logistic)")
    if 'start_value' not in trend or 'end_value' not in trend:
        raise ValueError("trend requires 'start_value' and 'end_value'")
    if trend['type'] in ['convex', 'concave'] and 'curvature' not in trend:
        raise ValueError(f"trend type '{trend['type']}' requires 'curvature'")
    if trend['type'] == 'logistic':
        trend.setdefault('logistic_midpoint', 0.5)
        trend.setdefault('logistic_steepness', 10.0)

    # Noise is optional
    config.setdefault('noise', {})
    noise = config['noise']
    if noise:
        if 'type' not in noise:
            noise['type'] = 'absolute'
        if noise['type'] == 'absolute' and 'std' not in noise:
            raise ValueError("absolute noise requires 'std'")
        if noise['type'] == 'relative' and 'rel' not in noise:
            raise ValueError("relative noise requires 'rel'")

    config.setdefault('output', 'outstanding.csv')
    if 'random_seed' in config:
        random.seed(config['random_seed'])

    return config


def main():
    parser = argparse.ArgumentParser(description="Generate outstanding balance CSV with trend and noise.")
    parser.add_argument("--config", required=True, help="YAML configuration file")
    args = parser.parse_args()

    config = load_config_from_yaml(args.config)
    start = config['start']
    end = config['end']
    output = config['output']
    trend_config = config['trend']
    noise_config = config.get('noise', {})

    dates = generate_monthly_dates(start, end)
    n_months = len(dates)
    if n_months == 0:
        print("No dates generated. Check start and end.")
        return

    # Generate deterministic trend
    trend_values = compute_trend_values(n_months, trend_config)

    # Add noise
    final_values = add_noise(trend_values, noise_config)

    # Write CSV
    write_csv(output, dates, final_values)

    # Print summary
    print(f"Trend type: {trend_config['type']}")
    print(f"Start value: {trend_config['start_value']} -> End value: {trend_config['end_value']}")
    if noise_config:
        noise_type = noise_config.get('type', 'absolute')
        if noise_type == 'absolute':
            print(f"Noise: absolute std = {noise_config.get('std', 0)}")
        else:
            print(f"Noise: relative std = {noise_config.get('rel', 0)}")
    else:
        print("No noise added")
    print(f"Outstanding range: [{min(final_values):.2f}, {max(final_values):.2f}]")


if __name__ == "__main__":
    main()
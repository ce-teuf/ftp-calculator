-- Migration 009: Link curves to rate series + cube projection config
-- ─────────────────────────────────────────────────────────────────────────────
-- 1. rate_curves.series_name  — which historical rate series underpins this curve
--    (e.g. "SOFR", "ESTR", "EURIBOR").  Nullable: manually-entered spreads have no
--    underlying series.
--
-- 2. curve_cubes.proj_config_json — per-series projection config, stored as JSON:
--    {
--      "SOFR": { "method": "pca_bootstrap", "n_scenarios": 100, "seed": 42 },
--      "ESTR": { "method": "hw2f",          "n_scenarios":  50, "seed": 137,
--                "params": {"a": 0.05, "sigma": 0.01, "rho": -0.70} }
--    }
--    Null means no projection (deterministic / single scenario).

ALTER TABLE rate_curves
    ADD COLUMN IF NOT EXISTS series_name TEXT;

ALTER TABLE curve_cubes
    ADD COLUMN IF NOT EXISTS proj_config_json TEXT;

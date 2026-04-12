-- Migration 004: Org entity tables + FK constraint fixes
-- ─────────────────────────────────────────────────────────────────────────────
-- Fixes:
--   • contracts.source_dataset_id FK → ON DELETE SET NULL (was blocking dataset delete)
--   • freeze_log.dataset_id FK → ON DELETE CASCADE
-- New tables:
--   • org_branches, org_business_units, org_departments, org_sellers, org_treasuries
--   • rate_series_data (historical time series rows, different from rate_curves spot curves)

-- ── Fix broken FK constraints ─────────────────────────────────────────────────

ALTER TABLE contracts
    DROP CONSTRAINT IF EXISTS contracts_source_dataset_id_fkey;
ALTER TABLE contracts
    ADD CONSTRAINT contracts_source_dataset_id_fkey
    FOREIGN KEY (source_dataset_id) REFERENCES datasets(id) ON DELETE SET NULL;

ALTER TABLE freeze_log
    DROP CONSTRAINT IF EXISTS freeze_log_dataset_id_fkey;
ALTER TABLE freeze_log
    ADD CONSTRAINT freeze_log_dataset_id_fkey
    FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON DELETE CASCADE;

-- ── Org entity tables ─────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS org_branches (
    id           TEXT PRIMARY KEY,
    branch_code  TEXT NOT NULL UNIQUE,
    branch_name  TEXT,
    country      TEXT,
    currency     TEXT NOT NULL DEFAULT 'EUR',
    city         TEXT,
    address      TEXT,
    phone        TEXT,
    status       TEXT NOT NULL DEFAULT 'active',
    created_date DATE
);

CREATE TABLE IF NOT EXISTS org_business_units (
    id           TEXT PRIMARY KEY,
    bu_name      TEXT NOT NULL,
    branch_id    TEXT REFERENCES org_branches(id) ON DELETE SET NULL,
    branch_code  TEXT,
    currency     TEXT NOT NULL DEFAULT 'EUR',
    status       TEXT NOT NULL DEFAULT 'active',
    created_date DATE
);

CREATE TABLE IF NOT EXISTS org_departments (
    id           TEXT PRIMARY KEY,
    dept_name    TEXT NOT NULL,
    bu_id        TEXT REFERENCES org_business_units(id) ON DELETE SET NULL,
    bu_name      TEXT,
    branch_id    TEXT REFERENCES org_branches(id) ON DELETE SET NULL,
    branch_code  TEXT,
    status       TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS org_sellers (
    id            TEXT PRIMARY KEY,
    seller_code   TEXT,
    first_name    TEXT,
    last_name     TEXT,
    email         TEXT,
    bu_id         TEXT REFERENCES org_business_units(id) ON DELETE SET NULL,
    bu_name       TEXT,
    branch_id     TEXT REFERENCES org_branches(id) ON DELETE SET NULL,
    branch_code   TEXT,
    hire_date     DATE,
    status        TEXT NOT NULL DEFAULT 'active',
    target_volume DOUBLE PRECISION,
    seniority     TEXT
);

CREATE TABLE IF NOT EXISTS org_treasuries (
    id            TEXT PRIMARY KEY,
    branch_id     TEXT REFERENCES org_branches(id) ON DELETE SET NULL,
    branch_code   TEXT,
    treasury_name TEXT,
    currency      TEXT NOT NULL DEFAULT 'EUR',
    status        TEXT NOT NULL DEFAULT 'active'
);

-- ── Rate series (historical daily/monthly rates) ───────────────────────────────
-- Distinct from rate_curves (spot curves at a snapshot date).
-- Each row = one observation (date + tenor + rate value).

CREATE TABLE IF NOT EXISTS rate_series_data (
    id          TEXT        PRIMARY KEY,
    series_name TEXT        NOT NULL,   -- e.g. "ESTR", "EURIBOR_3M"
    component   TEXT        NOT NULL,   -- base_rate | ibor | credit_spread
    currency    TEXT        NOT NULL DEFAULT 'EUR',
    obs_date    DATE        NOT NULL,
    tenor       TEXT,                   -- e.g. "1D", "3M", "1Y" (null for overnight)
    rate        DOUBLE PRECISION NOT NULL,
    source      TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_rate_series_data_unique
    ON rate_series_data(series_name, obs_date, COALESCE(tenor, ''));
CREATE INDEX IF NOT EXISTS idx_rate_series_data_series ON rate_series_data(series_name, obs_date);

-- ── Schedules table ───────────────────────────────────────────────────────────
-- Per-contract amortization schedules (period-by-period cash flows).

CREATE TABLE IF NOT EXISTS contract_schedules (
    id          TEXT        PRIMARY KEY,
    contract_id TEXT        NOT NULL REFERENCES contracts(id) ON DELETE CASCADE,
    period      INTEGER     NOT NULL,
    sched_date  DATE        NOT NULL,
    payment     DOUBLE PRECISION,
    principal   DOUBLE PRECISION,
    interest    DOUBLE PRECISION,
    outstanding DOUBLE PRECISION,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_schedules_contract ON contract_schedules(contract_id);

-- ── Indexes for org entities ──────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_org_sellers_branch     ON org_sellers(branch_code);
CREATE INDEX IF NOT EXISTS idx_org_sellers_bu         ON org_sellers(bu_id);
CREATE INDEX IF NOT EXISTS idx_org_bus_branch         ON org_business_units(branch_id);
CREATE INDEX IF NOT EXISTS idx_org_depts_bu           ON org_departments(bu_id);

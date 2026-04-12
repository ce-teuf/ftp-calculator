-- Migration: 001_initial_schema
-- Description: Initial database schema for FTP Simulator (PostgreSQL 18)

CREATE TABLE IF NOT EXISTS rate_curves (
    id          TEXT         PRIMARY KEY,
    name        TEXT         NOT NULL,
    component   TEXT         NOT NULL,
    currency    TEXT         NOT NULL DEFAULT 'EUR',
    version     INTEGER      NOT NULL DEFAULT 1,
    status      TEXT         NOT NULL DEFAULT 'draft',   -- draft | approved | archived
    valid_from  DATE,
    valid_to    DATE,
    tenors_json TEXT         NOT NULL,   -- JSON array of tenor labels, e.g. ["1M","3M","1Y","5Y"]
    values_json TEXT         NOT NULL,   -- JSON array of rate values (same length as tenors)
    source      TEXT,
    notes       TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    created_by  TEXT
);

CREATE TABLE IF NOT EXISTS rate_series (
    id          TEXT        PRIMARY KEY,
    name        TEXT        NOT NULL,
    component   TEXT        NOT NULL,
    frequency   TEXT        NOT NULL,   -- daily | monthly | quarterly
    dates_json  TEXT        NOT NULL,   -- JSON array of ISO date strings
    values_json TEXT        NOT NULL,   -- JSON array of rate values
    tenor       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS runoff_models (
    id              TEXT        PRIMARY KEY,
    name            TEXT        NOT NULL,
    product_type    TEXT        NOT NULL,
    category        TEXT,
    version         INTEGER     NOT NULL DEFAULT 1,
    status          TEXT        NOT NULL DEFAULT 'draft',
    method          TEXT        NOT NULL,   -- Stock | Flux | Behavioral | Replicating
    profile_json    TEXT        NOT NULL,   -- JSON array of profile values [0..1]
    parameters_json TEXT,                  -- JSON object (lambda, core_ratio, etc.)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS portfolios (
    id          TEXT        PRIMARY KEY,
    name        TEXT        NOT NULL,
    description TEXT,
    version     INTEGER     NOT NULL DEFAULT 1,
    status      TEXT        NOT NULL DEFAULT 'draft',
    as_of_date  DATE        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS portfolio_positions (
    id               TEXT    PRIMARY KEY,
    portfolio_id     TEXT    NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    position_ref     TEXT,
    product_type     TEXT    NOT NULL,
    branch           TEXT,
    seller           TEXT,
    currency         TEXT    NOT NULL DEFAULT 'EUR',
    outstanding      DOUBLE PRECISION NOT NULL,
    origination_date DATE,
    maturity_date    DATE,
    client_rate      DOUBLE PRECISION,
    runoff_model_id  TEXT    REFERENCES runoff_models(id),
    risk_weight      DOUBLE PRECISION DEFAULT 1.0,
    profiles_json    TEXT,   -- JSON array of profile values (overrides runoff_model if set)
    rates_json       TEXT,   -- JSON array of rate values per tenor
    metadata_json    TEXT
);

CREATE TABLE IF NOT EXISTS executions (
    id              TEXT        PRIMARY KEY,
    label           TEXT,
    method          TEXT        NOT NULL,
    portfolio_id    TEXT        NOT NULL REFERENCES portfolios(id),
    curve_ids_json  TEXT        NOT NULL DEFAULT '[]',
    runoff_ids_json TEXT,
    parameters_json TEXT        NOT NULL DEFAULT '{}',
    seeds_json      TEXT,
    result_json     TEXT,       -- serialised output matrices (ftp_rate, ftp_int, market_rate)
    status          TEXT        NOT NULL DEFAULT 'pending',   -- pending | completed | error
    error_message   TEXT,
    duration_ms     BIGINT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      TEXT,
    notes           TEXT
);

CREATE TABLE IF NOT EXISTS alco_approvals (
    id          TEXT        PRIMARY KEY,
    entity_type TEXT        NOT NULL,
    entity_id   TEXT        NOT NULL,
    action      TEXT        NOT NULL,
    by_user     TEXT        NOT NULL,
    at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    comment     TEXT
);

CREATE INDEX IF NOT EXISTS idx_rate_curves_status    ON rate_curves(status);
CREATE INDEX IF NOT EXISTS idx_rate_curves_component ON rate_curves(component);
CREATE INDEX IF NOT EXISTS idx_portfolios_status     ON portfolios(status);
CREATE INDEX IF NOT EXISTS idx_executions_status     ON executions(status);
CREATE INDEX IF NOT EXISTS idx_executions_portfolio  ON executions(portfolio_id);
CREATE INDEX IF NOT EXISTS idx_positions_portfolio   ON portfolio_positions(portfolio_id);

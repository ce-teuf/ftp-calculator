-- V3 schema: curve stacks, cubes, portfolios, linkers, studies

-- ── Curve Stacks ─────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS curve_stacks (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'draft',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS curve_stack_components (
    id       TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL REFERENCES curve_stacks(id) ON DELETE CASCADE,
    position INT  NOT NULL,
    label    TEXT NOT NULL,
    curve_id TEXT NOT NULL REFERENCES rate_curves(id) ON DELETE RESTRICT,
    weight   REAL NOT NULL DEFAULT 1.0
);

CREATE INDEX IF NOT EXISTS idx_stack_components_stack ON curve_stack_components(stack_id);

-- ── Curve Cubes ──────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS curve_cubes (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    stack_id        TEXT NOT NULL REFERENCES curve_stacks(id) ON DELETE RESTRICT,
    analysis_start  DATE NOT NULL,
    analysis_end    DATE NOT NULL,
    step_months     INT  NOT NULL DEFAULT 1,
    include_proj    BOOL NOT NULL DEFAULT false,
    proj_script     TEXT,
    mc_scenarios    INT  NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'draft',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Portfolios ────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS portfolios_v3 (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    description   TEXT,
    schedule_type TEXT NOT NULL DEFAULT 'stock_amort',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolio_rows (
    id               TEXT PRIMARY KEY,
    portfolio_id     TEXT NOT NULL REFERENCES portfolios_v3(id) ON DELETE CASCADE,
    label            TEXT,
    schedule_json    TEXT NOT NULL,
    outstanding_json TEXT NOT NULL,
    row_order        INT  NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_portfolio_rows_portfolio ON portfolio_rows(portfolio_id);

-- ── Linkers ───────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS linkers (
    id                   TEXT PRIMARY KEY,
    name                 TEXT NOT NULL,
    portfolio_id         TEXT NOT NULL REFERENCES portfolios_v3(id) ON DELETE RESTRICT,
    cube_id              TEXT NOT NULL REFERENCES curve_cubes(id) ON DELETE RESTRICT,
    start_date           DATE NOT NULL,
    fwd_schedule_json    TEXT,
    fwd_outstanding_json TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Studies ───────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS studies (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS study_linkers (
    study_id  TEXT NOT NULL REFERENCES studies(id) ON DELETE CASCADE,
    linker_id TEXT NOT NULL REFERENCES linkers(id) ON DELETE CASCADE,
    label     TEXT,
    position  INT  NOT NULL DEFAULT 0,
    PRIMARY KEY (study_id, linker_id)
);

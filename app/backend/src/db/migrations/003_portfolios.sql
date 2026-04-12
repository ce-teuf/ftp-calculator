-- ════════════════════════════════════════════════════════════════════════════
-- 003_portfolios.sql — FTP Simulator v7 — Module 3 : Portfolios
-- ════════════════════════════════════════════════════════════════════════════

-- ── Portfolios ───────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS portfolios (
    id          TEXT        NOT NULL PRIMARY KEY,
    name        TEXT        NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Vecteurs d'outstandings ───────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS outstanding_vectors (
    id          TEXT        NOT NULL PRIMARY KEY,
    name        TEXT        NOT NULL,
    description TEXT,
    -- [{"date":"2024-01","period_type":"observed","value":1500000000.0}, ...]
    rows_json   TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Matrices de schedules d'amortissement ────────────────────────────────────

CREATE TABLE IF NOT EXISTS amort_schedules (
    id                 TEXT        NOT NULL PRIMARY KEY,
    name               TEXT        NOT NULL,
    description        TEXT,
    -- Labels des buckets de tenor : ["1M","3M","6M","12M","24M","60M","120M"]
    bucket_labels_json TEXT        NOT NULL,
    -- [{"date":"2024-01","period_type":"observed","buckets":[0.02,0.05,0.10,0.20,0.30,0.20,0.13]}, ...]
    rows_json          TEXT        NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Associations portfolio ↔ vecteurs ────────────────────────────────────────

CREATE TABLE IF NOT EXISTS portfolio_vectors (
    portfolio_id TEXT NOT NULL REFERENCES portfolios(id)            ON DELETE CASCADE,
    vector_id    TEXT NOT NULL REFERENCES outstanding_vectors(id)   ON DELETE CASCADE,
    PRIMARY KEY (portfolio_id, vector_id)
);

-- ── Associations portfolio ↔ schedules ──────────────────────────────────────

CREATE TABLE IF NOT EXISTS portfolio_schedules (
    portfolio_id TEXT NOT NULL REFERENCES portfolios(id)   ON DELETE CASCADE,
    schedule_id  TEXT NOT NULL REFERENCES amort_schedules(id) ON DELETE CASCADE,
    PRIMARY KEY (portfolio_id, schedule_id)
);

-- ── Paires (vector, schedule) au sein d'un portfolio ────────────────────────

CREATE TABLE IF NOT EXISTS portfolio_pairs (
    id           TEXT NOT NULL PRIMARY KEY,
    portfolio_id TEXT NOT NULL REFERENCES portfolios(id)            ON DELETE CASCADE,
    vector_id    TEXT NOT NULL REFERENCES outstanding_vectors(id),
    schedule_id  TEXT NOT NULL REFERENCES amort_schedules(id),
    label        TEXT,
    UNIQUE (portfolio_id, vector_id, schedule_id)
);

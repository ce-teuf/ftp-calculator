-- ════════════════════════════════════════════════════════════════════════════
-- 002_hypercubes.sql — FTP Simulator v7 — Module 2 : Hypercubes
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS hypercubes (
    id               TEXT        NOT NULL PRIMARY KEY,
    name             TEXT        NOT NULL,
    description      TEXT,
    start_date       DATE        NOT NULL,
    end_date         DATE        NOT NULL,
    proj_end_date    DATE,
    time_granularity TEXT        NOT NULL DEFAULT 'monthly', -- daily | weekly | monthly
    status           TEXT        NOT NULL DEFAULT 'draft',   -- draft | active | archived
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS hypercube_matrices (
    hypercube_id TEXT NOT NULL REFERENCES hypercubes(id)    ON DELETE CASCADE,
    matrix_id    TEXT NOT NULL REFERENCES rate_matrices(id) ON DELETE RESTRICT,
    PRIMARY KEY (hypercube_id, matrix_id)
);

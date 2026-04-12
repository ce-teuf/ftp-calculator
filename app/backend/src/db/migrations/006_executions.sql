-- ════════════════════════════════════════════════════════════════════════════
-- 006_executions.sql — FTP Simulator v7 — Module 6 : Executions
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS executions (
    id            TEXT        NOT NULL PRIMARY KEY,
    study_id      TEXT        REFERENCES studies(id) ON DELETE SET NULL,
    study_name    TEXT,       -- snapshot du nom au moment du lancement
    label         TEXT,
    method        TEXT        NOT NULL DEFAULT 'maturity_matching',
    status        TEXT        NOT NULL DEFAULT 'pending', -- pending | running | completed | error
    result_json   TEXT,       -- JSON sérialisé du résultat complet
    duration_ms   BIGINT,
    error_message TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

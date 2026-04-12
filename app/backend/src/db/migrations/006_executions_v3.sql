CREATE TABLE IF NOT EXISTS executions_v3 (
    id            TEXT PRIMARY KEY,
    study_id      TEXT REFERENCES studies(id) ON DELETE SET NULL,
    label         TEXT,
    method        TEXT NOT NULL DEFAULT 'stock',
    status        TEXT NOT NULL DEFAULT 'pending',
    result_json   TEXT,
    error_message TEXT,
    duration_ms   BIGINT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_executions_v3_study ON executions_v3(study_id);

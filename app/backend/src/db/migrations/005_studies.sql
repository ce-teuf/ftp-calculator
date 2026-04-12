-- ════════════════════════════════════════════════════════════════════════════
-- 005_studies.sql — FTP Simulator v7 — Module 5 : Studies
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS studies (
    id          TEXT        NOT NULL PRIMARY KEY,
    name        TEXT        NOT NULL,
    description TEXT,
    status      TEXT        NOT NULL DEFAULT 'draft', -- draft | ready | archived
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Liaison many-to-many : study ↔ study_units (ordonnée par position)
CREATE TABLE IF NOT EXISTS study_items (
    study_id      TEXT NOT NULL REFERENCES studies(id)     ON DELETE CASCADE,
    study_unit_id TEXT NOT NULL REFERENCES study_units(id) ON DELETE CASCADE,
    label         TEXT,
    position      INT  NOT NULL DEFAULT 0,
    PRIMARY KEY (study_id, study_unit_id)
);

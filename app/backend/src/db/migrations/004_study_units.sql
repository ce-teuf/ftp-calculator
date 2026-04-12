-- ════════════════════════════════════════════════════════════════════════════
-- 004_study_units.sql — FTP Simulator v7 — Module 4 : Study unit builder
-- ════════════════════════════════════════════════════════════════════════════

-- ── Study units ───────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS study_units (
    id               TEXT        NOT NULL PRIMARY KEY,
    name             TEXT        NOT NULL,
    description      TEXT,
    hypercube_id     TEXT        NOT NULL REFERENCES hypercubes(id),
    portfolio_id     TEXT        NOT NULL REFERENCES portfolios(id),
    start_date       DATE        NOT NULL,
    granularity_rule TEXT        NOT NULL DEFAULT 'none', -- none | aggregate | interpolate
    is_valid         BOOLEAN     NOT NULL DEFAULT false,
    validation_log   TEXT,   -- JSON array of ValidationCheck (dernière passe)
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Assignments : paire × combinaison ────────────────────────────────────────
-- Une même paire peut avoir plusieurs assignments (plusieurs combinaisons).

CREATE TABLE IF NOT EXISTS study_unit_assignments (
    id                       TEXT        NOT NULL PRIMARY KEY,
    study_unit_id            TEXT        NOT NULL REFERENCES study_units(id) ON DELETE CASCADE,
    pair_id                  TEXT        NOT NULL REFERENCES portfolio_pairs(id),
    -- Combinaison = liste triée des IDs de matrices du hypercube (JSON array TEXT)
    -- ex. '["matrix_id_a","matrix_id_b"]'
    combination_matrix_ids   TEXT        NOT NULL,
    label                    TEXT,
    is_existing_stock        BOOLEAN     NOT NULL DEFAULT false,
    -- Profil FTP initial pour t=0 si is_existing_stock = true
    -- ex. '[{"tenor":"1M","rate":0.030},{"tenor":"3M","rate":0.032},...]'
    initial_ftp_profile_json TEXT,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

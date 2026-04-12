-- ════════════════════════════════════════════════════════════════════════════
-- 001_init.sql  —  FTP Simulator v7 — schéma initial (Module 1 : Matrices de taux)
--
-- Toutes les tables sont dans le schéma public.
-- Cette migration supprime les anciens schémas sc_* si présents.
-- ════════════════════════════════════════════════════════════════════════════

-- ── Drop des anciens schémas legacy ─────────────────────────────────────────
DROP SCHEMA IF EXISTS sc_series     CASCADE;
DROP SCHEMA IF EXISTS sc_curves     CASCADE;
DROP SCHEMA IF EXISTS sc_portfolios CASCADE;
DROP SCHEMA IF EXISTS sc_studies    CASCADE;

-- ── Module 1 : Types de risque (référentiel) ─────────────────────────────────

CREATE TABLE IF NOT EXISTS risk_types (
    key         TEXT NOT NULL PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT
);

INSERT INTO risk_types (key, label, description) VALUES
    ('base_rate',      'Taux sans risque',        'ESTR, SOFR'),
    ('credit_spread',  'Credit spread',            'Z-spread senior unsecured'),
    ('tlp',            'Term Liquidity Premium',   'Prime de liquidité à terme'),
    ('clp',            'Coussin de liquidité',     'LCR/NSFR réglementaire'),
    ('basis_risk',     'Basis risk',               'XCCY basis'),
    ('oas',            'Option-Adjusted Spread',   'Spread ajusté options — prépaiement'),
    ('capital_charge', 'Charge en capital',        'Coût des fonds propres — RWA × CoE'),
    ('xva',            'XVA',                      'CVA / MVA / KVA'),
    ('operational',    'Charge opérationnelle',    'Coûts opérationnels'),
    ('country_risk',   'Risque pays',              'Spread souverain'),
    ('concentration',  'Concentration',            'Add-on de concentration'),
    ('mrel_levy',      'MREL / Bail-in levy',      'Coût de la dette bail-inable / FDIC levy'),
    ('incentive',      'Incentive commercial',     'Ajustement commercial'),
    ('rollover',       'Risque de rollover',       'Coût de refinancement à l''échéance')
ON CONFLICT (key) DO NOTHING;

-- ── Module 1 : Matrices de taux ──────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS rate_matrices (
    id            TEXT        NOT NULL PRIMARY KEY,
    name          TEXT        NOT NULL,
    description   TEXT,
    currency      TEXT,
    status        TEXT        NOT NULL DEFAULT 'draft',   -- draft | active | archived
    interp_method TEXT        NOT NULL DEFAULT 'linear',  -- linear | cubic | flat_forward
    -- Tenors présents dans le fichier uploadé (non interpolés)
    -- Ex. '["1M","3M","6M","12M"]'
    tenors_json   TEXT        NOT NULL,
    -- Données brutes : une ligne par période
    -- Ex. '[{"date":"2024-01","period_type":"observed","values":[0.03,0.032]}]'
    rows_json     TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Module 1 : Liaison matrice ↔ types de risque ────────────────────────────

CREATE TABLE IF NOT EXISTS rate_matrix_risks (
    matrix_id TEXT NOT NULL REFERENCES rate_matrices(id) ON DELETE CASCADE,
    risk_key  TEXT NOT NULL REFERENCES risk_types(key),
    PRIMARY KEY (matrix_id, risk_key)
);

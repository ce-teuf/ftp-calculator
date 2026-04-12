-- Migration 003: Contracts (QuantLib-compatible) + Datasets (logical grouping)
-- ─────────────────────────────────────────────────────────────────────────────
-- Key design decisions:
--   • contracts  = source-of-truth for financial instruments (all QuantLib fields)
--   • datasets   = named logical groupings (like "Q1 2026 Portfolio" or "Test Set")
--   • dataset_items = many-to-many between datasets and ANY entity type
--   • Every entity can belong to multiple datasets
--   • portfolio_positions gains an optional contract_id FK for traceability

-- ── Datasets ─────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS datasets (
    id          TEXT        PRIMARY KEY,
    name        TEXT        NOT NULL,
    description TEXT,
    status      TEXT        NOT NULL DEFAULT 'active', -- active | frozen | archived
    source      TEXT        NOT NULL DEFAULT 'manual', -- manual | uploaded | generated
    as_of_date  DATE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by  TEXT
);

-- Many-to-many: dataset ↔ any entity
-- entity_type: 'contract' | 'rate_curve' | 'runoff_model' | 'portfolio' |
--              'rate_series' | 'execution'
CREATE TABLE IF NOT EXISTS dataset_items (
    dataset_id  TEXT        NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    entity_type TEXT        NOT NULL,
    entity_id   TEXT        NOT NULL,
    added_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (dataset_id, entity_type, entity_id)
);

CREATE INDEX IF NOT EXISTS idx_dataset_items_entity ON dataset_items(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_dataset_items_dataset ON dataset_items(dataset_id);

-- ── Contracts (full QuantLib instrument description) ─────────────────────────

CREATE TABLE IF NOT EXISTS contracts (
    -- Identity
    id                      TEXT        PRIMARY KEY,
    contract_id             TEXT        NOT NULL UNIQUE, -- e.g. CNT-0247ACD546CF
    contract_type           TEXT        NOT NULL,        -- PAM | ANNUITE | MORTGAGE | BULLET |
                                                         -- REVOLVER | COMMERCIAL_LOAN |
                                                         -- DEMAND_DEPOSIT | SAVINGS |
                                                         -- TERM_DEPOSIT | CERTIFICATE_OF_DEPOSIT
    side                    TEXT        NOT NULL DEFAULT 'ASSET', -- ASSET | PASSIF

    -- Organisational
    seller_id               TEXT,       -- links to data_entities/sales.csv
    branch_code             TEXT,       -- US | FR | ES | DE
    currency                TEXT        NOT NULL DEFAULT 'EUR',
    rating                  TEXT,       -- AAA | AA | A | BBB | BB | B | NR

    -- Financial terms
    notional                DOUBLE PRECISION NOT NULL,
    rate_type               TEXT        DEFAULT 'fixed',  -- fixed | floating
    interest_rate           DOUBLE PRECISION,             -- contractual rate
    spread_over_index       DOUBLE PRECISION,             -- for floating rate contracts

    -- Schedule / QuantLib terms
    settlement_date         DATE,
    maturity_date           DATE,
    tenor_months            INTEGER,
    payment_frequency       TEXT,       -- monthly | quarterly | semiannual | annual
    day_count               TEXT,       -- 30/360 | ACT/360 | ACT/365 | ACT/ACT
    business_day_convention TEXT,       -- Following | ModifiedFollowing | Preceding
    amortization_type       TEXT,       -- linear | constant_installment | bullet | behavioral
    prepayment_allowed      BOOLEAN     NOT NULL DEFAULT FALSE,
    prepayment_penalty      DOUBLE PRECISION NOT NULL DEFAULT 0,
    guarantee_type          TEXT,       -- none | personal | mortgage | commercial

    -- FTP / ALM overrides (set by modelers, override contractual schedule)
    runoff_model_id         TEXT        REFERENCES runoff_models(id),
    profiles_json           TEXT,       -- computed amortisation profile [0..1] per period
    rates_json              TEXT,       -- market rates for FTP computation (per tenor)
    risk_weight             DOUBLE PRECISION NOT NULL DEFAULT 1.0,  -- Basel RWA weight

    -- Audit
    metadata_json           TEXT,
    source_dataset_id       TEXT        REFERENCES datasets(id),   -- first dataset this came from
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_contracts_contract_id   ON contracts(contract_id);
CREATE INDEX IF NOT EXISTS idx_contracts_side          ON contracts(side);
CREATE INDEX IF NOT EXISTS idx_contracts_branch        ON contracts(branch_code);
CREATE INDEX IF NOT EXISTS idx_contracts_contract_type ON contracts(contract_type);
CREATE INDEX IF NOT EXISTS idx_contracts_currency      ON contracts(currency);

-- ── Extend portfolio_positions with optional contract traceability ────────────

ALTER TABLE portfolio_positions
    ADD COLUMN IF NOT EXISTS contract_id TEXT REFERENCES contracts(id);

CREATE INDEX IF NOT EXISTS idx_positions_contract ON portfolio_positions(contract_id);

-- ── Uploaded files tracking ──────────────────────────────────────────────────
-- Tracks raw CSV/JSON files the user uploaded (before parsing into contracts)

CREATE TABLE IF NOT EXISTS uploaded_files (
    id           TEXT        PRIMARY KEY,
    filename     TEXT        NOT NULL,
    content_type TEXT        NOT NULL DEFAULT 'text/csv',
    size_bytes   INTEGER,
    row_count    INTEGER,
    dataset_id   TEXT        REFERENCES datasets(id),
    parse_status TEXT        NOT NULL DEFAULT 'pending', -- pending | success | error
    parse_error  TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Freeze log ───────────────────────────────────────────────────────────────
-- Tracks dataset freeze operations (exports to CSV in original format)

CREATE TABLE IF NOT EXISTS freeze_log (
    id          TEXT        PRIMARY KEY,
    dataset_id  TEXT        NOT NULL REFERENCES datasets(id),
    frozen_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    row_counts  TEXT,       -- JSON: {"contracts": 1000, "rate_curves": 3, ...}
    frozen_by   TEXT
);

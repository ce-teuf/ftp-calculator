-- Migration 008: Finalization
-- ─────────────────────────────────────────────────────────────────────────────
-- Fixes and additions to align the DB with the full v3 plan.
-- All statements are idempotent (safe to re-run).

-- ── 1. Fix executions_v3 missing index (name collision with ta_executions) ────

CREATE INDEX IF NOT EXISTS idx_executions_v3_study_id ON executions_v3(study_id);

-- ── 2. Fix sc_series.ta_series column name ────────────────────────────────────

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'sc_series' AND table_name = 'ta_series'
          AND column_name = 'column_name'
    ) THEN
        ALTER TABLE sc_series.ta_series RENAME COLUMN column_name TO value;
    END IF;
END$$;

-- ── 3. FK: sc_studies.ta_study_linkers → sc_studies.ta_studies / ta_linkers ──

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'fk_study_linkers_study'
    ) THEN
        ALTER TABLE sc_studies.ta_study_linkers
            ADD CONSTRAINT fk_study_linkers_study
            FOREIGN KEY (id) REFERENCES sc_studies.ta_studies(id) ON DELETE CASCADE;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'fk_study_linkers_linker'
    ) THEN
        ALTER TABLE sc_studies.ta_study_linkers
            ADD CONSTRAINT fk_study_linkers_linker
            FOREIGN KEY (linker_id) REFERENCES sc_studies.ta_linkers(id) ON DELETE CASCADE;
    END IF;
END$$;

-- ── 4. Note: fk_linkers_cube skipped — UUID/TEXT type mismatch ────────────────
-- sc_studies.ta_linkers.cube_id is UUID, public.curve_cubes.id is TEXT.
-- A comment on the column documents the logical reference instead.

-- ── 5. Index on contracts(seller_id) ─────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_contracts_seller ON contracts(seller_id);

-- ── 6. Index on portfolio_positions(branch, seller) ──────────────────────────

CREATE INDEX IF NOT EXISTS idx_positions_branch_seller
    ON portfolio_positions(branch, seller);

-- ── 7. FK constraints on contracts org fields ─────────────────────────────────

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'fk_contracts_branch'
    ) THEN
        ALTER TABLE contracts
            ADD CONSTRAINT fk_contracts_branch
            FOREIGN KEY (branch_code) REFERENCES org_branches(branch_code)
            ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'fk_contracts_seller'
    ) THEN
        ALTER TABLE contracts
            ADD CONSTRAINT fk_contracts_seller
            FOREIGN KEY (seller_id) REFERENCES org_sellers(id)
            ON DELETE SET NULL DEFERRABLE INITIALLY DEFERRED;
    END IF;
END$$;

-- ── 8. Contract-to-schedule cross-reference (TEXT FK to match contracts.id) ──

ALTER TABLE sc_portfolios.ta_schedules_metadata
    ADD COLUMN IF NOT EXISTS contract_id TEXT REFERENCES public.contracts(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_schedules_meta_contract
    ON sc_portfolios.ta_schedules_metadata(contract_id);

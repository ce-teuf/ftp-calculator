-- ════════════════════════════════════════════════════════════════════════════
-- 007_schedule_type.sql — FTP Simulator — Ajout du type de schedule
--
-- stock          : schedule de stock (encours existant, amortissement du book en cours)
-- new_production : schedule de nouvelle production (flux entrants, nouvelles originations)
--
-- Valeur par défaut : 'stock' (comportement historique).
-- ════════════════════════════════════════════════════════════════════════════

ALTER TABLE amort_schedules
    ADD COLUMN IF NOT EXISTS schedule_type TEXT NOT NULL DEFAULT 'stock';

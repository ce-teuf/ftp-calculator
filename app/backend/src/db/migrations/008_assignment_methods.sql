-- ════════════════════════════════════════════════════════════════════════════
-- 008_assignment_methods.sql — FTP Simulator — Méthodes de calcul par assignment
--
-- Chaque assignment peut désormais spécifier une ou plusieurs méthodes FTP
-- à exécuter. Valeurs valides : Stock | Flux | Duration | Pool | Refinancing
--                               Floating | Behavioral | Replicating | Ldi
--
-- Valeur par défaut : '["Stock"]'
-- ════════════════════════════════════════════════════════════════════════════

ALTER TABLE study_unit_assignments
    ADD COLUMN IF NOT EXISTS methods_json TEXT NOT NULL DEFAULT '["Stock"]';

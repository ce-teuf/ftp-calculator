-- FTP Simulator — données de développement (seed local)
-- Chargé par `make dev-seed` (pas de serveur FTP requis)
-- Usage : psql "$DATABASE_URL" -f data/dev/seed.sql

-- ── Courbes de taux ─────────────────────────────────────────────────────────

INSERT INTO rate_curves (id, name, component, currency, version, status,
    tenors_json, values_json, source, created_at)
VALUES
    ('curve-ois-eur',
     'OIS €STR 2026-04', 'base_rate', 'EUR', 1, 'approved',
     '["1M","3M","6M","1Y","2Y","3Y","5Y","7Y","10Y","15Y","20Y","30Y"]',
     '[0.0310,0.0318,0.0325,0.0332,0.0340,0.0345,0.0352,0.0358,0.0365,0.0370,0.0373,0.0375]',
     'Manual', NOW()),
    ('curve-tlp-eur',
     'TLP Senior 2026-04', 'tlp', 'EUR', 1, 'approved',
     '["1M","3M","6M","1Y","2Y","3Y","5Y","7Y","10Y","15Y","20Y","30Y"]',
     '[0.0015,0.0020,0.0025,0.0035,0.0045,0.0055,0.0070,0.0080,0.0090,0.0095,0.0098,0.0100]',
     'Manual', NOW()),
    ('curve-cs-eur',
     'Credit Spread AA 2026-04', 'credit_spread', 'EUR', 1, 'draft',
     '["1M","3M","6M","1Y","2Y","3Y","5Y","7Y","10Y"]',
     '[0.0005,0.0008,0.0010,0.0015,0.0020,0.0025,0.0030,0.0035,0.0040]',
     'Manual', NOW())
ON CONFLICT (id) DO NOTHING;

-- ── Modèles de runoff ────────────────────────────────────────────────────────

INSERT INTO runoff_models (id, name, product_type, category, version, status,
    method, profile_json, parameters_json, created_at)
VALUES
    ('rm-immo-20y',
     'Prêt immobilier linéaire 20Y', 'mortgage', 'retail', 1, 'approved',
     'contractual',
     '[1.0,0.95,0.9,0.85,0.8,0.75,0.7,0.65,0.6,0.55,0.5,0.45,0.4,0.35,0.3,0.25,0.2,0.15,0.1,0.05,0.0]',
     '{"original_term_months": 240}', NOW()),
    ('rm-nmd-retail',
     'NMD dépôt vue retail (core 70%)', 'nmd', 'retail', 1, 'approved',
     'behavioral_exponential',
     '[1.0,0.979,0.958,0.938,0.919,0.900,0.881,0.863,0.845,0.827,0.810,0.793,0.776,0.760,0.744,0.728,0.713,0.698,0.683,0.669,0.655,0.641,0.627,0.614,0.601,0.588,0.576,0.564,0.552,0.540,0.529,0.518,0.507,0.496,0.486,0.475,0.465,0.455,0.446,0.436,0.427,0.418,0.409,0.400,0.392,0.384,0.375,0.368,0.360,0.352,0.345,0.337,0.330,0.323,0.317,0.310,0.304,0.297,0.291,0.285,0.279]',
     '{"lambda": 0.021, "wal": 47.6, "core_ratio": 0.7, "eba_capped": false}', NOW())
ON CONFLICT (id) DO NOTHING;

-- ── Portefeuille de démonstration ────────────────────────────────────────────

INSERT INTO portfolios (id, name, description, version, status, as_of_date, created_at)
VALUES
    ('ptf-demo',
     'Portefeuille Retail Demo', 'Jeu de données de développement — 10 positions', 1, 'draft',
     CURRENT_DATE, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO portfolio_positions
    (id, portfolio_id, position_ref, product_type, branch, seller, currency,
     outstanding, origination_date, maturity_date, client_rate, risk_weight,
     profiles_json, rates_json)
VALUES
    ('pos-001','ptf-demo','LOAN-001','mortgage','Paris Sud','Martin','EUR',
     500000,'2023-01-15','2043-01-15',0.0420,0.35,
     '[1.0,0.95,0.90,0.85,0.80,0.75,0.70,0.65,0.60,0.55,0.50,0.45,0.40]',
     '[0.0360,0.0368,0.0375,0.0382,0.0388,0.0393,0.0398,0.0402,0.0406,0.0410,0.0413,0.0415]'),
    ('pos-002','ptf-demo','LOAN-002','mortgage','Lyon','Dupont','EUR',
     320000,'2022-06-01','2042-06-01',0.0395,0.35,
     '[1.0,0.95,0.90,0.85,0.80,0.75,0.70,0.65,0.60,0.55,0.50,0.45,0.40]',
     '[0.0360,0.0368,0.0375,0.0382,0.0388,0.0393,0.0398,0.0402,0.0406,0.0410,0.0413,0.0415]'),
    ('pos-003','ptf-demo','LOAN-003','consumer_loan','Bordeaux','Petit','EUR',
     45000,'2024-03-01','2029-03-01',0.0650,0.75,
     '[1.0,0.83,0.67,0.50,0.33,0.17,0.0]',
     '[0.0325,0.0340,0.0352,0.0362,0.0370,0.0378]'),
    ('pos-004','ptf-demo','LOAN-004','corporate_loan','Paris Nord','Bernard','EUR',
     2000000,'2024-01-01','2029-01-01',0.0480,1.00,
     '[1.0,0.80,0.60,0.40,0.20,0.0]',
     '[0.0332,0.0348,0.0358,0.0366,0.0373]'),
    ('pos-005','ptf-demo','DEP-001','nmd','Paris Sud','Martin','EUR',
     800000,'2020-01-01',NULL,0.0100,0.0,
     '[1.0,0.979,0.958,0.938,0.919,0.900,0.881,0.863,0.845,0.827,0.810,0.793,0.776,0.760,0.744,0.728,0.713]',
     '[0.0310,0.0318,0.0325,0.0332,0.0340,0.0345,0.0352,0.0358,0.0365,0.0370,0.0373,0.0375,0.0376,0.0377,0.0378,0.0378]'),
    ('pos-006','ptf-demo','LOAN-005','mortgage','Toulouse','Rousseau','EUR',
     280000,'2021-09-01','2041-09-01',0.0375,0.35,
     '[1.0,0.95,0.90,0.85,0.80,0.75,0.70,0.65,0.60,0.55,0.50,0.45,0.40]',
     '[0.0360,0.0368,0.0375,0.0382,0.0388,0.0393,0.0398,0.0402,0.0406,0.0410,0.0413,0.0415]'),
    ('pos-007','ptf-demo','LOAN-006','sme_loan','Lyon','Dupont','EUR',
     150000,'2023-07-01','2028-07-01',0.0520,0.85,
     '[1.0,0.80,0.60,0.40,0.20,0.0]',
     '[0.0332,0.0348,0.0358,0.0366,0.0373]'),
    ('pos-008','ptf-demo','DEP-002','savings','Bordeaux','Petit','EUR',
     450000,'2019-06-01',NULL,0.0300,0.0,
     '[1.0,0.90,0.80,0.70,0.60,0.50,0.40,0.30,0.20,0.10,0.05,0.0]',
     '[0.0310,0.0318,0.0325,0.0332,0.0340,0.0345,0.0352,0.0358,0.0365,0.0370,0.0373]'),
    ('pos-009','ptf-demo','LOAN-007','mortgage','Nice','Blanc','EUR',
     620000,'2022-11-01','2052-11-01',0.0450,0.35,
     '[1.0,0.97,0.93,0.90,0.87,0.83,0.80,0.77,0.73,0.70,0.67,0.63,0.60]',
     '[0.0360,0.0368,0.0375,0.0382,0.0388,0.0393,0.0398,0.0402,0.0406,0.0410,0.0413,0.0415]'),
    ('pos-010','ptf-demo','LOAN-008','corporate_loan','Paris Nord','Bernard','EUR',
     5000000,'2025-01-01','2030-01-01',0.0460,1.00,
     '[1.0,0.80,0.60,0.40,0.20,0.0]',
     '[0.0332,0.0348,0.0358,0.0366,0.0373]')
ON CONFLICT (id) DO NOTHING;

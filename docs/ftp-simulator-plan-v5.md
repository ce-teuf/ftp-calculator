# FTP Simulator — Plan V5

---

## Sommaire

- [Stack technique commun](#stack-technique-commun)
- [Architecture globale](#architecture-globale)
- [Module 1 — Séries temporelles](#module-1--séries-temporelles)
- [Module 2 — Courbes de taux](#module-2--courbes-de-taux)
- [Module 3 — Curve Stacker](#module-3--curve-stacker)
- [Module 4 — Hypercube](#module-4--hypercube)
- [Module 5 — Portfolio](#module-5--portfolio)
- [Module 6 — Linker](#module-6--linker)
- [Module 7 — Studies](#module-7--studies)
- [Module 8 — Execution](#module-8--execution)
- [Module 9 — Dashboard](#module-9--dashboard)

---

## Stack technique commun

| Couche    | Choix                                                                 |
|-----------|-----------------------------------------------------------------------|
| Frontend  | Svelte 5 + SvelteKit 2 (runes), Tailwind CSS v4, ECharts 5           |
| Backend   | Rust + Axum 0.8, seul autorisé à lire/écrire la BDD                  |
| Base de données | PostgreSQL 18 + extension TimescaleDB                          |
| Migrations | sqlx migrate                                                         |
| Python Lab | Pyodide (WASM) + CodeMirror 6, exécuté dans le navigateur           |

**Principe fondamental** : le frontend ne communique jamais directement avec la base de données. Tout passe par le service Rust.

---

## Architecture globale

```
Séries temporelles (Module 1)
        │
        ▼
  Courbes de taux (Module 2) ◄── CSV ou Pyodide
        │
        ▼
  Curve Stacker (Module 3) ◄── combinaisons sans doublon de risque
        │
        ▼
  Hypercube (Module 4) ◄── dimension temporelle + projections
        │
        │◄──── Portfolio (Module 5) ◄── indépendant (outstandings + schedules)
        │
        ▼
     Linker (Module 6) ◄── jonction Hypercube × Portfolio + vérifications
        │
        ▼
    Studies (Module 7) ◄── regroupement logique de linkers
        │
        ▼
  Execution (Module 8) ◄── maturity matching → matrices FTP
        │
        ▼
  Dashboard (Module 9) ◄── visualisation read-only
```

---

## Module 1 — Séries temporelles

### Objectif

Stocker des séries temporelles quotidiennes (taux d'intérêt, spreads) et leurs projections.  
Une série peut être **réelle** (`is_actual = true`) ou **contrefactuelle** (`is_actual = false`, issue d'un modèle ou scénario).  
Les projections peuvent être liées entre plusieurs séries via un `group_id` (groupe de modélisation).

### Base de données

```sql
-- Métadonnées des séries
CREATE TABLE ta_timeseries_metadata (
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    unit        VARCHAR(50),
    is_actual   BOOLEAN NOT NULL DEFAULT true,  -- true = données réelles, false = contrefactuelles
    component   TEXT,   -- base_rate | credit_spread | tlp | ibor | basis_risk | …
    currency    TEXT,
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

-- Valeurs quotidiennes (hypertable TimescaleDB)
CREATE TABLE ta_timeseries (
    time      DATE NOT NULL,
    series_id INTEGER REFERENCES ta_timeseries_metadata(id) ON DELETE CASCADE,
    value     DOUBLE PRECISION NOT NULL
);
SELECT create_hypertable('ta_timeseries', 'time');
CREATE INDEX idx_ts_series_time ON ta_timeseries (series_id, time DESC);

-- Métadonnées des projections
CREATE TABLE ta_timeseries_proj_metadata (
    id           SERIAL PRIMARY KEY,
    series_id    INTEGER REFERENCES ta_timeseries_metadata(id) ON DELETE CASCADE,
    start_date   DATE NOT NULL,
    horizon_days INTEGER NOT NULL,
    group_id     UUID,   -- NULL = projection indépendante ; même UUID = projections liées
    created_at   TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(series_id, start_date, group_id)
);

-- Valeurs projetées (hypertable TimescaleDB)
CREATE TABLE ta_timeseries_proj (
    proj_id    INTEGER REFERENCES ta_timeseries_proj_metadata(id) ON DELETE CASCADE,
    offset_day INTEGER NOT NULL,   -- 0..horizon_days
    value      DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (proj_id, offset_day)
);
SELECT create_hypertable('ta_timeseries_proj', 'offset_day', chunk_time_interval => 30);
```

**Règle de groupe** : des projections de séries différentes partant de la même date et portant le même `group_id` sont conceptuellement liées (même scénario / même modèle). Cette liaison est optionnelle.

### Backend (API REST)

| Méthode | Route                                    | Description                                         |
|---------|------------------------------------------|-----------------------------------------------------|
| GET     | `/api/timeseries`                        | Liste des séries (avec filtres : composante, devise, is_actual) |
| POST    | `/api/timeseries`                        | Créer une série (métadonnées)                       |
| GET     | `/api/timeseries/:id`                    | Détail + valeurs (avec plage de dates)              |
| DELETE  | `/api/timeseries/:id`                    | Supprimer                                           |
| POST    | `/api/timeseries/:id/upload`             | Charger des valeurs CSV (date, valeur)              |
| GET     | `/api/timeseries/:id/projections`        | Lister les projections d'une série                  |
| POST    | `/api/timeseries/:id/projections`        | Créer une projection (avec valeurs et group_id opt.)|
| DELETE  | `/api/timeseries/projections/:proj_id`   | Supprimer une projection                            |
| GET     | `/api/timeseries/projection-groups`      | Lister les group_id existants                       |

### Frontend

- **Page `/time-series`** :
  - Liste des séries avec badge réel/contrefactuel
  - Onglet **Chargement CSV** (upload de valeurs)
  - Onglet **Projections** : créer/visualiser des projections, assigner ou créer un `group_id`
  - Deux graphiques ECharts : historique de la série + superposition des projections
  - Filtre par composante, devise, is_actual

---

## Module 2 — Courbes de taux

### Objectif

Générer des courbes de taux à partir des séries temporelles (module 1) ou par import CSV.  
Chaque courbe est associée à **un ou plusieurs types de risque** parmi 14.  
Si plusieurs risques sont attribués à une même courbe, ils deviennent **indissociables** lors des analyses ultérieures.

### Types de risque (14)

| Clé               | Description                                       |
|-------------------|---------------------------------------------------|
| `base_rate`       | Taux sans risque (ESTR, SOFR)                     |
| `credit_spread`   | Z-spread senior unsecured                         |
| `tlp`             | Term Liquidity Premium                            |
| `clp`             | Coussin de liquidité réglementaire (LCR/NSFR)     |
| `basis_risk`      | XCCY basis                                        |
| `oas`             | Option-Adjusted Spread (prépaiement)              |
| `capital_charge`  | Coût des fonds propres (RWA × CoE)                |
| `xva`             | CVA / MVA / KVA                                   |
| `operational`     | Charge opérationnelle                             |
| `country_risk`    | Spread souverain                                  |
| `concentration`   | Add-on de concentration                           |
| `mrel_levy`       | Coût de la dette bail-inable / FDIC levy          |
| `incentive`       | Ajustement commercial                             |
| `rollover`        | Coût de refinancement à l'échéance                |

### Base de données

```sql
-- Courbes de taux
CREATE TABLE ta_rate_curves (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    currency        TEXT,
    version         INT DEFAULT 1,
    status          TEXT DEFAULT 'draft',   -- draft | active | archived
    valid_from      DATE,
    tenors_json     TEXT NOT NULL,   -- ["1M","3M","1Y",…]
    values_json     TEXT NOT NULL,   -- [0.03, 0.032, …]
    series_name     TEXT,            -- série sous-jacente optionnelle (lien module 1)
    has_mixed_risks BOOLEAN DEFAULT false,   -- true si plusieurs risques → indissociables
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Table de référence des 14 types de risque
CREATE TABLE ta_risk_types (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT
);

-- Liaison many-to-many courbe ↔ risques
CREATE TABLE ta_curve_risks (
    curve_id  TEXT REFERENCES ta_rate_curves(id) ON DELETE CASCADE,
    risk_key  TEXT REFERENCES ta_risk_types(key),
    PRIMARY KEY (curve_id, risk_key)
);
```

### Backend (API REST)

| Méthode | Route                          | Description                                       |
|---------|--------------------------------|---------------------------------------------------|
| GET     | `/api/curves`                  | Liste (avec filtres statut, risque, devise)        |
| POST    | `/api/curves`                  | Créer (mode CSV ou pyodide)                       |
| GET     | `/api/curves/:id`              | Détail avec risques associés                      |
| PUT     | `/api/curves/:id`              | Mettre à jour                                     |
| DELETE  | `/api/curves/:id`              | Supprimer                                         |
| GET     | `/api/risk-types`              | Liste des 14 types de risque                      |

### Frontend

- **Page `/curves`** :
  - Liste des courbes avec risques associés (badges couleur), statut, date d'effet
  - Onglet **Chargement CSV** : format fixe (tenors + valeurs), sélection multiple des risques avec avertissement si plusieurs
  - Onglet **Python Lab** (Pyodide) : éditeur CodeMirror, accès aux séries du module 1, script Python → courbe générée → sélection des risques
  - Interpolation des tenors : `linear` | `cubic` | `flat_forward`

---

## Module 3 — Curve Stacker

### Objectif

À partir d'une sélection de N courbes, générer toutes les combinaisons valides (de taille 1 à N) sans doublon de type de risque.

**Règle** : dans chaque combinaison générée, chaque type de risque ne peut apparaître qu'une seule fois au maximum. Si une courbe porte plusieurs risques (indissociables), elle exclut toute autre courbe portant l'un de ces risques.

### Base de données

```sql
-- Stacks (ensembles de courbes validés)
CREATE TABLE ta_curve_stacks (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    status     TEXT DEFAULT 'draft',   -- draft | active | archived
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Composantes d'un stack
CREATE TABLE ta_curve_stack_components (
    stack_id       TEXT REFERENCES ta_curve_stacks(id) ON DELETE CASCADE,
    curve_id       TEXT REFERENCES ta_rate_curves(id),
    position       INT NOT NULL,
    label          TEXT,
    weight         REAL DEFAULT 1.0,
    interp_method  TEXT DEFAULT 'linear',   -- linear | cubic | flat_forward
    PRIMARY KEY (stack_id, curve_id)
);
```

### Backend (API REST)

| Méthode | Route                                   | Description                                          |
|---------|-----------------------------------------|------------------------------------------------------|
| GET     | `/api/stacks`                           | Liste des stacks                                     |
| POST    | `/api/stacks`                           | Créer un stack manuellement                          |
| GET     | `/api/stacks/:id`                       | Détail avec composantes                              |
| PUT     | `/api/stacks/:id`                       | Mettre à jour                                        |
| DELETE  | `/api/stacks/:id`                       | Supprimer                                            |
| POST    | `/api/stacks/generate-combinations`     | Générer toutes les combinaisons valides depuis une sélection de courbes |

**Algorithme `generate-combinations`** :
- Entrée : liste d'IDs de courbes
- Pour chaque sous-ensemble (taille 1..N) : vérifier que l'union des `risk_key` de toutes les courbes ne contient aucun doublon
- Retourne la liste des sous-ensembles valides (= stacks candidats)

### Frontend

- **Page `/stacks`** :
  - Liste des stacks existants
  - Sélection multiple de courbes → bouton « Générer les combinaisons »
  - Affichage des combinaisons valides avec compteur de risques couverts
  - Option pour sauvegarder une combinaison comme stack nommé
  - Visualisation graphique des courbes superposées (ECharts)

---

## Module 4 — Hypercube

### Objectif

Ajouter la dimension temporelle à chaque stack. Pour chaque stack, construire une matrice **L × M** par courbe :
- **L** = nombre de pas de temps (daily / weekly / monthly)
- **M** = nombre de tenors mensuels interpolés

Le cube couvre deux périodes :
1. **Période réalisée / contrefactuelle** (`start_date` → `end_date`) — séries historiques ou contrefactuelles du module 1
2. **Période de projection** (`end_date` → `proj_end_date`) — projections du module 1

Si plusieurs projections existent à `end_date` (différents `group_id`), la matrice gagne une troisième dimension (tenseur L × M × nb_scénarios).

### Base de données

```sql
CREATE TABLE ta_hypercubes (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    stack_id         TEXT REFERENCES ta_curve_stacks(id),
    start_date       DATE NOT NULL,
    end_date         DATE NOT NULL,           -- fin période réalisée
    proj_end_date    DATE,                    -- fin période projection (NULL = pas de projection)
    time_granularity TEXT DEFAULT 'monthly',  -- daily | weekly | monthly
    status           TEXT DEFAULT 'draft',
    -- validation résultats
    is_valid         BOOLEAN DEFAULT false,
    validation_log   TEXT,
    created_at       TIMESTAMPTZ DEFAULT NOW()
);
```

**Pas de stockage des matrices** dans la BDD : elles sont calculées à la volée lors de l'exécution (module 8). Le cube est une configuration, pas un résultat.

### Backend (API REST)

| Méthode | Route                        | Description                                               |
|---------|------------------------------|-----------------------------------------------------------|
| GET     | `/api/cubes`                 | Liste                                                     |
| POST    | `/api/cubes`                 | Créer                                                     |
| GET     | `/api/cubes/:id`             | Détail                                                    |
| PUT     | `/api/cubes/:id`             | Mettre à jour                                             |
| DELETE  | `/api/cubes/:id`             | Supprimer                                                 |
| POST    | `/api/cubes/:id/validate`    | Vérifier la disponibilité des données (séries + proj.) pour les dates configurées |

**Validation** (endpoint `validate`) :
- Les séries sous-jacentes à chaque courbe du stack couvrent-elles [`start_date`, `end_date`] ?
- À `end_date`, existe-t-il au moins une projection pour chaque série ?
- Cohérence de la granularité temporelle

### Frontend

- **Page `/cubes`** :
  - Liste des cubes avec statut de validation
  - Formulaire : sélection d'un stack, plage de dates, granularité, date fin projection
  - Bouton « Valider la disponibilité » → affichage du rapport (séries manquantes, projections absentes)
  - Affichage du nombre de scénarios disponibles (groupe de projections distincts à `end_date`)

---

## Module 5 — Portfolio

### Objectif

Module **indépendant** des modules précédents. Permet de gérer des portefeuilles définis par la combinaison de :
- **Vecteurs d'outstandings** : encours au fil du temps (dimension L × 1)
- **Matrices de schedules d'amortissement** : profil d'amortissement par pas de temps (dimension L × M_buckets)

Un portfolio est une relation many-to-many entre vecteurs et matrices. Granularités supportées : daily, weekly, monthly.

### Base de données

```sql
CREATE TABLE ta_portfolios (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    description      TEXT,
    time_granularity TEXT DEFAULT 'monthly',   -- daily | weekly | monthly
    created_at       TIMESTAMPTZ DEFAULT NOW()
);

-- Vecteurs d'outstandings
CREATE TABLE ta_outstanding_vectors (
    id           TEXT PRIMARY KEY,
    portfolio_id TEXT REFERENCES ta_portfolios(id) ON DELETE CASCADE,
    label        TEXT,
    data_json    TEXT NOT NULL,   -- [{"date":"2025-01","value":1500000000.0}, …]
    row_order    INT DEFAULT 0
);

-- Matrices de schedules d'amortissement
CREATE TABLE ta_amort_schedules (
    id           TEXT PRIMARY KEY,
    portfolio_id TEXT REFERENCES ta_portfolios(id) ON DELETE CASCADE,
    label        TEXT,
    data_json    TEXT NOT NULL,   -- [{"date":"2025-01","buckets":[0.02,0.05,…]}, …]
    row_order    INT DEFAULT 0
);

-- Liaison many-to-many : vector ↔ schedule
CREATE TABLE ta_portfolio_links (
    portfolio_id  TEXT REFERENCES ta_portfolios(id) ON DELETE CASCADE,
    vector_id     TEXT REFERENCES ta_outstanding_vectors(id),
    schedule_id   TEXT REFERENCES ta_amort_schedules(id),
    PRIMARY KEY (vector_id, schedule_id)
);
```

### Backend (API REST)

| Méthode | Route                                         | Description                               |
|---------|-----------------------------------------------|-------------------------------------------|
| GET     | `/api/portfolios`                             | Liste                                     |
| POST    | `/api/portfolios`                             | Créer                                     |
| GET     | `/api/portfolios/:id`                         | Détail avec lignes                        |
| PUT     | `/api/portfolios/:id`                         | Renommer / modifier description           |
| DELETE  | `/api/portfolios/:id`                         | Supprimer                                 |
| POST    | `/api/portfolios/:id/vectors/upload`          | Upload CSV → vecteur d'outstandings       |
| POST    | `/api/portfolios/:id/schedules/upload`        | Upload CSV → matrice de schedules         |
| POST    | `/api/portfolios/:id/links`                   | Créer une liaison vector ↔ schedule       |
| DELETE  | `/api/portfolios/:id/links/:link_id`          | Supprimer une liaison                     |
| DELETE  | `/api/portfolio-vectors/:id`                  | Supprimer un vecteur                      |
| DELETE  | `/api/portfolio-schedules/:id`                | Supprimer un schedule                     |

### Frontend

- **Page `/portfolios`** :
  - Liste des portfolios avec granularité et nombre de lignes
  - Vue détaillée : aperçu tableau des outstanding vectors et schedules, graphique ECharts
  - Onglet **Upload CSV** (outstanding vector ou amortization schedule)
  - Gestion des liaisons many-to-many (glisser-déposer ou cases à cocher)
  - Ajout incrémental de données (append CSV)

---

## Module 6 — Linker

### Objectif

Jonction entre un **hypercube** (module 4) et un **portfolio** (module 5).  
Vérifie la disponibilité des données et la concordance des dimensions. Gère les conversions de granularité.

### Base de données

```sql
CREATE TABLE ta_linkers (
    id               TEXT PRIMARY KEY,
    name             TEXT,
    cube_id          TEXT REFERENCES ta_hypercubes(id),
    portfolio_id     TEXT REFERENCES ta_portfolios(id),
    start_date       DATE NOT NULL,
    -- résultat de validation
    is_valid         BOOLEAN DEFAULT false,
    validation_log   TEXT,
    -- conversion de granularité (si nécessaire)
    granularity_rule TEXT,   -- none | aggregate | interpolate
    created_at       TIMESTAMPTZ DEFAULT NOW()
);
```

**Vérifications** :
1. La granularité du cube et du portfolio sont-elles compatibles (ou convertibles) ?
2. Les dimensions L correspondent-elles (nombre de pas de temps sur la plage start_date → end_date) ?
3. Les données d'outstandings couvrent-elles la plage du cube ?
4. Alertes en cas d'incohérence

### Backend (API REST)

| Méthode | Route                        | Description                               |
|---------|------------------------------|-------------------------------------------|
| GET     | `/api/linkers`               | Liste                                     |
| POST    | `/api/linkers`               | Créer (cube_id + portfolio_id)            |
| GET     | `/api/linkers/:id`           | Détail avec rapport de validation         |
| DELETE  | `/api/linkers/:id`           | Supprimer                                 |
| POST    | `/api/linkers/:id/validate`  | Lancer la validation des dimensions       |

### Frontend

- **Page `/linkers`** :
  - Liste des linkers avec statut de validation (vert/orange/rouge)
  - Formulaire : sélection cube + portfolio + date de départ + règle de conversion de granularité
  - Bouton « Valider » → rapport détaillé (dimensions, dates, alertes)
  - Aperçu de la structure unifiée résultante

---

## Module 7 — Studies

### Objectif

Regrouper plusieurs linkers dans une **unité logique de travail** (étude). Une study permet de comparer des scénarios, d'organiser des analyses, et de lancer des exécutions batch.

### Base de données

```sql
CREATE TABLE ta_studies (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    status      TEXT DEFAULT 'draft',   -- draft | ready | archived
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE ta_study_linkers (
    study_id    TEXT REFERENCES ta_studies(id) ON DELETE CASCADE,
    linker_id   TEXT REFERENCES ta_linkers(id),
    label       TEXT,
    position    INT DEFAULT 0,
    PRIMARY KEY (study_id, linker_id)
);
```

### Backend (API REST)

| Méthode | Route                                      | Description                              |
|---------|--------------------------------------------|------------------------------------------|
| GET     | `/api/studies`                             | Liste                                    |
| POST    | `/api/studies`                             | Créer                                    |
| GET     | `/api/studies/:id`                         | Détail avec linkers                      |
| PUT     | `/api/studies/:id`                         | Mettre à jour                            |
| DELETE  | `/api/studies/:id`                         | Supprimer                                |
| POST    | `/api/studies/:id/linkers`                 | Ajouter un linker                        |
| DELETE  | `/api/studies/:id/linkers/:linker_id`      | Retirer un linker                        |

### Frontend

- **Page `/studies`** :
  - Liste des études avec statut
  - Création / édition : nom, description, ajout de linkers (glisser-déposer)
  - Vue détaillée : liste des linkers avec leur statut de validation
  - Option export/import JSON

---

## Module 8 — Execution

### Objectif

Exécuter une study : calculer les **matrices de FTP** pour chaque linker via la méthode de **maturity matching**.

**Méthode maturity matching** :
1. Pour chaque pas de temps `t` et chaque ligne du portfolio :
   - Récupérer l'`outstanding` à `t`
   - Récupérer le profil d'amortissement (vecteur de poids sur les M tenors) à `t`
   - Récupérer les taux du stack interpolés aux M tenors à `t`
   - `FTP(t)` = produit scalaire (profil d'amortissement × taux du stack) = taux pondéré par maturité
2. Agréger par outstanding pour obtenir les KPIs du portfolio à chaque pas de temps

**KPIs produits** :
- `weighted_ftp_rate` : taux FTP moyen pondéré par encours
- `total_outstanding` : encours total
- `ftp_interest_periodic` : intérêts FTP pour la période

### Base de données

```sql
CREATE TABLE ta_executions (
    id            TEXT PRIMARY KEY,
    study_id      TEXT REFERENCES ta_studies(id),
    label         TEXT,
    method        TEXT DEFAULT 'maturity_matching',
    status        TEXT DEFAULT 'pending',   -- pending | running | completed | error
    result_json   TEXT,
    -- {"linkers":[{"linker_id","time_steps":[{"date","kpis":{…},"ftp_matrix_json":"…"}]}]}
    duration_ms   BIGINT,
    error_message TEXT,
    created_at    TIMESTAMPTZ DEFAULT NOW()
);
```

### Backend (API REST)

| Méthode | Route                         | Description                         |
|---------|-------------------------------|-------------------------------------|
| GET     | `/api/executions`             | Liste                               |
| POST    | `/api/executions`             | Lancer (`{ study_id, label? }`)     |
| GET     | `/api/executions/:id`         | Résultat complet                    |
| DELETE  | `/api/executions/:id`         | Supprimer                           |

**Pipeline d'exécution** (lancé en tâche asynchrone) :

```
POST /api/executions { study_id }
  │
  ├─ Persister execution (status = "running")
  │
  ├─ Pour chaque linker de la study :
  │   ├─ Charger l'hypercube (stack_id, plages de dates, granularité)
  │   ├─ Pour chaque courbe du stack :
  │   │     → interpoler les taux aux M tenors mensuels pour chaque pas de temps
  │   │     → accumuler dans summed_rates[L × M] × weight
  │   │
  │   ├─ Charger les portfolio_rows (outstanding_json, schedule_json)
  │   │
  │   └─ Pour chaque pas de temps t :
  │         outstanding[t] = valeur dans outstanding_json
  │         profile[t]     = vecteur de poids M depuis schedule_json
  │         ftp_rate[t]    = dot(profile[t], summed_rates[t])
  │         → kpis : total_outstanding, weighted_ftp_rate, ftp_interest_periodic
  │
  └─ Persister result_json + status = "completed" | "error"
```

### Frontend

- **Page `/executions`** :
  - Liste des exécutions avec statut, durée, étude associée
  - Bouton « Lancer » (sélection de study)
  - Polling du statut en temps réel
  - Vue résultat : tableau des KPIs par linker × pas de temps

---

## Module 9 — Dashboard

### Objectif

Module **purement graphique** — lecture seule des résultats d'exécution. Aucune nouvelle table.

### Backend (API REST)

Réutilise `GET /api/executions/:id` (module 8). Pas de nouveaux endpoints.

### Frontend

- **Page `/dashboard`** :
  - Sélecteur d'exécution(s) à visualiser
  - **Visualisations** :
    - Évolution temporelle du `weighted_ftp_rate` (line chart par linker)
    - Évolution de l'`total_outstanding` (area chart)
    - Courbe FTP par tenor à une date donnée (line chart)
    - Heatmap FTP (temps × tenor)
    - Comparaison multi-exécutions (superposition)
    - Ventilation par type de risque (contribution de chaque courbe du stack)
  - **Indicateurs clés** : FTP moyen, écart-type, contribution par risque
  - Filtres interactifs (plage de dates, linkers, types de risque)
  - Export graphiques (PNG/SVG) et données sous-jacentes (CSV)

---

## État d'implémentation

| Module                        | DB  | Backend | Frontend | Moteur         |
|-------------------------------|-----|---------|----------|----------------|
| 1 — Séries temporelles        | ⚠️ refactor | ✅ partiel | ✅ partiel | n/a     |
| 2 — Courbes de taux           | ✅  | ✅      | ✅       | ✅ interp       |
| 3 — Curve Stacker             | ✅  | ✅      | ✅       | ✅ combinaisons |
| 4 — Hypercube                 | ✅  | ✅      | ✅       | ⚠️ proj N/A    |
| 5 — Portfolio                 | ✅  | ✅      | ✅       | n/a            |
| 6 — Linker                    | ✅  | ✅      | ✅       | n/a            |
| 7 — Studies                   | ✅  | ✅      | ✅       | n/a            |
| 8 — Execution                 | ✅  | ✅      | ✅       | ✅ base         |
| 9 — Dashboard                 | n/a | n/a     | ✅       | n/a            |

> **Refactor module 1** : migrer les tables `rate_series_data` existantes vers le schéma `ta_timeseries_*` avec hypertables TimescaleDB et ajout de la colonne `is_actual`.

---

## Ce qui n'est pas encore implémenté

- **Module 1** : gestion des projections groupées (`group_id`) — côté UI et API
- **Module 4** : calcul effectif des matrices pour la période de projection (`proj_end_date`) — la config est présente, la logique est absente
- **Module 4** : gestion des scénarios multiples (tenseur L × M × nb_proj) — non branché
- **Module 8** : Monte Carlo (choc parallèle ±σ par tenor, seed contrôlé, N scénarios)
- **Module 8** : Export `.bin` protobuf pour réduire la taille des résultats en base

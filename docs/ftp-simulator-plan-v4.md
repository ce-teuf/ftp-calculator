# FTP Simulator — Plan V4

---

## Sommaire

- [1. Logique métier](#1-logique-métier)
  - [Qu'est-ce que le FTP ?](#quest-ce-que-le-ftp-)
  - [Composantes du taux FTP](#composantes-du-taux-ftp)
  - [Buckets de maturité FTP](#buckets-de-maturité-ftp-12-tenors-standard)
  - [Ce que calcule le moteur](#ce-que-calcule-le-moteur)
  - [Méthodes de calcul disponibles](#méthodes-de-calcul-disponibles)
  - [Pipeline fonctionnel complet](#pipeline-fonctionnel-complet)
- [2. Base de données](#2-base-de-données)
  - [Organisation en schémas PostgreSQL](#organisation-en-schémas-postgresql)
  - [sc\_series](#sc_series)
  - [sc\_curves](#sc_curves)
  - [sc\_portfolios](#sc_portfolios)
  - [sc\_studies](#sc_studies)
- [3. Backend (Rust / Axum)](#3-backend-rust--axum)
  - [Stack technique](#stack-technique)
  - [API REST](#api-rest)
  - [Moteur de calcul](#moteur-de-calcul-executions_v3rs)
  - [Ce qui n'est pas encore implémenté](#ce-qui-nest-pas-encore-implémenté-côté-moteur)
- [4. Frontend (SvelteKit + Svelte 5)](#4-frontend-sveltekit--svelte-5)
  - [Stack technique](#stack-technique-1)
  - [Navigation](#navigation)
  - [Pages et composants](#pages-et-composants)
- [5. État d'implémentation et prochaines priorités](#5-état-dimplémentation-et-prochaines-priorités)
  - [Tableau de bord](#tableau-de-bord)
  - [Priorités](#priorités)

---

# 1. Logique métier

## Qu'est-ce que le FTP ?

Le **Funds Transfer Pricing (FTP)** est le mécanisme par lequel une banque alloue un coût de
refinancement interne à chaque poste de bilan. Pour un prêt (actif), la banque facture
implicitement une charge FTP à la ligne métier qui le porte : c'est le coût de "financer" ce
prêt sur la durée de son profil d'amortissement. Pour un dépôt (passif), elle lui alloue un
crédit FTP : c'est la valeur de ce dépôt comme source de financement stable.

**Ce simulateur se concentre uniquement sur la partie actif (prêts, portefeuilles de crédits).**

## Composantes du taux FTP

Le taux FTP appliqué à un prêt est une somme de composantes, chacune reflétant une dimension
du coût du capital :

| Composante            | Clé              | Description                                            |
|-----------------------|------------------|--------------------------------------------------------|
| Taux de base (OIS)    | `base_rate`      | Taux sans risque (ESTR, SOFR) — courbe de référence    |
| Spread de crédit      | `credit_spread`  | Z-spread senior unsecured de la banque                 |
| Prime de liquidité    | `tlp`            | Term Liquidity Premium — coût du financement à terme   |
| Liquidité contingente | `clp`            | Coussin de liquidité réglementaire (LCR/NSFR)          |
| Basis risk            | `basis_risk`     | XCCY basis (ex : EUR/USD) si la devise diffère         |
| Risque de prépaiement | `oas`            | Option-Adjusted Spread pour les prêts avec prépaiement |
| Capital charge        | `capital_charge` | Coût des fonds propres alloués (RWA × CoE)             |
| XVA                   | `xva`            | CVA / MVA / KVA selon les contreparties                |
| Risque opérationnel   | `operational`    | Charge opérationnelle allouée                          |
| Risque pays           | `country_risk`   | Spread souverain si exposition hors-zone               |
| Concentration         | `concentration`  | Add-on de concentration sectorielle / géographique     |
| MREL / Prélèvement    | `mrel_levy`      | Coût de la dette bail-inable (MREL) ou FDIC levy       |
| Prime d'incitation    | `incentive`      | Ajustement commercial (bonus/malus par produit)        |
| Risque de rollover    | `rollover`       | Coût de refinancement à l'échéance (cliff risk)        |

**Règle fondamentale :** `Taux FTP = Σ composantes` pondérées par le profil d'amortissement
du prêt sur les 12 buckets de maturité standard.

## Buckets de maturité FTP (12 tenors standard)

```
1M  3M  6M  1Y  2Y  3Y  5Y  7Y  10Y  15Y  20Y  30Y
```

Chaque prêt est décomposé sur ces 12 buckets selon son profil d'amortissement. Le taux FTP
final est la moyenne pondérée des taux de chaque bucket.

## Ce que calcule le moteur

Pour un portfolio à une date d'analyse donnée :

1. **Lire** l'encours (`outstanding`) de chaque ligne du portfolio à cette date
2. **Lire** le profil d'amortissement (`schedule`) de chaque ligne : vecteur de 12 % par bucket
3. **Lire** les taux du stack FTP interpolés aux 12 tenors standard
4. **Calculer** le taux FTP de chaque ligne = produit scalaire profil × taux par bucket
5. **Agréger** : taux FTP pondéré par encours, intérêts FTP mensuels totaux

## Méthodes de calcul disponibles

| Méthode       | Description                                                                 |
|---------------|-----------------------------------------------------------------------------|
| `stock`       | Méthode stock : taux figé à l'origination, appliqué sur l'encours résiduel  |
| `flux`        | Méthode flux : taux recalculé à chaque période sur les nouveaux flux        |
| `duration`    | Duration-weighted average : pondération par duration modifiée               |
| `pool`        | Pool de liquidité : taux moyen du pool de financement                       |
| `refinancing` | Coût de refinancement : taux de marché à la date de renouvellement          |
| `floating`    | Taux variable : réévaluation périodique selon l'index flottant              |

## Pipeline fonctionnel complet

```
Séries historiques (ESTR, SOFR, TLP, XCCY…)
        │
        ▼
   Curves Lab  ──────────────────────────────────────────────────────┐
   (Pyodide + Python)                                                 │
        │                                                             │
        ▼  (ou import CSV direct)                                     ▼
   rate_curves  ←── 1 courbe = 1 composante FTP, N tenors, 1 devise  │
        │                                                             │
        ▼                                                             │
   curve_stacks  ←── 1 stack = liste ordonnée de composantes ────────┘
        │              additivité : FTP = Σ composantes
        ▼
   curve_cubes  ←── 1 cube = stack × plage temporelle [× scénarios]
        │
        │◀──── portfolios_v3  ←── upload CSV (schedule + outstanding)
        │
        ▼
     linkers  ←── 1 linker = 1 portfolio + 1 cube + date départ
        │
        ▼
     studies  ←── N linkers groupés pour comparaison
        │
        ▼
  executions  ──▶  KPIs par linker × analysis_date
                   (weighted_ftp_rate, total_outstanding, ftp_int_monthly)
        │
        ▼
   Dashboard  ──▶  visualisation temporelle + comparaison
```

---

# 2. Base de données

## Organisation en schémas PostgreSQL

Toutes les tables sont dans des schémas préfixés `sc_`. Le schéma `public` contient uniquement
`_sqlx_migrations`. Le `search_path` est configuré à la connexion :
`public, sc_series, sc_curves, sc_portfolios, sc_studies`.

## sc_series

**`rate_series_data`** — séries temporelles de marché

| Colonne       | Type     | Description                                              |
|---------------|----------|----------------------------------------------------------|
| `id`          | TEXT PK  |                                                          |
| `series_name` | TEXT     | Ex : `SOFR`, `ESTR`, `EUR_TLP`, `XCCY_EUR_USD`           |
| `component`   | TEXT     | `base_rate` \| `credit_spread` \| `tlp` \| `ibor` \| `basis_risk` |
| `currency`    | TEXT     | `EUR`, `USD`…                                            |
| `obs_date`    | DATE     |                                                          |
| `tenor`       | TEXT?    | NULL pour séries scalaires (ex: SOFR ON), `5Y` pour term |
| `rate`        | FLOAT8   | Valeur décimale (0.04 = 4%)                              |

Index unique sur `(series_name, obs_date, COALESCE(tenor, ''))`.
Données chargées : ~317k observations, 2014–2026, 8 séries.

## sc_curves

**`rate_curves`** — courbes de taux construites ou importées

| Colonne        | Type    | Description                                           |
|----------------|---------|-------------------------------------------------------|
| `id`           | TEXT PK |                                                       |
| `name`         | TEXT    |                                                       |
| `component`    | TEXT    | Une des 14 composantes FTP                            |
| `currency`     | TEXT    |                                                       |
| `version`      | INT     | Incrément à chaque mise à jour                        |
| `status`       | TEXT    | `draft` \| `active` \| `archived`                    |
| `valid_from`   | DATE?   | Date d'effet                                          |
| `tenors_json`  | TEXT    | JSON array de labels : `["1M","3M","1Y",…]`           |
| `values_json`  | TEXT    | JSON array de f64 (décimal) : `[0.03, 0.032, 0.038,…]` |
| `series_name`  | TEXT?   | Série historique sous-jacente (lien optionnel)        |

**`curve_stacks`** — stacks de courbes

| Colonne     | Type    | Description                         |
|-------------|---------|-------------------------------------|
| `id`        | TEXT PK |                                     |
| `name`      | TEXT    |                                     |
| `status`    | TEXT    | `draft` \| `active` \| `archived`   |

**`curve_stack_components`** — composantes d'un stack

| Colonne         | Type   | Description                                       |
|-----------------|--------|---------------------------------------------------|
| `stack_id`      | TEXT FK → `curve_stacks`                          |
| `curve_id`      | TEXT FK → `rate_curves`                           |
| `position`      | INT    | Ordre dans le stack (0-based)                     |
| `label`         | TEXT   | Ex : "Base ESTR", "TLP 5Y"                        |
| `weight`        | REAL   | Multiplicateur (1.0 = 100%)                       |
| `interp_method` | TEXT   | `linear` \| `cubic` \| `flat_forward`             |

**`curve_cubes`** — cubes d'analyse

| Colonne           | Type    | Description                                        |
|-------------------|---------|----------------------------------------------------|
| `stack_id`        | TEXT FK → `curve_stacks`                            |
| `analysis_start`  | DATE    |                                                    |
| `analysis_end`    | DATE    |                                                    |
| `step_months`     | INT     | Pas entre chaque analysis time (défaut : 1)        |
| `include_proj`    | BOOL    | Activer la projection des séries                   |
| `proj_config_json`| TEXT?   | Config par série : méthode, scénarios, seed, params |
| `mc_scenarios`    | INT     | 0 = pas de Monte Carlo                             |

## sc_portfolios

**`portfolios_v3`**

| Colonne         | Type    | Description                                     |
|-----------------|---------|-------------------------------------------------|
| `schedule_type` | TEXT    | `stock_amort` \| `new_prod_amort`               |

**`portfolio_rows`** — une ligne = un segment de portfolio

| Colonne            | Type  | Description                                                  |
|--------------------|-------|--------------------------------------------------------------|
| `portfolio_id`     | FK    |                                                              |
| `label`            | TEXT? | Ex : "Crédits immobiliers 2022", "Prêts auto"                |
| `schedule_json`    | TEXT  | `[{"date":"2025-01","buckets":[0.02,0.05,…12 valeurs…]},…]`  |
| `outstanding_json` | TEXT  | `[{"date":"2025-01","outstanding":1500000000.0},…]`          |
| `row_order`        | INT   | Ordre d'affichage                                            |

Les 12 buckets du schedule correspondent aux 12 tenors FTP standard.
La valeur `1.0` est toujours prépendée au moment du calcul (encours total au bucket 0).

## sc_studies

**`linkers`** — association portfolio ↔ cube

| Colonne               | Type  | Description                                    |
|-----------------------|-------|------------------------------------------------|
| `portfolio_id`        | TEXT  |                                                |
| `cube_id`             | TEXT  |                                                |
| `start_date`          | DATE  | Date de démarrage de l'analyse                 |
| `fwd_schedule_json`   | TEXT? | Schedules forward si cube avec projection      |
| `fwd_outstanding_json`| TEXT? | Outstanding projeté si cube avec projection    |

**`studies`** — groupes de linkers

**`study_linkers`** — table d'association study ↔ linker (avec label et position)

**`executions_v3`** — résultats persistés

| Colonne        | Type  | Description                                              |
|----------------|-------|----------------------------------------------------------|
| `study_id`     | FK?   |                                                          |
| `method`       | TEXT  | stock \| flux \| duration \| pool \| refinancing \| floating |
| `status`       | TEXT  | `pending` \| `running` \| `completed` \| `error`        |
| `result_json`  | TEXT? | `{"linkers":[{"linker_id","analysis_times":[{"date","kpis":{…}}]}]}` |
| `duration_ms`  | INT8  |                                                          |

**`alco_approvals`** — traçabilité des validations ALCO

---

# 3. Backend (Rust / Axum)

## Stack technique

- **Framework** : Axum 0.8
- **DB** : sqlx 0.8 + PostgreSQL (pool de 10 connexions, search_path configuré à la connexion)
- **Migrations** : sqlx migrate (`src/db/migrations/001_init.sql`)
- **Crate métier** : `ftp-calculator-core` (workspace local)
- **Frontend embarqué** : `include_dir!` — le build SvelteKit est inclus dans le binaire

## API REST

### Rates

| Méthode | Route                     | Description                              |
|---------|---------------------------|------------------------------------------|
| GET     | `/api/rate-series/names`  | Liste des séries disponibles             |
| GET     | `/api/rate-series`        | Requête avec filtres (série, dates, tenor) |

### Curves

| Méthode | Route               | Description           |
|---------|---------------------|-----------------------|
| GET     | `/api/curves`       | Liste                 |
| POST    | `/api/curves`       | Créer                 |
| GET     | `/api/curves/:id`   | Détail                |
| PUT     | `/api/curves/:id`   | Mettre à jour         |
| DELETE  | `/api/curves/:id`   | Supprimer             |

### Stacks

| Méthode | Route                               | Description                    |
|---------|-------------------------------------|--------------------------------|
| GET     | `/api/stacks`                       | Liste                          |
| POST    | `/api/stacks`                       | Créer                          |
| GET     | `/api/stacks/:id`                   | Détail (avec composantes)      |
| PUT     | `/api/stacks/:id`                   | Mettre à jour                  |
| DELETE  | `/api/stacks/:id`                   | Supprimer                      |
| POST    | `/api/stacks/generate-combinations` | Générer le produit cartésien   |

### Cubes

| Méthode | Route             | Description |
|---------|-------------------|-------------|
| GET     | `/api/cubes`      | Liste       |
| POST    | `/api/cubes`      | Créer       |
| GET     | `/api/cubes/:id`  | Détail      |
| PUT     | `/api/cubes/:id`  | Mettre à jour |
| DELETE  | `/api/cubes/:id`  | Supprimer   |

### Portfolios V3

| Méthode | Route                                    | Description                      |
|---------|------------------------------------------|----------------------------------|
| GET     | `/api/portfolios-v3`                     | Liste                            |
| POST    | `/api/portfolios-v3`                     | Créer                            |
| GET     | `/api/portfolios-v3/:id`                 | Détail (avec lignes)             |
| DELETE  | `/api/portfolios-v3/:id`                 | Supprimer                        |
| POST    | `/api/portfolios-v3/:id/rows/upload`     | Upload CSV (schedule+outstanding)|
| GET     | `/api/portfolios-v3/:id/rows/:row_id`    | Détail d'une ligne               |
| DELETE  | `/api/portfolio-rows/:row_id`            | Supprimer une ligne              |

### Linkers, Studies

| Méthode | Route                                  | Description                      |
|---------|----------------------------------------|----------------------------------|
| GET/POST | `/api/linkers`                        |                                  |
| GET/DELETE | `/api/linkers/:id`                  |                                  |
| GET/POST | `/api/studies`                        |                                  |
| GET/PUT/DELETE | `/api/studies/:id`             |                                  |
| POST    | `/api/studies/:id/linkers`             | Ajouter un linker à une study    |
| DELETE  | `/api/studies/:id/linkers/:linker_id`  | Retirer un linker                |

### Exécutions

| Méthode | Route                      | Description                    |
|---------|----------------------------|--------------------------------|
| GET     | `/api/executions-v3`       | Liste                          |
| POST    | `/api/executions-v3`       | Lancer une exécution           |
| GET     | `/api/executions-v3/:id`   | Résultat complet               |
| DELETE  | `/api/executions-v3/:id`   | Supprimer                      |

## Moteur de calcul (executions_v3.rs)

```
POST /api/executions-v3  { study_id, label?, method? }
  │
  ├─ Persister execution (status = "running")
  │
  ├─ Pour chaque linker de la study :
  │   ├─ Charger portfolio_rows → (schedule_json, outstanding_json)
  │   ├─ Charger cube → stack_id, analysis_start, analysis_end, step_months
  │   ├─ Charger curve_stack_components → pour chaque composante :
  │   │     interpoler rate_curves aux 12 tenors FTP (linear|cubic|flat_forward)
  │   │     accumuler dans summed_rates[12] × weight
  │   │
  │   └─ Pour chaque analysis_date :
  │         outstanding[i] = nearest(outstanding_json, date)
  │         profile[i]     = [1.0] + nearest(schedule_json, date).buckets
  │         → ftp_calculator_core::FtpResult::compute(method)
  │         → kpis : total_outstanding, weighted_ftp_rate, ftp_int_monthly
  │
  └─ Persister result_json + status = "completed" | "error"
```

**Interpolation** (`compute/interpolate.rs`) : convertit les tenors d'une courbe (format
libre : `1M`, `2Y`, `30Y`…) vers les 12 buckets FTP standard. Méthodes : `linear`,
`cubic` (spline naturelle), `flat_forward`.

## Ce qui n'est pas encore implémenté côté moteur

- Projection des séries pour analysis times futurs (`include_proj = true`) — config présente,
  logique absente
- Monte Carlo (`mc_scenarios > 0`) — champ présent, non branché
- Export `.bin` protobuf (résultats actuellement en JSON)

---

# 4. Frontend (SvelteKit + Svelte 5)

## Stack technique

- **Framework** : SvelteKit 2 + Svelte 5 (runes : `$state`, `$derived.by`, `$effect`)
- **UI** : Tailwind CSS v4 + composants sur-mesure
- **Charts** : ECharts 5
- **Python Lab** : Pyodide (WASM) + CodeMirror 6
- **API client** : `src/lib/api/client.ts` — fetch vers `/api/*` (proxied vers `:3000` en dev)

## Navigation

```
⚡ FTP Simulator
├── 📊 Dashboard                  /dashboard
├── 〰️  Rates  ▾
│     ├── Time Series             /time-series
│     ├── Curves                  /curves
│     ├── Stacks                  /stacks
│     └── Cubes                   /cubes
├── 💼 Portfolios                 /portfolios
└── ▶️  Execution  ▾
      ├── Linkers                 /linkers
      ├── Studies                 /studies
      └── Executions              /executions
```

Rates et Execution sont des groupes collapsibles — ils s'ouvrent automatiquement si une
sous-route est active.

## Pages et composants

| Route           | Fichier                          | Statut | Description                                    |
|-----------------|----------------------------------|--------|------------------------------------------------|
| `/time-series`  | `routes/time-series/+page.svelte`| ✅     | Filtres série/date/tenor + 2 charts ECharts    |
| `/curves`       | `routes/curves/+page.svelte`     | ✅     | Library + builder 14 composantes + Python Lab  |
| `/stacks`       | `routes/stacks/+page.svelte`     | ✅     | Liste + builder + générateur de combinaisons   |
| `/cubes`        | `routes/cubes/+page.svelte`      | ✅     | Liste + builder + config projection/MC         |
| `/portfolios`   | `routes/portfolios/+page.svelte` | ✅     | Upload CSV + liste lignes + aperçu             |
| `/linkers`      | `routes/linkers/+page.svelte`    | ✅     | Sélecteur portfolio+cube + date départ         |
| `/studies`      | `routes/studies/+page.svelte`    | ✅     | Créer study, y attacher des linkers            |
| `/executions`   | `routes/executions/+page.svelte` | ✅     | Lancer + liste historique                      |
| `/dashboard`    | `routes/dashboard/+page.svelte`  | ✅     | Charger exécution + charts KPIs                |

**Composant Python Lab** (`lib/components/CurveLab.svelte`) : éditeur Python complet
(CodeMirror + Pyodide). Accès aux séries historiques. Persistance des courbes produites
dans `rate_curves`.

---

# 5. État d'implémentation et prochaines priorités

## Tableau de bord

| Périmètre                     | DB  | Backend | Frontend | Moteur  |
|-------------------------------|-----|---------|----------|---------|
| Time Series                   | ✅  | ✅      | ✅       | n/a     |
| Curves (Lab + Library)        | ✅  | ✅      | ✅       | ✅ interp |
| Stacks                        | ✅  | ✅      | ✅       | ✅ somme |
| Cubes                         | ✅  | ✅      | ✅       | ⚠️ proj N/A |
| Portfolios                    | ✅  | ✅      | ✅       | n/a     |
| Linkers                       | ✅  | ✅      | ✅       | n/a     |
| Studies                       | ✅  | ✅      | ✅       | n/a     |
| Exécutions (base)             | ✅  | ✅      | ✅       | ✅      |
| Dashboard                     | n/a | n/a     | ✅       | n/a     |
| Projection séries (cubes)     | ✅  | ⚠️ stub  | ⚠️ config| ❌      |
| Monte Carlo                   | ✅  | ⚠️ stub  | ⚠️ config| ❌      |

## Priorités

1. **Test end-to-end** — stack → cube → portfolio → linker → study → exécution → dashboard :
   vérifier que les KPIs sont cohérents avec des données connues
2. **Projection flat-forward** — pour les cubes `include_proj = true` : maintenir les taux
   du stack constants (flat) ou avec un drift linéaire paramétrable, pour chaque analysis time
3. **Monte Carlo simple** — choc parallèle ±σ par tenor sur le stack (seed contrôlé, N scénarios)
4. **Export .bin** — sérialisation protobuf des résultats pour réduire la taille en base

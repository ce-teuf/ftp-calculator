# FTP Simulator — Plan V3

## Principes directeurs

- L'app se concentre **uniquement** sur le calcul FTP pour un portfolio donné
- Pas de ventilation par vendeur / département / branche (version future)
- Pas d'analyse NIM/NII au niveau bilan
- Contrats, entités org, schedules de contrats → **supprimés**
- Les séries historiques (`rate_series_data`) restent — elles alimentent le Curves Lab

---

## Architecture des données

```
rate_series_data        ← séries historiques (ESTR, EURIBOR, etc.)
       │
       ▼
  rate_curves           ← courbes construites (Curves Lab ou manuelles)
       │
       ▼
  curve_stacks          ← N composantes, chacune pointant vers 1 courbe
  curve_stack_components
       │
       ▼
  curve_cubes           ← stack × dimension temporelle × scénarios
       │
  portfolios            ← vecteur outstanding + matrice schedules
  portfolio_rows        ← (1 ligne = 1 profil d'amort + vecteur outstanding)
       │
       ▼
  linkers               ← portfolio ↔ cube + paramètres
       │
  studies               ← N linkers + commentaires
  study_linkers
       │
       ▼
  executions            ← résultats .bin
```

---

## Module 1 — Curves Lab ✅

Pyodide + CodeMirror + séries historiques → courbes `rate_curves` persistées.
**Implémenté.** Accessible via Courbes → Python Lab.

---

## Module 2 — Curve Stack Builder

**Concept** : un Stack = liste ordonnée de composantes (1 à 14), chaque composante pointe vers exactement 1 courbe. Toutes les composantes sont **additives** (FTP = Σ composantes).

**Combinaisons plates** : si l'on sélectionne plusieurs courbes candidates par composante, le builder génère automatiquement le produit cartésien de toutes les combinaisons possibles, chaque combinaison devenant un stack plat nommé.

### DB

```sql
CREATE TABLE curve_stacks (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL,
  description TEXT,
  status     TEXT NOT NULL DEFAULT 'draft',  -- draft | approved | archived
  created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE curve_stack_components (
  id         TEXT PRIMARY KEY,
  stack_id   TEXT NOT NULL REFERENCES curve_stacks(id) ON DELETE CASCADE,
  position   INT  NOT NULL,   -- ordre (0-based)
  label      TEXT NOT NULL,   -- ex: "Taux base", "Spread liquidité"
  curve_id   TEXT NOT NULL REFERENCES rate_curves(id),
  weight     REAL NOT NULL DEFAULT 1.0
);
```

### API

- `GET  /api/stacks`
- `POST /api/stacks`
- `GET  /api/stacks/:id`
- `PUT  /api/stacks/:id`
- `DELETE /api/stacks/:id`
- `POST /api/stacks/generate-combinations` → body: `{ name, components: [{label, curve_ids[]}] }` → retourne N stacks créés

### UI (StacksTab.svelte)

- Vue **Library** : liste des stacks (badge status, N composantes, courbes liées)
- Vue **Builder** :
  - Formulaire : nom, description
  - Tableau des composantes : position | label | sélecteur de courbe(s)
  - Bouton **Générer les combinaisons** si plusieurs courbes par composante
  - Bouton **Sauvegarder** pour un stack simple

---

## Module 3 — Curve Cube Builder

**Concept** : un Cube = Stack × temps × [scénarios]

- **Analysis times** : liste de dates (début, fin, pas mensuel) → pour chaque date, le moteur s'engage dans le run-off
- **Projection des séries** : optionnel. Si activé, les courbes basées sur des séries sont reprojectées pour chaque analysis time via un script Pyodide (ou méthode simple intégrée)
- **Monte Carlo** : N scénarios sur les courbes de taux (ex: ±σ aléatoire par tenor)

### DB

```sql
CREATE TABLE curve_cubes (
  id              TEXT PRIMARY KEY,
  name            TEXT NOT NULL,
  stack_id        TEXT NOT NULL REFERENCES curve_stacks(id),
  analysis_start  DATE NOT NULL,
  analysis_end    DATE NOT NULL,
  step_months     INT  NOT NULL DEFAULT 1,
  include_proj    BOOL NOT NULL DEFAULT false,
  proj_script     TEXT,           -- script Pyodide optionnel
  mc_scenarios    INT  NOT NULL DEFAULT 0,  -- 0 = pas de MC
  status          TEXT NOT NULL DEFAULT 'draft',
  created_at      TIMESTAMPTZ DEFAULT now()
);
```

### UI (CubesTab.svelte)

- Liste des cubes + aperçu (stack lié, N dates, N scénarios)
- Builder : sélecteur de stack, date range, options MC

---

## Module 4 — Portfolio Definer

**Concept** : un Portfolio = N lignes, chaque ligne = 1 vecteur d'outstanding + 1 profil de schedule

- **Type de schedule** : `stock_amort` (amortissement du stock existant) ou `new_prod_amort` (amortissement de la nouvelle production)
- Le type désactive certaines méthodes kernel côté moteur

### Format CSV schedule

```
date,m1,m3,m6,m12,m24,m36,m60,m84,m120,m180,m240,m360
2025-01-01,0.02,0.05,0.08,0.12,...
2025-02-01,...
```
(profil d'amortissement : % restant à chaque bucket de maturité FTP, pour chaque date)

### Format CSV outstanding

```
date,outstanding
2025-01-01,1500000000
2025-02-01,1480000000
...
```

### DB

```sql
CREATE TABLE portfolios (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  schedule_type TEXT NOT NULL DEFAULT 'stock_amort',  -- stock_amort | new_prod_amort
  created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE portfolio_rows (
  id             TEXT PRIMARY KEY,
  portfolio_id   TEXT NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
  label          TEXT,
  schedule_json  TEXT NOT NULL,   -- matrice amort sérialisée
  outstanding_json TEXT NOT NULL, -- vecteur outstanding sérialisé
  row_order      INT  NOT NULL DEFAULT 0
);
```

### UI (PortfolioTab.svelte — réécrit)

- Liste des portfolios
- Upload CSV schedule + CSV outstanding → preview + validation
- Éditeur inline pour ajout de lignes
- Graphe d'amortissement + courbe d'outstanding

---

## Module 5 — Portfolio–Cube Linker

**Concept** : association entre 1 portfolio et 1 cube, avec paramètres d'exécution

### DB

```sql
CREATE TABLE linkers (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  portfolio_id TEXT NOT NULL REFERENCES portfolios(id),
  cube_id      TEXT NOT NULL REFERENCES curve_cubes(id),
  start_date   DATE NOT NULL,
  -- si cube avec projections :
  fwd_schedule_json   TEXT,   -- schedules forward (optionnel)
  fwd_outstanding_json TEXT,  -- outstanding projeté (optionnel)
  created_at   TIMESTAMPTZ DEFAULT now()
);
```

### UI (LinkerTab.svelte)

- Sélecteur portfolio + cube
- Date de départ
- Si cube avec projections → upload schedules forward

---

## Module 6 — Studies

**Concept** : regroupe N linkers pour une analyse comparative

### DB

```sql
CREATE TABLE studies (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  notes       TEXT,
  created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE study_linkers (
  study_id  TEXT NOT NULL REFERENCES studies(id) ON DELETE CASCADE,
  linker_id TEXT NOT NULL REFERENCES linkers(id),
  label     TEXT,   -- alias dans cette study
  PRIMARY KEY (study_id, linker_id)
);
```

### UI (StudiesTab.svelte)

- Créer une study, y ajouter des linkers
- Zone de notes libre (markdown)
- Vue comparative des résultats entre linkers

---

## Module 7 — Execution

- Sélectionner une study
- Lancer l'exécution (appel moteur FTP Rust)
- Logs en temps réel (SSE ou polling)
- Résultat sauvegardé : JSON (prototype → migration vers .bin protobuf plus tard)
- Liste des exécutions avec KPIs

---

## Module 8 — Dashboard

- Charger une exécution
- FTP rate par composante, par analysis time
- Courbes d'amortissement et d'outstanding
- Comparaison entre linkers d'une même study

---

## Navigation cible

```
⚡ FTP Simulator
├── 📈 Rate Series          (données historiques — lecture seule)
├── 〰️  Courbes             (library + builder manuel + Python Lab)
├── 🧱 Stacks               (curve stack builder)
├── 📦 Cubes                (curve cube builder)
├── 💼 Portfolios           (vecteurs + schedules)
├── 🔗 Linkers              (portfolio ↔ cube)
├── 📋 Studies              (regroupement + notes)
├── ▶️  Exécutions           (lancer + historique)
└── 📊 Dashboard            (visualisation résultats)
```

---

## Ordre d'implémentation

1. ✅ Curves Lab (fait)
2. 🔧 Navigation V3 (App.svelte) + migration DB (tables stacks, cubes, portfolios, linkers, studies)
3. 🔧 Curve Stack Builder (backend + UI)
4. 🔧 Portfolio Definer (backend + UI)
5. 🔧 Curve Cube Builder (backend + UI)
6. 🔧 Linker (backend + UI)
7. 🔧 Studies (backend + UI)
8. 🔧 Execution engine (adapter le moteur Rust)
9. 🔧 Dashboard

---

## Ce qui est supprimé

- Datasets (contrats, entités org, imports CSV contrats)
- Runoff models (remplacés par CSV schedule dans le portfolio)
- Analytics NIM/heatmap
- Pricer, Scenarios, NMD, Governance (stubs vides → à supprimer)

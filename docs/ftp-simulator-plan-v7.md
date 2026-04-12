# FTP Simulator — Plan V7

---

## Sommaire

- [Stack technique commun](#stack-technique-commun)
- [Architecture globale](#architecture-globale)
- [Module 1 — Matrices de taux](#module-1--matrices-de-taux)
- [Module 2 — Hypercube](#module-2--hypercube)
- [Module 3 — Portfolio](#module-3--portfolio)
- [Module 4 — Study unit builder](#module-4--study-unit-builder)
- [Module 5 — Studies](#module-5--studies)
- [Module 6 — Execution](#module-6--execution)
- [Module 7 — Dashboard](#module-7--dashboard)

---

## Stack technique commun

| Couche          | Choix                                                        |
|-----------------|--------------------------------------------------------------|
| Frontend        | Svelte 5 + SvelteKit 2 (runes), Tailwind CSS v4, ECharts 5  |
| Backend         | Rust + Axum 0.8, seul autorisé à lire/écrire la BDD         |
| Base de données | PostgreSQL 18                                                |
| Migrations      | sqlx migrate                                                 |

**Principe fondamental** : le frontend ne communique jamais directement avec la base de données. Tout passe par le service Rust.

**Interpolation** : les données brutes sont toujours stockées non interpolées. L'interpolation est effectuée à la volée lors de l'exécution.

---

## Architecture globale

```
Matrices de taux (Module 1)
        │
        ▼
Hypercube (Module 2)  ◄── sélection de matrices + dimension temporelle
        │                  génère les combinaisons sans doublon de risque
        │
        │◄──── Portfolio (Module 3)  ◄── vecteurs d'outstandings + schedules (indépendant)
        │
        ▼
Study unit builder (Module 4)  ◄── jonction Hypercube × Portfolio
        │                           assignment combinaison par paire (vector, schedule)
        │                           gestion du stock existant (initialisation FTP)
        ▼
Studies (Module 5)  ◄── regroupement logique de study units
        │
        ▼
Execution (Module 6)  ◄── maturity matching → matrices FTP + KPIs
        │
        ▼
Dashboard (Module 7)  ◄── visualisation read-only
```

---

## Module 1 — Matrices de taux

### Objectif

L'utilisateur uploade une ou plusieurs matrices de taux d'intérêt depuis un fichier `.ods`, `.xlsx` ou `.xlsm`.

**Format du fichier** : les colonnes sont fixes :
- Colonne A : `date_month` (format YYYY-MM)
- Colonne B : `period_type`
- Colonnes suivantes : valeurs par tenor non interpolé (ex. M1, M3, M6, M12…)

**Contrainte sur `period_type`** : une matrice ne peut contenir que deux valeurs simultanément, et uniquement dans cet ordre :
- Soit `observed` (premières lignes) puis `projected` (dernières lignes, optionnel)
- Soit `contrafactual` (premières lignes) puis `projected` (dernières lignes, optionnel)

**Types de risque** : à chaque matrice, l'utilisateur associe **un ou plusieurs** des 14 types de risque. Si plusieurs risques sont attribués à une même matrice, ils deviennent **indissociables** dans toutes les analyses ultérieures.

**Interpolation** : l'utilisateur sélectionne un algorithme d'interpolation par matrice pour obtenir tous les tenors mensuels à partir des tenors présents. Les données brutes (non interpolées) sont seules stockées ; l'interpolation est effectuée à la volée.

### Types de risque (14)

| Clé               | Description                                    |
|-------------------|------------------------------------------------|
| `base_rate`       | Taux sans risque (ESTR, SOFR)                  |
| `credit_spread`   | Z-spread senior unsecured                      |
| `tlp`             | Term Liquidity Premium                         |
| `clp`             | Coussin de liquidité réglementaire (LCR/NSFR)  |
| `basis_risk`      | XCCY basis                                     |
| `oas`             | Option-Adjusted Spread (prépaiement)           |
| `capital_charge`  | Coût des fonds propres (RWA × CoE)             |
| `xva`             | CVA / MVA / KVA                                |
| `operational`     | Charge opérationnelle                          |
| `country_risk`    | Spread souverain                               |
| `concentration`   | Add-on de concentration                        |
| `mrel_levy`       | Coût de la dette bail-inable / FDIC levy       |
| `incentive`       | Ajustement commercial                          |
| `rollover`        | Coût de refinancement à l'échéance             |

### Base de données

```sql
-- Table de référence des 14 types de risque (seed data, immuable)
CREATE TABLE risk_types (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT
);

-- Matrices de taux uploadées par l'utilisateur
CREATE TABLE rate_matrices (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    description   TEXT,
    currency      TEXT,
    status        TEXT DEFAULT 'draft',   -- draft | active | archived
    interp_method TEXT DEFAULT 'linear',  -- linear | cubic | flat_forward
    -- Tenors présents dans le fichier uploadé (non interpolés), ex. ["1M","3M","6M","12M"]
    tenors_json   JSONB NOT NULL,
    -- Données brutes: une ligne par période
    -- [{"date":"2024-01","period_type":"observed","values":[0.03,0.032,0.035,0.04]}, ...]
    rows_json     JSONB NOT NULL,
    created_at    TIMESTAMPTZ DEFAULT NOW()
);

-- Liaison many-to-many matrice ↔ types de risque
CREATE TABLE rate_matrix_risks (
    matrix_id TEXT REFERENCES rate_matrices(id) ON DELETE CASCADE,
    risk_key  TEXT REFERENCES risk_types(key),
    PRIMARY KEY (matrix_id, risk_key)
);
```

### Backend (API REST)

| Méthode | Route                      | Description                                          |
|---------|----------------------------|------------------------------------------------------|
| GET     | `/api/rate-matrices`       | Liste (filtres : status, currency, risk_key)         |
| POST    | `/api/rate-matrices`       | Créer via upload fichier                             |
| GET     | `/api/rate-matrices/:id`   | Détail + risques associés                            |
| PUT     | `/api/rate-matrices/:id`   | Mettre à jour (name, description, currency, status, interp_method, risks) |
| DELETE  | `/api/rate-matrices/:id`   | Supprimer                                            |
| GET     | `/api/risk-types`          | Liste des 14 types de risque                         |

### Frontend

- **Page `/rate-matrices`** :
  - Liste des matrices avec risques associés (badges couleur), statut, devise, dates couvertes
  - Upload fichier `.ods`/`.xlsx`/`.xlsm` : parsing côté backend, aperçu du contenu avant validation
  - Sélection des risques associés (avertissement si plusieurs → indissociables)
  - Sélection de la méthode d'interpolation
  - Vue détaillée : tableau des données brutes + graphique ECharts (courbe par tenor en fonction du temps)

---

## Module 2 — Hypercube

### Objectif

L'utilisateur crée un **hypercube** en deux étapes :

1. **Sélection des matrices** : il choisit un ensemble de matrices de taux. L'algorithme génère ensuite toutes les combinaisons valides de ces matrices, avec la contrainte fondamentale : **dans chaque combinaison, un même type de risque ne peut apparaître qu'une seule fois**. Deux matrices partageant au moins un type de risque commun ne peuvent pas coexister dans la même combinaison. Ces combinaisons représentent les scénarios possibles de construction du taux FTP.

2. **Dimension temporelle** : l'utilisateur définit la plage de dates couverte par le hypercube (`start_date`, `end_date`), une éventuelle période de projection (`proj_end_date`) et la granularité temporelle.

Un hypercube est une **configuration**, pas un résultat. Les matrices elles-mêmes (y compris l'interpolation) sont calculées à la volée lors de l'exécution.

### Base de données

```sql
CREATE TABLE hypercubes (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    description      TEXT,
    -- Dimension temporelle
    start_date       DATE NOT NULL,
    end_date         DATE NOT NULL,          -- fin de la période réalisée
    proj_end_date    DATE,                   -- fin de la période de projection (NULL = aucune)
    time_granularity TEXT DEFAULT 'monthly', -- daily | weekly | monthly
    status           TEXT DEFAULT 'draft',
    created_at       TIMESTAMPTZ DEFAULT NOW()
);

-- Matrices de taux incluses dans ce hypercube
CREATE TABLE hypercube_matrices (
    hypercube_id TEXT REFERENCES hypercubes(id) ON DELETE CASCADE,
    matrix_id    TEXT REFERENCES rate_matrices(id),
    PRIMARY KEY (hypercube_id, matrix_id)
);
```

Les combinaisons valides ne sont **pas stockées** : elles sont calculées à la volée à partir des matrices et de leurs risques associés.

### Backend (API REST)

| Méthode | Route                                  | Description                                                      |
|---------|----------------------------------------|------------------------------------------------------------------|
| GET     | `/api/hypercubes`                      | Liste                                                            |
| POST    | `/api/hypercubes`                      | Créer                                                            |
| GET     | `/api/hypercubes/:id`                  | Détail + matrices incluses + nombre de combinaisons valides      |
| PUT     | `/api/hypercubes/:id`                  | Mettre à jour                                                    |
| DELETE  | `/api/hypercubes/:id`                  | Supprimer                                                        |
| GET     | `/api/hypercubes/:id/combinations`     | Liste des combinaisons valides (calculées à la volée) avec leur couverture de risques |

### Frontend

- **Page `/hypercubes`** :
  - Liste des hypercubes avec nombre de matrices, nombre de combinaisons valides, plage de dates
  - Formulaire de création : sélection des matrices de taux, plage de dates, granularité, date fin projection
  - Visualisation des combinaisons générées : tableau avec colonnes = types de risque, lignes = combinaisons (cellule cochée si le risque est couvert)
  - Avertissement si des matrices sélectionnées ne couvrent pas la plage de dates définie

---

## Module 3 — Portfolio

### Objectif

Module **indépendant** des modules précédents. L'utilisateur gère des portefeuilles composés de deux types d'objets :

- **Vecteurs d'outstandings** : série temporelle des encours (une valeur par période)
- **Matrices de schedules d'amortissement** : profil d'amortissement par pas de temps (une ligne par période, avec une distribution de poids sur les buckets de tenor)

**Workflow d'upload** :
- À chaque upload d'un vecteur d'outstandings, l'utilisateur l'associe à un portfolio existant ou en crée un nouveau.
- À chaque upload d'une matrice de schedules, l'utilisateur l'associe à un ou plusieurs portfolios existants ou en crée de nouveaux.
- Au sein d'un portfolio, l'utilisateur peut ensuite lier un vecteur d'outstandings à une ou plusieurs matrices de schedules → ces liaisons forment des **paires (vector, schedule)**, unité de base du calcul FTP.

**Format des fichiers** : identique aux matrices de taux pour la colonne `period_type` (mêmes contraintes : `observed` ou `contrafactual` en premier, `projected` optionnel en dernier).

### Base de données

```sql
CREATE TABLE portfolios (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    description      TEXT,
    time_granularity TEXT DEFAULT 'monthly', -- daily | weekly | monthly
    created_at       TIMESTAMPTZ DEFAULT NOW()
);

-- Vecteurs d'outstandings (objets indépendants, associables à plusieurs portfolios)
CREATE TABLE outstanding_vectors (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    -- [{"date":"2024-01","period_type":"observed","value":1500000000.0}, ...]
    rows_json   JSONB NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

-- Matrices de schedules d'amortissement (objets indépendants, associables à plusieurs portfolios)
CREATE TABLE amort_schedules (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    description        TEXT,
    -- Labels des buckets de tenor, ex. ["1M","3M","6M","12M","24M","60M","120M"]
    bucket_labels_json JSONB NOT NULL,
    -- [{"date":"2024-01","period_type":"observed","buckets":[0.02,0.05,0.10,0.20,0.30,0.20,0.13]}, ...]
    rows_json          JSONB NOT NULL,
    created_at         TIMESTAMPTZ DEFAULT NOW()
);

-- Association many-to-many : portfolio ↔ vecteurs d'outstandings
CREATE TABLE portfolio_vectors (
    portfolio_id TEXT REFERENCES portfolios(id) ON DELETE CASCADE,
    vector_id    TEXT REFERENCES outstanding_vectors(id) ON DELETE CASCADE,
    PRIMARY KEY (portfolio_id, vector_id)
);

-- Association many-to-many : portfolio ↔ schedules
CREATE TABLE portfolio_schedules (
    portfolio_id TEXT REFERENCES portfolios(id) ON DELETE CASCADE,
    schedule_id  TEXT REFERENCES amort_schedules(id) ON DELETE CASCADE,
    PRIMARY KEY (portfolio_id, schedule_id)
);

-- Paires (vector, schedule) au sein d'un portfolio — unité de base du calcul FTP
CREATE TABLE portfolio_pairs (
    id           TEXT PRIMARY KEY,
    portfolio_id TEXT REFERENCES portfolios(id) ON DELETE CASCADE,
    vector_id    TEXT REFERENCES outstanding_vectors(id),
    schedule_id  TEXT REFERENCES amort_schedules(id),
    label        TEXT,
    UNIQUE (portfolio_id, vector_id, schedule_id)
);
```

### Backend (API REST)

| Méthode | Route                                            | Description                               |
|---------|--------------------------------------------------|-------------------------------------------|
| GET     | `/api/portfolios`                                | Liste                                     |
| POST    | `/api/portfolios`                                | Créer                                     |
| GET     | `/api/portfolios/:id`                            | Détail + vecteurs + schedules + paires    |
| PUT     | `/api/portfolios/:id`                            | Mettre à jour                             |
| DELETE  | `/api/portfolios/:id`                            | Supprimer                                 |
| POST    | `/api/outstanding-vectors`                       | Upload fichier → vecteur                  |
| GET     | `/api/outstanding-vectors/:id`                   | Détail                                    |
| PUT     | `/api/outstanding-vectors/:id`                   | Mettre à jour                             |
| DELETE  | `/api/outstanding-vectors/:id`                   | Supprimer                                 |
| POST    | `/api/amort-schedules`                           | Upload fichier → schedule                 |
| GET     | `/api/amort-schedules/:id`                       | Détail                                    |
| PUT     | `/api/amort-schedules/:id`                       | Mettre à jour                             |
| DELETE  | `/api/amort-schedules/:id`                       | Supprimer                                 |
| POST    | `/api/portfolios/:id/vectors`                    | Associer un vecteur à un portfolio        |
| DELETE  | `/api/portfolios/:id/vectors/:vector_id`         | Désassocier                               |
| POST    | `/api/portfolios/:id/schedules`                  | Associer un schedule à un portfolio       |
| DELETE  | `/api/portfolios/:id/schedules/:schedule_id`     | Désassocier                               |
| POST    | `/api/portfolios/:id/pairs`                      | Créer une paire (vector_id, schedule_id)  |
| DELETE  | `/api/portfolios/:id/pairs/:pair_id`             | Supprimer une paire                       |

### Frontend

- **Page `/portfolios`** :
  - Liste des portfolios avec granularité, nombre de paires
  - Vue détaillée d'un portfolio : tableau des vecteurs, tableau des schedules, liste des paires définies
  - Upload vecteur : parsing du fichier, aperçu, association portfolio(s)
  - Upload schedule : parsing du fichier, aperçu des buckets, association portfolio(s)
  - Gestion des paires : sélection glisser-déposer ou cases à cocher (vecteur × schedule)
  - Graphiques ECharts : évolution temporelle de l'outstanding, heatmap des schedules

---

## Module 4 — Study unit builder

### Objectif

Jonction entre un **hypercube** (Module 2) et un **portfolio** (Module 3). On construit une **study unit** en associant un portfolio à un hypercube.

Pour chaque **paire (vector, schedule)** existante dans le portfolio, l'utilisateur choisit une ou plusieurs **combinaisons** issues du hypercube. C'est à ce niveau qu'on précise quel sous-ensemble de matrices de taux s'applique à quels encours.

**Vérification des dimensions** : le module vérifie la compatibilité des dates et granularités entre hypercube et portfolio, et propose des règles de conversion si nécessaire. Il peut suggérer :
- De compléter les matrices de taux (périodes manquantes)
- De prolonger les vecteurs d'outstandings ou les schedules (réutiliser le dernier profil disponible)
- Le plus petit dénominateur commun en termes de dimensions

**Stock existant (initialisation FTP)** : pour chaque assignment (paire + combinaison), l'utilisateur peut indiquer si la **première période** correspond à un **stock nouveau** ou à un **stock existant**. Si c'est un stock existant, il peut charger un **profil FTP initial** : un vecteur de taux par bucket de tenor, représentant le taux FTP moyen pondéré du portefeuille existant à date d'initialisation. Ce profil remplace, pour t=0, le taux qui serait calculé à partir de la combinaison de matrices courantes, permettant de reconstituer la charge d'intérêt du back-book à son coût historique plutôt qu'au taux de marché courant.

### Base de données

```sql
CREATE TABLE study_units (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    description      TEXT,
    hypercube_id     TEXT REFERENCES hypercubes(id),
    portfolio_id     TEXT REFERENCES portfolios(id),
    start_date       DATE NOT NULL,
    granularity_rule TEXT DEFAULT 'none', -- none | aggregate | interpolate
    is_valid         BOOLEAN DEFAULT false,
    validation_log   TEXT,
    created_at       TIMESTAMPTZ DEFAULT NOW()
);

-- Pour chaque paire du portfolio: association à une combinaison du hypercube
-- Une même paire peut avoir plusieurs assignments (plusieurs combinaisons)
CREATE TABLE study_unit_assignments (
    id                       TEXT PRIMARY KEY,
    study_unit_id            TEXT REFERENCES study_units(id) ON DELETE CASCADE,
    pair_id                  TEXT REFERENCES portfolio_pairs(id),
    -- Combinaison = liste triée des IDs de matrices du hypercube
    -- ex. ["matrix_id_a", "matrix_id_b"]
    combination_matrix_ids   JSONB NOT NULL,
    label                    TEXT,
    -- Stock existant
    is_existing_stock        BOOLEAN DEFAULT false,
    -- Profil FTP initial pour t=0 (uniquement si is_existing_stock = true)
    -- Même format que les buckets d'un schedule: poids par tenor correspondant
    -- à la combinaison choisie.
    -- Ex. [{"tenor":"1M","rate":0.030},{"tenor":"3M","rate":0.032},...]
    initial_ftp_profile_json JSONB
);
```

### Backend (API REST)

| Méthode | Route                                           | Description                                               |
|---------|-------------------------------------------------|-----------------------------------------------------------|
| GET     | `/api/study-units`                              | Liste                                                     |
| POST    | `/api/study-units`                              | Créer (hypercube_id + portfolio_id + start_date)          |
| GET     | `/api/study-units/:id`                          | Détail + assignments                                      |
| PUT     | `/api/study-units/:id`                          | Mettre à jour                                             |
| DELETE  | `/api/study-units/:id`                          | Supprimer                                                 |
| POST    | `/api/study-units/:id/validate`                 | Vérifier compatibilité dimensions + dates                 |
| POST    | `/api/study-units/:id/assignments`              | Ajouter un assignment (pair_id + combination)             |
| PUT     | `/api/study-units/:id/assignments/:aid`         | Modifier un assignment                                    |
| DELETE  | `/api/study-units/:id/assignments/:aid`         | Supprimer un assignment                                   |

**Validation** (endpoint `validate`) :
1. La granularité du hypercube et du portfolio sont-elles compatibles (ou convertibles) ?
2. Les dimensions L correspondent-elles (nombre de pas de temps sur la plage `start_date` → `end_date`) ?
3. Les vecteurs d'outstandings couvrent-ils la plage temporelle du hypercube ?
4. Chaque combinaison assignée appartient-elle bien aux matrices du hypercube ?
5. Alertes et suggestions de correction

### Frontend

- **Page `/study-units`** :
  - Liste des study units avec statut de validation (vert/orange/rouge), hypercube et portfolio associés
  - Formulaire de création : sélection hypercube + portfolio + date de départ + règle de conversion
  - Vue détaillée : pour chaque paire du portfolio, interface d'assignment des combinaisons
    - Sélecteur de combinaison (liste déroulante avec les combinaisons valides du hypercube + leur couverture de risques)
    - Toggle « Stock existant » → si activé, formulaire de saisie du profil FTP initial (une ligne de taux par tenor)
  - Bouton « Valider » → rapport détaillé (dimensions, dates, alertes, suggestions)

---

## Module 5 — Studies

### Objectif

Regroupement logique de study units. Une study est une unité d'exécution : elle rassemble plusieurs study units qui seront calculées ensemble.

### Base de données

```sql
CREATE TABLE studies (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    status      TEXT DEFAULT 'draft', -- draft | ready | archived
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE study_items (
    study_id      TEXT REFERENCES studies(id) ON DELETE CASCADE,
    study_unit_id TEXT REFERENCES study_units(id),
    label         TEXT,
    position      INT DEFAULT 0,
    PRIMARY KEY (study_id, study_unit_id)
);
```

### Backend (API REST)

| Méthode | Route                                       | Description                       |
|---------|---------------------------------------------|-----------------------------------|
| GET     | `/api/studies`                              | Liste                             |
| POST    | `/api/studies`                              | Créer                             |
| GET     | `/api/studies/:id`                          | Détail + study units              |
| PUT     | `/api/studies/:id`                          | Mettre à jour                     |
| DELETE  | `/api/studies/:id`                          | Supprimer                         |
| POST    | `/api/studies/:id/units`                    | Ajouter une study unit            |
| DELETE  | `/api/studies/:id/units/:unit_id`           | Retirer une study unit            |

### Frontend

- **Page `/studies`** :
  - Liste des études avec statut, nombre de study units
  - Création / édition : nom, description, ajout de study units (glisser-déposer)
  - Vue détaillée : liste des study units avec leur statut de validation
  - Option export / import JSON

---

## Module 6 — Execution

### Objectif

Exécuter une study : calculer les **matrices de FTP** pour chaque assignment de chaque study unit via la méthode de **maturity matching**.

**Méthode maturity matching** :

Pour chaque study unit → pour chaque assignment (paire + combinaison) → pour chaque pas de temps `t` :

1. Récupérer `outstanding[t]` depuis le vecteur d'outstandings
2. Récupérer `profile[t]` depuis la matrice de schedules (vecteur de poids sur les M buckets de tenor)
3. Récupérer les taux interpolés de la combinaison aux M tenors de la matrice de schedules à la date `t`
   - Si `t == t₀` et `is_existing_stock == true` : utiliser `initial_ftp_profile` au lieu des taux interpolés
4. `ftp_rate[t]` = produit scalaire (`profile[t]` × `taux_interpolés[t]`)
5. KPIs :
   - `weighted_ftp_rate` : taux FTP moyen pondéré
   - `total_outstanding` : encours total
   - `ftp_interest_periodic` : intérêts FTP pour la période = `outstanding[t]` × `ftp_rate[t]` / fréquence

**Structure du résultat** :

```json
{
  "study_units": [
    {
      "study_unit_id": "...",
      "assignments": [
        {
          "assignment_id": "...",
          "pair_label": "...",
          "combination": ["matrix_id_a", "matrix_id_b"],
          "time_steps": [
            {
              "date": "2024-01",
              "kpis": {
                "total_outstanding": 1500000000.0,
                "weighted_ftp_rate": 0.0312,
                "ftp_interest_periodic": 390000.0
              },
              "ftp_by_tenor": {"1M": 0.030, "3M": 0.032, "6M": 0.035, "12M": 0.038}
            }
          ]
        }
      ]
    }
  ]
}
```

### Base de données

```sql
CREATE TABLE executions (
    id            TEXT PRIMARY KEY,
    study_id      TEXT REFERENCES studies(id),
    label         TEXT,
    method        TEXT DEFAULT 'maturity_matching',
    status        TEXT DEFAULT 'pending', -- pending | running | completed | error
    result_json   JSONB,
    duration_ms   BIGINT,
    error_message TEXT,
    created_at    TIMESTAMPTZ DEFAULT NOW()
);
```

### Backend (API REST)

| Méthode | Route                    | Description                              |
|---------|--------------------------|------------------------------------------|
| GET     | `/api/executions`        | Liste                                    |
| POST    | `/api/executions`        | Lancer `{ study_id, label? }`            |
| GET     | `/api/executions/:id`    | Résultat complet                         |
| DELETE  | `/api/executions/:id`    | Supprimer                                |

**Pipeline d'exécution** (tâche asynchrone) :

```
POST /api/executions { study_id }
  │
  ├─ Persister execution (status = "running")
  │
  ├─ Pour chaque study unit de la study :
  │   ├─ Charger le hypercube (matrices sélectionnées, plage de dates, granularité)
  │   ├─ Charger le portfolio (vecteurs + schedules + paires)
  │   │
  │   └─ Pour chaque assignment (paire + combinaison) :
  │       ├─ Identifier les matrices de la combinaison
  │       ├─ Pour chaque pas de temps t :
  │       │   ├─ outstanding[t]          ← vecteur d'outstandings
  │       │   ├─ profile[t]              ← matrice de schedules (buckets)
  │       │   ├─ rates[t]                ← interpolation à la volée des matrices de la combinaison
  │       │   │                            (ou initial_ftp_profile si t=t₀ et stock existant)
  │       │   └─ ftp_rate[t]             = dot(profile[t], rates[t])
  │       └─ Calculer les KPIs
  │
  └─ Persister result_json + status = "completed" | "error"
```

### Frontend

- **Page `/executions`** :
  - Liste des exécutions avec statut, durée, étude associée
  - Bouton « Lancer » avec sélection de la study
  - Polling du statut en temps réel
  - Vue résultat : tableau des KPIs par assignment × pas de temps
  - Lien vers le Dashboard pour la visualisation

---

## Module 7 — Dashboard

### Objectif

Module **purement graphique** — lecture seule des résultats d'exécution. Aucune nouvelle table.

### Backend

Réutilise `GET /api/executions/:id`. Pas de nouveaux endpoints.

### Frontend

- **Page `/dashboard`** :
  - Sélecteur d'exécution(s) à visualiser
  - **Visualisations** :
    - Évolution temporelle du `weighted_ftp_rate` (line chart par assignment)
    - Évolution du `total_outstanding` (area chart)
    - Courbe FTP par tenor à une date donnée (line chart)
    - Heatmap FTP (temps × tenor)
    - Comparaison multi-exécutions (superposition)
    - Ventilation par type de risque (contribution de chaque matrice de la combinaison)
  - **Indicateurs clés** : FTP moyen, écart-type, contribution par risque
  - Filtres interactifs (plage de dates, assignments, types de risque)
  - Export graphiques (PNG/SVG) et données sous-jacentes (CSV)

---

## État d'implémentation

| Module                   | DB  | Backend | Frontend | Moteur |
|--------------------------|-----|---------|----------|--------|
| 1 — Matrices de taux     | ⬜  | ⬜      | ⬜       | ⬜     |
| 2 — Hypercube            | ⬜  | ⬜      | ⬜       | ⬜     |
| 3 — Portfolio            | ⬜  | ⬜      | ⬜       | ⬜     |
| 4 — Study unit builder   | ⬜  | ⬜      | ⬜       | ⬜     |
| 5 — Studies              | ⬜  | ⬜      | ⬜       | ⬜     |
| 6 — Execution            | ⬜  | ⬜      | ⬜       | ⬜     |
| 7 — Dashboard            | n/a | n/a     | ⬜       | n/a    |

---

## Points ouverts

- **Stock existant (Module 4)** : la spec du `initial_ftp_profile_json` est à préciser. En particulier : est-il défini une seule fois (pour t=0) ou peut-il être une série temporelle (pour couvrir un back-book à amortissement progressif) ? Voir `ftp-core-test.xlsm` cellule J28 feuille1 pour la logique de référence.
- **Scénarios multiples (Module 2)** : un hypercube peut produire plusieurs combinaisons ; lors de l'exécution, chaque assignment traite une combinaison à la fois. La gestion d'un run multi-combinaisons simultanées (sensibilité) est à spécifier.
- **Export résultats** : format `.bin` protobuf pour réduire la taille du `result_json` en base si les données deviennent volumineuses.

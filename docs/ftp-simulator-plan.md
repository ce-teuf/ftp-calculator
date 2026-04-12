# FTP Simulator — Plan Complet

*Un outil de simulation et monitoring FTP complet, déployable en local sur n'importe quel PC via un installeur autonome.*

---

## Vision

Un dashboard FTP interactif, **installable en double-cliquant sur un `.exe`**, qui couvre toutes les méthodes de calcul FTP, visualise la décomposition NIM par branche/produit/vendeur, persiste toutes les données en PostgreSQL 18 local, et exporte vers Excel. Zéro configuration technique requise de l'utilisateur.

```
┌─────────────────────────────────────────────────────────────┐
│                  PC LOCAL DE L'UTILISATEUR                  │
│                                                             │
│   🌐 Navigateur  →  http://localhost:3000                   │
│                           │                                 │
│   ┌───────────────────────▼─────────────────────────────┐  │
│   │  ftp-backend.exe   (Rust + Axum)                    │  │
│   │  • Sert le frontend Svelte 5 (statique embarqué)    │  │
│   │  • API REST CRUD                                    │  │
│   │  • Lance ftp-calculator-core (natif, pas WASM)      │  │
│   └───────────────────────┬─────────────────────────────┘  │
│                           │ localhost:5432                  │
│   ┌───────────────────────▼─────────────────────────────┐  │
│   │  postgres.exe   (PostgreSQL 18 portable)            │  │
│   │  • Données dans C:\ProgramData\FtpSimulator\data\   │  │
│   └─────────────────────────────────────────────────────┘  │
│                                                             │
│   🔲 ftp-tray.exe  (icône system tray — contrôleur)         │
└─────────────────────────────────────────────────────────────┘
```

**Aucun Docker. Aucune installation manuelle. Un `.exe` suffit.**

---

## Idées de Fonctionnalités

### Module 1 — Méthodes de Calcul FTP (Exhaustivité Totale)

Implémenter toutes les méthodes théoriques connues, sélectionnables par produit :

| Méthode | Statut actuel | À implémenter |
|---------|--------------|--------------|
| Weighted-Average (Stock) | ✅ Core | Exposer via WASM |
| Flux (multi-vintage) | ✅ Core | Exposer via WASM |
| Refinancing / Forward Rate | ✅ (via input_rate) | UI de construction des forward rates |
| Duration Method | ❌ | Calcul de modified duration → lecture courbe |
| Pool Method (single pool) | ❌ | Taux unique blended pour tous les actifs |
| Multiple Pool Method | ❌ | Pools par bucket de maturité |
| Pool with Maturity Ladder | ❌ | Grille maturité × produit |
| Replicating Portfolio (NMDs) | ❌ | Optimiseur de portefeuille répliquant |
| Behavioral Run-off (NMDs) | ❌ | Modèle de décroissance exponentielle |
| LDI (pension / insurance) | ❌ | Horizon intergénérationnel, SCR |
| Floating-Rate (deux profils) | ❌ | Double passage kernel : profil taux + liq |

Chaque méthode produit les mêmes outputs (stock_amort, ftp_rate, ftp_int, market_rate) et alimente le même dashboard de résultats.

---

### Module 2 — CoF Curve Builder (simplifié)

Un constructeur de courbe interactif qui permet de définir `input_rate[i,j]` composante par composante :

- **Courbe de base (SOFR OIS)** : saisie manuelle ou import CSV de la courbe du jour
- **14 composantes** : champs numériques par tenor (1M → 30Y), empilés visuellement
- **Visualisation de la courbe finale** : graphique interactif montrant la décomposition en bandes colorées par composante
- **Modes** :
  - *Spot rates* → weighted-average method
  - *Forward rates* (calculés automatiquement depuis les spots) → refinancing method
  - *Duration-equivalent* → duration method
- **Sauvegarde / rappel de courbes** : localStorage, export JSON

---

### Module 3 — Chargement et Gestion des Données

#### Import
- **CSV drag & drop** : glisser-déposer un fichier contenant le portefeuille
- **Excel import** : lecture de fichiers .xlsx via SheetJS
- **Template téléchargeable** : un fichier Excel pré-formaté avec les colonnes attendues
- **Saisie manuelle** : tableau éditable dans l'interface pour tester des cas simples
- **Données de démonstration** : jeux de données pré-chargés (portefeuille retail, corporate, mixte)

#### Format de données attendu
```
colonnes : product_id | product_type | branch | seller | outstanding | profile_* | rate_* | currency | ...
```

#### Validation
- Vérification des dimensions (outstanding 1 col, profiles et rates cohérents)
- Détection des valeurs manquantes
- Rapport d'erreurs ligne par ligne

---

### Module 4 — Dashboard Monitoring

#### 4.1 Vue Portefeuille Global
- **KPIs synthétiques** : NIM total, FTP total, marge nette agrégée, RAROC moyen pondéré
- **Évolution temporelle** : ligne du temps des métriques (si données multi-périodes)
- **Répartition du bilan** : treemap actifs × passifs coloré par marge FTP

#### 4.2 Décomposition NIM (la vue centrale)
Décomposition du NIM global en 3 axes simultanés :
```
NIM total = Marge BU prêts (Asset NIM)
          + Marge BU dépôts (Liability NIM)
          + Contribution Structurelle ALM (Treasury)
```
Graphique waterfall interactif montrant comment chaque composante contribue au NIM final.

#### 4.3 Décomposition du Taux FTP par Composante
Pour chaque produit / ligne, un graphique à barres empilées montrant :
- Base Rate
- Credit Spread
- TLP
- CLP
- OAS
- Capital Charge
- Basis Risk
- Operational Risk
- Coupon client
- Marge résiduelle (en surbrillance verte ou rouge selon rentabilité)

#### 4.4 Heatmaps Multi-Dimensionnelles
- **Branche × Produit** : marge FTP moyenne en dégradé de couleur
- **Vendeur × Produit** : RAROC par commercial
- **Tenor × Méthode** : comparaison des taux FTP selon la méthode choisie
- **Vintage × Période** : performance des cohortes d'origination dans le temps

#### 4.5 Vue Trésorerie (Treasury P&L)
- Contribution structurelle (FTP chargé aux actifs − FTP crédité aux passifs)
- HQLA carry drag (estimation à partir des paramètres CLP)
- P&L de couverture (si des taux de hedges sont saisis)
- Comparaison Treasury P&L vs BU P&L

#### 4.6 Vue Vendeur / Commercial
- Classement des commerciaux par marge FTP générée
- Volume originé vs marge nette (scatter plot)
- Alerte si un commercial originé systématiquement sous le taux FTP

#### 4.7 Vue Branche / Agence
- Carte géographique des marges par agence (si coordonnées disponibles)
- Tableau de bord de performance comparative entre branches
- Drill-down : cliquer sur une branche → voir ses produits

---

### Module 5 — RAROC et Capital

- Saisie des paramètres Basel (risk weight par type de produit, capital ratio, CoE)
- Calcul du RAROC par ligne, produit, branche, commercial
- **Traffic light** : vert (RAROC ≥ hurdle), orange (RAROC entre 0 et hurdle), rouge (RAROC négatif)
- Seuil de hurdle rate configurable
- Analyse "what-if" : que se passe-t-il si le CoE passe de 12% à 15% ?

---

### Module 6 — Analyse de Scénarios

Un des modules les plus puissants — simuler l'impact de chocs de marché sur l'ensemble du portefeuille FTP :

#### Scénarios de taux
- Choc parallèle : +100 bps, +200 bps, +300 bps, −100 bps sur toute la courbe
- Pentification : court terme +200 bps, long terme +50 bps
- Aplatissement : court terme stable, long terme +200 bps
- Inversion : court terme +300 bps, long terme −50 bps
- Scénarios BCBS (6 scénarios standard IRRBB)

#### Impact calculé
Pour chaque scénario, recalculer :
- Taux FTP (via le kernel WASM avec les nouvelles courbes)
- NIM par produit (marges qui changent selon la méthode : fixed → pas de changement car locké ; floating → recalcul)
- P&L Treasury (impact sur la contribution structurelle)
- RAROC (si le capital charge change)

#### Visualisation
- Graphique "tornado" : impact de chaque scénario sur le NIM total
- Comparaison avant/après pour chaque ligne du portefeuille
- Identification des produits les plus sensibles

---

### Module 7 — Simulateur de Nouvelle Origination ("Pricer")

Un outil de pricing en temps réel pour aider un commercial à pricer un nouveau prêt :

1. **Saisir les caractéristiques** : montant, durée, type, devise, profil d'amortissement, option de prépaiement
2. **Choisir les paramètres de risque** : rating contrepartie, secteur, garanties (→ risk weight)
3. **Le dashboard calcule instantanément** via WASM :
   - Taux FTP all-in (toutes composantes)
   - Marge minimale pour atteindre le hurdle RAROC
   - Prix plancher (FTP + EL + OpEx)
   - Sensibilité à +/- 50 bps sur la courbe
4. **Comparer plusieurs structures** : bullet vs amortissant vs revolving, même client
5. **Exporter la fiche de pricing** en PDF ou Excel

---

### Module 8 — Modèle NMD (Non-Maturity Deposits)

Outil dédié à la modélisation comportementale des dépôts à vue :

- **Saisie des données historiques** : volumes mensuels sur 3-10 ans
- **Calibration automatique** du taux de décroissance λ (modèle exponentiel)
- **Segmentation** : stable core vs volatile non-core (seuil paramétrable)
- **Génération du profil** : sortie en matrice `profiles` compatible kernel
- **Contrainte EBA** : WAL automatiquement plafonnée à 60 mois, alerte si dépassée
- **Optimiseur replicating portfolio** : trouver les poids optimaux (1Y, 3Y, 5Y) qui maximisent le Sharpe Ratio de la marge de dépôt
- **Visualisation** : histogramme des taux FTP crédités selon les différents modèles

---

### Module 9 — Analyse Vintage (Cohortes)

Pour les portefeuilles multi-périodes (méthode Flux) :

- **Matrice vintage** : origination par période × performance dans le temps
- **Courbes de survie** : quelle fraction de chaque cohorte reste en vie à chaque période
- **Contribution par vintage** : qui parmi les cohortes passées génère encore le plus de FTP int ?
- **Drift de marge** : la marge d'une cohorte 2020 comparée à une cohorte 2024 — quel est l'impact de la remontée des taux ?
- **Burn-out** : visualisation de l'effet burn-out sur les prépaiements (pour les méthodes OAS)

---

### Module 10 — Export et Reporting

#### Export Excel (SheetJS)
Un fichier Excel multi-onglets généré à la demande :
- Onglet "Portefeuille" : données d'entrée
- Onglet "FTP Rates" : taux FTP par produit, décomposé par composante
- Onglet "Stock & Installments" : matrices stock_amort, stock_instal, varstock
- Onglet "NIM Attribution" : décomposition NIM par BU/branche/vendeur
- Onglet "RAROC" : capital charge et RAROC par ligne
- Onglet "Scenarios" : résultats des scénarios de stress
- Onglet "Courbe CoF" : la courbe de taux utilisée, composante par composante

#### Export CSV
- Toutes les matrices de sortie exportables en CSV plat

#### Export PDF
- Rapport de synthèse une page : KPIs + graphiques principaux + date + paramètres utilisés

#### Partage / Persistance
- URL shareable : paramètres encodés en base64 dans l'URL (portefeuilles légers)
- localStorage : sauvegarde automatique de la dernière session
- Export/Import JSON : état complet de la session (courbe + portefeuille + paramètres)

---

### Module 11 — Gouvernance et Audit Trail

- **Historique des courbes FTP** : chaque courbe sauvegardée avec horodatage et auteur
- **Journal des calculs** : qui a lancé quel calcul avec quels paramètres
- **Comparateur de courbes** : afficher deux courbes côte à côte (ex : courbe ALCO vs courbe proposée)
- **Incentive Premium overlay** : ajouter un IP positif ou négatif par produit, documenté avec rationale
- **Validation ALCO** : workflow de validation : draft → soumis → approuvé → actif

---

### Module 12 — Idées Avancées

#### Multi-devises
- Saisie de la courbe SOFR + courbe €STR + basis EUR/USD
- Calcul automatique du coût XCCY pour les actifs cross-currency
- Heatmap devise × produit

#### Simulation Monte Carlo des taux
- Générer N chemins de taux via Hull-White (paramètres calibrables)
- Calculer la distribution du NIM sous incertitude
- Afficher l'intervalle de confiance [P10, P50, P90] sur le NIM

#### Comparateur de méthodes
- Appliquer toutes les méthodes sur le même portefeuille simultanément
- Tableau comparatif : quelle méthode donne quel taux FTP pour chaque produit ?
- Identifier les produits où le choix de méthode est le plus impactant

#### Benchmark sectoriel (données simulées)
- Comparer les marges FTP du portefeuille chargé à des benchmarks sectoriels simulés
- "Votre TLP moyen est 35 bps vs un benchmark retail de 28 bps"

#### Mode Pédagogique
- Pour chaque calcul, affichage des formules utilisées pas à pas
- Annotations sur les graphiques expliquant ce que représente chaque barre/point
- Mode "tutoriel" guidant un nouvel utilisateur à travers un cas complet

---

## Base de Données : Principe d'Inputs-Only et Déterminisme

### Philosophie fondamentale

> **On ne stocke que les inputs. Les outputs sont toujours recalculés.**

Les outputs (ftp_rate, stock_amort, ftp_int…) sont une **fonction pure** des inputs. Les stocker créerait deux sources de vérité susceptibles de diverger. Stocker uniquement les inputs garantit que la base de données est la seule source de vérité, et que toute simulation est 100% reproductible en relançant le calcul avec les mêmes entrées.

Pour les simulations stochastiques (Monte Carlo de taux, modèles de prépaiement aléatoires), le **seed** est la seule donnée supplémentaire à conserver — il transforme l'aléatoire en déterministe.

---

### Schéma de la Base de Données

#### Table `rate_curves` — Courbes de taux (toutes composantes)

```sql
CREATE TABLE rate_curves (
    id          TEXT PRIMARY KEY,          -- UUID
    name        TEXT NOT NULL,             -- "SOFR OIS 2025-01-15", "TLP FHLB Q1-2025"
    component   TEXT NOT NULL,             -- 'base_rate' | 'tlp' | 'credit_spread' | 'clp' | ...
    currency    TEXT NOT NULL DEFAULT 'USD',
    version     INTEGER NOT NULL DEFAULT 1,
    status      TEXT NOT NULL DEFAULT 'draft', -- 'draft' | 'approved' | 'archived'
    valid_from  DATE,
    valid_to    DATE,
    tenors_json TEXT NOT NULL,             -- '["1M","3M","6M","1Y","2Y","3Y","5Y","7Y","10Y"]'
    values_json TEXT NOT NULL,             -- '[4.33, 4.30, 4.22, 4.10, 3.95, 3.90, 3.85, 3.88, 3.90]'
    source      TEXT,                      -- 'Bloomberg', 'FHLB SF', 'Manual', 'AXI'
    notes       TEXT,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by  TEXT
);
```

Jamais d'`UPDATE` sur une courbe approuvée — uniquement un nouvel `INSERT` avec `version + 1`.

#### Table `rate_series` — Séries temporelles de taux de marché

```sql
CREATE TABLE rate_series (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,             -- "SOFR historique 2020-2025", "Fed Funds"
    component   TEXT NOT NULL,
    frequency   TEXT NOT NULL,             -- 'daily' | 'weekly' | 'monthly'
    dates_json  TEXT NOT NULL,             -- '["2025-01-01", "2025-01-02", ...]'
    values_json TEXT NOT NULL,             -- '[4.30, 4.31, ...]' (même tenor pour chaque date)
    tenor       TEXT,                      -- '3M' — si série mono-tenor
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### Table `runoff_models` — Profils de remboursement par produit

```sql
CREATE TABLE runoff_models (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,         -- "Prêt immo linéaire 20Y", "Dépôt vue retail core"
    product_type    TEXT NOT NULL,         -- 'mortgage' | 'corporate_loan' | 'nmd' | 'revolver'
    category        TEXT,                  -- 'retail' | 'corporate' | 'sme'
    version         INTEGER NOT NULL DEFAULT 1,
    status          TEXT NOT NULL DEFAULT 'draft',
    method          TEXT NOT NULL,         -- 'contractual' | 'behavioral_exponential' | 'replicating'
    profile_json    TEXT NOT NULL,         -- '[1.0, 0.9, 0.8, 0.7, ...]' (profil normalisé)
    parameters_json TEXT,                  -- '{"lambda": 0.15, "core_ratio": 0.7}' pour NMDs
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### Table `portfolios` — Portefeuilles d'encours

```sql
CREATE TABLE portfolios (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    version     INTEGER NOT NULL DEFAULT 1,
    status      TEXT NOT NULL DEFAULT 'draft',
    as_of_date  DATE NOT NULL,             -- date de référence du portefeuille
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE portfolio_positions (
    id              TEXT PRIMARY KEY,
    portfolio_id    TEXT NOT NULL REFERENCES portfolios(id),
    position_ref    TEXT,                  -- référence interne (numéro de prêt, ISIN...)
    product_type    TEXT NOT NULL,
    branch          TEXT,
    seller          TEXT,
    currency        TEXT NOT NULL DEFAULT 'USD',
    outstanding     REAL NOT NULL,
    origination_date DATE,
    maturity_date   DATE,
    client_rate     REAL,                  -- taux client (pour calcul NIM)
    runoff_model_id TEXT REFERENCES runoff_models(id),
    risk_weight     REAL DEFAULT 1.0,
    metadata_json   TEXT                   -- champs libres : région, secteur, rating...
);
```

#### Table `executions` — Registre de toutes les simulations

C'est la table centrale du système de reproductibilité.

```sql
CREATE TABLE executions (
    id              TEXT PRIMARY KEY,      -- UUID de l'exécution
    label           TEXT,                  -- "Simulation ALCO Janvier 2025"
    method          TEXT NOT NULL,         -- 'stock' | 'flux' | 'pool' | 'duration' | 'refinancing'
    portfolio_id    TEXT NOT NULL REFERENCES portfolios(id),
    curve_ids_json  TEXT NOT NULL,         -- '{"base_rate": "uuid1", "tlp": "uuid2", ...}'
    runoff_ids_json TEXT,                  -- '{"mortgage": "uuid3", "nmd": "uuid4"}'
    parameters_json TEXT NOT NULL,         -- '{"capital_ratio": 0.08, "coe": 0.12, "hurdle": 0.12}'
    -- Reproductibilité stochastique
    seeds_json      TEXT,                  -- '{"monte_carlo_rates": 42, "prepayment": 137}'  null si déterministe
    -- Métadonnées
    status          TEXT NOT NULL DEFAULT 'pending', -- 'pending' | 'running' | 'completed' | 'failed'
    duration_ms     INTEGER,               -- temps de calcul (pour benchmarking)
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by      TEXT,
    notes           TEXT
);
```

#### Table `alco_approvals` — Workflow de validation des inputs

```sql
CREATE TABLE alco_approvals (
    id          TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,             -- 'rate_curve' | 'runoff_model' | 'portfolio'
    entity_id   TEXT NOT NULL,
    action      TEXT NOT NULL,             -- 'submit' | 'approve' | 'reject' | 'archive'
    by_user     TEXT NOT NULL,
    at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    comment     TEXT
);
```

---

### Principe de Seed Multi-Composante

Pour les simulations contenant plusieurs sources d'aléatoire indépendantes, chaque composante stochastique a son propre seed :

```json
{
  "monte_carlo_rates": 42,
  "prepayment_cpr": 137,
  "nmd_behavioral": 2048
}
```

Cela permet de **figer une composante et re-randomiser une autre** :
- Rejouer la même trajectoire de taux mais avec un comportement NMD différent → `monte_carlo_rates: 42`, `nmd_behavioral: new_random()`
- Tester la sensibilité au seed Monte Carlo tout en gardant le même comportement NMD → `monte_carlo_rates: new_random()`, `nmd_behavioral: 2048`

Le kernel WASM accepte le seed comme paramètre d'entrée et initialise son PRNG (xoshiro256\*\* en Rust) de façon déterministe.

---

### Reproduire une Exécution Passée

```typescript
// Charger l'exécution depuis la DB
const exec = db.get('SELECT * FROM executions WHERE id = ?', execId)
const curves = loadCurvesByIds(JSON.parse(exec.curve_ids_json))
const portfolio = loadPortfolio(exec.portfolio_id)
const seeds = JSON.parse(exec.seeds_json ?? '{}')
const params = JSON.parse(exec.parameters_json)

// Relancer le kernel WASM avec exactement les mêmes inputs
const result = await kernelWasm.compute({
  method: exec.method,
  outstanding: portfolio.outstanding,
  profiles: portfolio.profiles,
  rates: buildRateMatrix(curves, params),
  seed_monte_carlo: seeds.monte_carlo_rates,
  seed_prepayment: seeds.prepayment_cpr,
  ...params
})

// Résultat identique bit-pour-bit à l'exécution originale
```

---

### Technologie : SQLite WASM (wa-sqlite)

Puisque l'objectif est de rester sans backend, la DB tourne dans le navigateur via **wa-sqlite** — SQLite compilé en WebAssembly, persisté dans l'IndexedDB du browser.

```
IndexedDB (navigateur)
    ↕ persistance
SQLite WASM (wa-sqlite)
    ↕ requêtes SQL standard
Svelte 5 stores / runes
    ↕ réactivité
ftp-calculator-core WASM (calculs)
```

**Export/Import de la DB** : un bouton "Backup" exporte le fichier `.sqlite` entier — archive autonome portable contenant l'intégralité de l'historique des inputs et des exécutions. Partage entre collègues, archivage réglementaire, audit externe.

**Migration future vers multi-utilisateurs** : le schéma SQLite est directement compatible PostgreSQL. Si un backend devient nécessaire, la migration est un `pg_restore` sur le `.sqlite` exporté.

---

### Ce que le Système de DB Garantit

| Propriété | Mécanisme |
|-----------|-----------|
| **Reproductibilité totale** | Inputs immuables + seeds stockés → résultat identique garanti |
| **Auditabilité réglementaire** | Chaque exécution référence exactement les versions d'inputs utilisées |
| **Immutabilité des inputs approuvés** | Statut `approved` → lecture seule, toute modification crée une nouvelle version |
| **Lineage complète** | Exécution → portfolio version → position → runoff model version + curve version |
| **Diff entre exécutions** | Comparer deux `executions.id` pour isoler l'impact d'un changement de courbe vs changement de portefeuille |
| **RGPD by design** | Tout en local (IndexedDB), aucune donnée ne quitte le navigateur sans action explicite de l'utilisateur |

---

## Architecture Technique

### Vue d'ensemble

```
┌────────────────────────────────────────────────────────────────┐
│                      PC LOCAL DE L'UTILISATEUR                 │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  NAVIGATEUR                                              │  │
│  │  Svelte 5 dashboard  →  requêtes REST  →  Backend API   │  │
│  └──────────────────────────────┬───────────────────────────┘  │
│                                 │ HTTP localhost:3000           │
│  ┌──────────────────────────────▼───────────────────────────┐  │
│  │  BACKEND (Rust + Axum) — service léger                   │  │
│  │  • API REST CRUD (courbes, portefeuilles, exécutions)    │  │
│  │  • Lance le kernel ftp-calculator-core (natif, pas WASM) │  │
│  │  • Gère les seeds et la reproductibilité                 │  │
│  └──────────────────────────────┬───────────────────────────┘  │
│                                 │                               │
│  ┌──────────────────────────────▼───────────────────────────┐  │
│  │  BASE DE DONNÉES                                          │  │
│  │  PostgreSQL 18 (mode Docker)                             │  │
│  │  OU SQLite fichier local (mode sans Docker)              │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

Le backend est un **binaire Rust unique** qui embarque `ftp-calculator-core` comme dépendance de crate — le kernel tourne nativement côté serveur, plus besoin de WASM. Le frontend Svelte est servi statiquement par le backend lui-même (un seul processus à lancer).

---

### Stack

| Couche | Technologie | Justification |
|--------|------------|--------------|
| UI | Svelte 5 (runes) | Réactivité native, bundle léger |
| Calcul | ftp-calculator-core (natif via backend) | Performance maximale, pas de sérialisation WASM |
| Backend | Rust + Axum | Léger (~5 MB binaire), même langage que le kernel, latence < 1ms |
| Base de données | PostgreSQL 18 (primaire) / SQLite (fallback) | PostgreSQL pour la robustesse ; SQLite pour le déploiement sans Docker |
| ORM / Migrations | sqlx | Async, compile-time query checking, supporte PG + SQLite |
| Charts | Apache ECharts | Richesse des types de graphiques, interactivité |
| Excel I/O | SheetJS (xlsx) | Import/export .xlsx côté client |
| PDF | jsPDF + html2canvas | Export PDF client-side |
| Styles | Tailwind CSS v4 | Utilitaires rapides, dark mode |
| State | Svelte 5 runes ($state, $derived) | Réactivité fine |
| Déploiement | Installeur natif (.exe / .dmg / .deb) | PostgreSQL portable embarqué, zéro prérequis |
| Build frontend | Vite | HMR rapide, sortie statique embarquée dans le binaire backend |

---

### Déploiement : Installeur Slim + Téléchargement depuis Serveur FTP

**Principe clé :** L'installeur `.exe` est **minimal** (~5 MB — binaires applicatifs seulement). Il ne contient **ni PostgreSQL ni les données d'exemple**. Tout est téléchargé pendant l'installation depuis un **serveur FTP interne** contrôlé par l'organisation.

```
FtpSimulator-v1.0-windows-x64.exe   (Inno Setup slim)
│
├── ftp-backend.exe          ← Backend Rust (frontend Svelte embarqué en statique)
├── ftp-tray.exe             ← Contrôleur system tray (Start/Stop/Open)
└── ftp-installer-helper.exe ← Helper de téléchargement FTP + vérification SHA256
```

> PostgreSQL 18 portable (~80 MB) et les jeux de données d'exemple sont stockés sur le serveur FTP de l'organisation et téléchargés à la demande.

#### Ce que fait l'installation (wizard classique)

```
Étape 1 — Extraction
  → ftp-backend.exe + ftp-tray.exe → C:\Program Files\FtpSimulator\

Étape 2 — Configuration du serveur FTP (pré-rempli, modifiable)
  ┌─────────────────────────────────────────────────────────┐
  │  Serveur FTP de téléchargement                          │
  │  URL  : [ftp://ftp.monorganisation.com/ftp-simulator/▼] │
  │  (Ce paramètre est mémorisé dans la config application) │
  └─────────────────────────────────────────────────────────┘

Étape 3 — Téléchargement de PostgreSQL 18
  → Barre de progression : "Téléchargement de PostgreSQL 18…"
  → Source : ftp://[serveur]/postgresql/postgresql-18-windows-x64.zip
  → Vérification SHA-256 (hash publié sur le serveur dans postgresql-18.sha256)
  → Extraction dans C:\Program Files\FtpSimulator\postgresql-18\

Étape 4 — Sélection des jeux de données d'exemple
  ┌─────────────────────────────────────────────────────────┐
  │  Jeux de données à installer (optionnels)               │
  │  ☑  Portefeuille Retail (250 contrats, ~2 MB)           │
  │  ☑  Portefeuille Corporate (80 contrats, ~1 MB)         │
  │  ☐  Portefeuille Mixte Grande Banque (1 200 contrats)   │
  │  ☐  Courbes de taux historiques 2015-2024 (~50 MB)      │
  │  ☐  Modèles de runoff NMD calibrés                      │
  └─────────────────────────────────────────────────────────┘
  → Téléchargement des datasets sélectionnés depuis ftp://[serveur]/datasets/
  → Vérification SHA-256 de chaque fichier

Étape 5 — Initialisation PostgreSQL
  → initdb.exe --pgdata="C:\ProgramData\FtpSimulator\data" --encoding=UTF8
  → Crée le cluster PostgreSQL (données dans ProgramData, pas Program Files)

Étape 6 — Création de la base
  → postgres.exe démarre temporairement
  → CREATE DATABASE ftp_simulator;
  → CREATE USER ftp WITH PASSWORD 'ftp_local';
  → GRANT ALL ON DATABASE ftp_simulator TO ftp;
  → Migrations sqlx appliquées automatiquement
  → Import des datasets sélectionnés (si applicable)

Étape 7 — Services Windows
  → Service "FtpSimulatorDB"  → postgres.exe  (démarre au boot, auto-restart)
  → Service "FtpSimulatorApp" → ftp-backend.exe (démarre au boot, dépend de DB)
  → (NSSM — Non-Sucking Service Manager — pour transformer un .exe en service)

Étape 8 — Raccourcis
  → Icône bureau → ouvre http://localhost:3000 dans le navigateur par défaut
  → ftp-tray.exe démarre → icône system tray verte
  → Notification : "FTP Simulator est prêt — cliquer pour ouvrir"
```

#### Structure du serveur FTP

```
ftp://ftp.monorganisation.com/ftp-simulator/
│
├── postgresql/
│   ├── postgresql-18-windows-x64.zip   ← EDB portable ZIP
│   ├── postgresql-18-windows-x64.sha256
│   ├── postgresql-18-macos-arm64.tar.gz
│   ├── postgresql-18-macos-arm64.sha256
│   ├── postgresql-18-linux-x64.tar.gz
│   └── postgresql-18-linux-x64.sha256
│
├── datasets/
│   ├── catalog.json                     ← liste des datasets + tailles + SHA256
│   ├── retail-portfolio-250.sql.gz
│   ├── corporate-portfolio-80.sql.gz
│   ├── mixed-portfolio-1200.sql.gz
│   ├── rate-curves-2015-2024.sql.gz
│   └── nmd-runoff-models.sql.gz
│
└── releases/
    ├── latest.json                      ← version courante pour l'auto-update
    ├── FtpSimulator-v1.0-windows-x64.exe
    ├── FtpSimulator-v1.0-macos.dmg
    └── FtpSimulator-v1.0-linux.deb
```

#### Configuration persistante

L'URL du serveur FTP est enregistrée dans `C:\ProgramData\FtpSimulator\config.toml` :

```toml
[ftp_server]
url = "ftp://ftp.monorganisation.com/ftp-simulator"
# Peut être modifié dans l'app → Paramètres → Serveur FTP
```

La **Bibliothèque de Datasets** dans l'application utilise cette même URL pour permettre le téléchargement de datasets supplémentaires après l'installation initiale.

#### Le contrôleur System Tray (`ftp-tray.exe`)

Petit exécutable Rust (~2 MB) utilisant la crate `tray-icon` + `winit` :

```
Clic droit sur l'icône tray :
┌─────────────────────────────┐
│  🟢 FTP Simulator (actif)   │
├─────────────────────────────┤
│  📂 Ouvrir le dashboard     │  → ouvre http://localhost:3000
│  ⏹  Arrêter les services    │  → net stop FtpSimulatorApp, FtpSimulatorDB
│  ▶  Démarrer les services   │  → net start FtpSimulatorDB, FtpSimulatorApp
│  📋 Voir les logs           │  → ouvre le dossier de logs
│  ℹ️  À propos               │  → version, licence
├─────────────────────────────┤
│  ✖  Quitter le tray         │  → ferme seulement l'icône, services continuent
└─────────────────────────────┘
```

L'icône est **verte** si les deux services tournent, **rouge** si un service est arrêté, **orange** si démarrage en cours.

#### Désinstallation propre

L'installeur Inno Setup génère automatiquement un désinstalleur :
- Arrête les services
- Propose de conserver ou supprimer les données (`C:\ProgramData\FtpSimulator\`)
- Supprime tout le reste

---

### Déploiement Cross-Platform

| Plateforme | Installeur | Gestionnaire de services |
|-----------|-----------|------------------------|
| **Windows** | Inno Setup `.exe` slim (~5 MB) | Windows Services via NSSM |
| **macOS** | `.dmg` avec App Bundle slim (~5 MB) | launchd (plist dans `~/Library/LaunchAgents/`) |
| **Debian 13** | `.deb` slim (~5 MB) | systemd (deux units : `ftp-simulator-db` + `ftp-simulator-app`) |

**macOS :** L'app bundle (`FtpSimulator.app`) est slim (~5 MB). Au premier lancement, un wizard télécharge PostgreSQL 18 macOS depuis le serveur FTP configuré et l'extrait dans `~/Library/Application Support/FtpSimulator/postgresql/`. Double-cliquer lance l'app dans la barre de menu (équivalent du tray Windows).

---

### Déploiement Debian 13 (Trixie) — Détail Complet

#### Ce que contient le paquet `.deb`

```
ftp-simulator_1.0_amd64.deb   (paquet Debian slim, ~5 MB)
│
├── /usr/bin/ftp-backend              ← binaire backend Rust
├── /usr/bin/ftp-installer-helper     ← helper téléchargement FTP + SHA-256
├── /lib/systemd/system/
│   ├── ftp-simulator-db.service      ← unit PostgreSQL 18
│   └── ftp-simulator-app.service     ← unit backend (After=ftp-simulator-db)
├── /etc/ftp-simulator/config.toml    ← config (FTP URL, ports)
├── /usr/share/applications/
│   └── ftp-simulator.desktop         ← entrée menu GNOME/KDE
└── /usr/share/ftp-simulator/
    └── icon.png
```

> PostgreSQL 18 portable est téléchargé depuis le serveur FTP pendant `postinst` — il n'est **pas** dans le `.deb` et n'est **pas** installé via `apt`.

#### Ce que fait l'installation (`dpkg -i` ou double-clic)

```
Script postinst (exécuté automatiquement par dpkg) :

Étape 1 — Lecture de la configuration FTP
  → /etc/ftp-simulator/config.toml consulté pour l'URL du serveur FTP
  → Si absent : wizard CLI interactif demande l'URL
    "Entrez l'URL du serveur FTP [ftp://ftp.monorganisation.com/ftp-simulator] :"

Étape 2 — Téléchargement de PostgreSQL 18 portable
  → ftp-installer-helper download \
        ftp://[serveur]/postgresql/postgresql-18-linux-x64.tar.gz \
        /opt/ftp-simulator/postgresql/ \
        --sha256 ftp://[serveur]/postgresql/postgresql-18-linux-x64.sha256
  → Extraction dans /opt/ftp-simulator/postgresql/
    (postgres, initdb, psql — aucun binaire PG dans /usr/bin, aucun conflit avec
     un éventuel postgresql système)

Étape 3 — Sélection des datasets d'exemple
  → ftp-installer-helper catalog ftp://[serveur]/datasets/catalog.json
  → Affiche la checklist en CLI :
    [ ] Portefeuille Retail (250 contrats, 2 MB)      → retail-portfolio-250.sql.gz
    [ ] Portefeuille Corporate (80 contrats, 1 MB)    → corporate-portfolio-80.sql.gz
    [ ] Portefeuille Mixte 1 200 contrats             → mixed-portfolio-1200.sql.gz
    [ ] Courbes historiques 2015–2024 (50 MB)         → rate-curves-2015-2024.sql.gz
    [ ] Modèles de runoff NMD calibrés                → nmd-runoff-models.sql.gz
  → L'utilisateur saisit les numéros des datasets souhaités (ex. "1 2")
  → Téléchargement + vérification SHA-256

Étape 4 — Initialisation PostgreSQL
  → Création de l'utilisateur système ftp-simulator (nologin)
  → /opt/ftp-simulator/postgresql/bin/initdb \
        --pgdata=/var/lib/ftp-simulator/pgdata \
        --username=ftp_simulator \
        --encoding=UTF8 \
        --auth=trust
  → pg_hba.conf : accès local uniquement (127.0.0.1/32, ::1/128)

Étape 5 — Création de la base
  → Démarrage temporaire de postgres
  → CREATE DATABASE ftp_simulator;
  → CREATE USER ftp WITH PASSWORD 'ftp_local';
  → GRANT ALL ON DATABASE ftp_simulator TO ftp;
  → Migrations sqlx appliquées (ftp-backend --migrate)
  → Import des datasets sélectionnés (psql -f *.sql)
  → Arrêt du postgres temporaire

Étape 6 — Activation des services systemd
  → systemctl daemon-reload
  → systemctl enable --now ftp-simulator-db.service
  → systemctl enable --now ftp-simulator-app.service
  → echo "FTP Simulator prêt → http://localhost:3000"
```

#### Units systemd

```ini
# /lib/systemd/system/ftp-simulator-db.service
[Unit]
Description=FTP Simulator — PostgreSQL 18
After=network.target

[Service]
Type=forking
User=ftp-simulator
ExecStart=/opt/ftp-simulator/postgresql/bin/pg_ctl \
    start -D /var/lib/ftp-simulator/pgdata \
    -l /var/log/ftp-simulator/postgresql.log
ExecStop=/opt/ftp-simulator/postgresql/bin/pg_ctl \
    stop -D /var/lib/ftp-simulator/pgdata
PIDFile=/var/lib/ftp-simulator/pgdata/postmaster.pid
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```ini
# /lib/systemd/system/ftp-simulator-app.service
[Unit]
Description=FTP Simulator — Backend Rust
After=ftp-simulator-db.service
Requires=ftp-simulator-db.service

[Service]
Type=simple
User=ftp-simulator
EnvironmentFile=/etc/ftp-simulator/env
ExecStart=/usr/bin/ftp-backend
Restart=on-failure
RestartSec=3

[Install]
WantedBy=multi-user.target
```

```bash
# /etc/ftp-simulator/env
DATABASE_URL=postgresql://ftp:ftp_local@127.0.0.1:5432/ftp_simulator
LISTEN_ADDR=127.0.0.1:3000
```

#### Fichiers et répertoires sur le système

```
/opt/ftp-simulator/postgresql/   ← PostgreSQL 18 portable (extrait du tar.gz FTP)
/var/lib/ftp-simulator/pgdata/   ← données PostgreSQL (cluster)
/var/lib/ftp-simulator/datasets/ ← datasets téléchargés
/var/log/ftp-simulator/          ← logs backend + PostgreSQL
/etc/ftp-simulator/config.toml   ← URL serveur FTP, ports
/etc/ftp-simulator/env           ← variables d'environnement service
```

#### Configuration persistante (Linux)

```toml
# /etc/ftp-simulator/config.toml
[ftp_server]
url = "ftp://ftp.monorganisation.com/ftp-simulator"
# Modifiable dans l'app → Paramètres → Serveur FTP
# (redémarre ftp-simulator-app pour prise en compte)

[server]
listen = "127.0.0.1:3000"
pg_port = 5432
```

#### Commandes utiles post-installation

```bash
# Statut des services
systemctl status ftp-simulator-db ftp-simulator-app

# Arrêt / démarrage
systemctl stop ftp-simulator-app ftp-simulator-db
systemctl start ftp-simulator-db ftp-simulator-app

# Logs en temps réel
journalctl -fu ftp-simulator-app
journalctl -fu ftp-simulator-db

# Télécharger un dataset supplémentaire (même outil que l'installeur)
ftp-installer-helper download-dataset nmd-runoff-models \
    --config /etc/ftp-simulator/config.toml

# Mise à jour de l'application
wget ftp://[serveur]/releases/FtpSimulator-v1.1_amd64.deb
dpkg -i FtpSimulator-v1.1_amd64.deb
# Le postinst met à jour les binaires sans toucher pgdata ni les datasets
```

#### Désinstallation propre

```bash
# Désinstallation (conserve les données)
dpkg -r ftp-simulator

# Désinstallation complète (supprime données + PostgreSQL portable)
dpkg -P ftp-simulator
# Le script postrm propose :
# "Supprimer les données PostgreSQL ? (/var/lib/ftp-simulator) [o/N]"
# "Supprimer les logs ? (/var/log/ftp-simulator) [o/N]"
```

---

### Pipeline de Build (GitHub Actions)

```yaml
# .github/workflows/release.yml
jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - Compile ftp-backend.exe (cargo build --release)
      - Compile ftp-tray.exe (cargo build --release)
      - Compile ftp-installer-helper.exe
      - Build frontend (vite build → statique embarqué via include_dir!)
      - Package avec Inno Setup → FtpSimulator-windows-x64.exe  (~5 MB slim)
      # PostgreSQL et datasets NE SONT PAS téléchargés ici — ils viennent du serveur FTP

  build-macos:
    runs-on: macos-latest
    steps:
      - Compile ftp-backend (cargo build --release)
      - Package App Bundle → FtpSimulator-macos.dmg  (~5 MB slim)
      - Code sign (Apple Developer certificate)

  build-linux:
    runs-on: ubuntu-latest
    container: debian:trixie  # Debian 13 — compatibilité glibc garantie
    steps:
      - Compile ftp-backend + ftp-installer-helper (cargo build --release, target x86_64-unknown-linux-gnu)
      - Build frontend (vite build → statique embarqué)
      - Package → ftp-simulator_amd64.deb (slim ~5 MB, postinst télécharge PG18 depuis FTP)

  upload-ftp:
    needs: [build-windows, build-macos, build-linux]
    steps:
      - Upload FtpSimulator-*.exe / .dmg / .deb vers ftp://[serveur]/ftp-simulator/releases/
      - Mettre à jour releases/latest.json (version + SHA256 de chaque package)
```

Chaque release GitHub génère automatiquement les 5 packages.

---

### Mise à jour de l'Application

Le tray controller vérifie silencieusement si une nouvelle version est disponible en lisant `ftp://[serveur]/ftp-simulator/releases/latest.json`. Si oui :
1. Notification discrète : "Une mise à jour est disponible (v1.1)"
2. Clic → télécharge le nouvel installeur slim depuis le serveur FTP
3. L'installeur met à jour les binaires **sans toucher aux données PostgreSQL** (migrations appliquées automatiquement au démarrage si schéma plus ancien)

> L'URL du serveur FTP lue dans `config.toml` est utilisée pour l'auto-update — pas de dépendance à GitHub.

---

### Structure des fichiers du projet

```
ftp-calculator/
├── crates/
│   ├── ftp-calculator-core/          (existant — kernel pur)
│   ├── ftp-calculator-bindings-pyo3/ (existant)
│   └── ftp-calculator-bindings-c/    (existant)
│
├── backend/                          (nouveau — service Rust + Axum)
│   ├── src/
│   │   ├── main.rs                   ← démarrage, connexion PG, serve frontend statique
│   │   ├── api/
│   │   │   ├── curves.rs
│   │   │   ├── portfolio.rs
│   │   │   ├── runoff.rs
│   │   │   ├── executions.rs
│   │   │   └── export.rs             ← pg_dump → téléchargement .sql
│   │   ├── compute/
│   │   │   ├── runner.rs             ← inputs DB → ftp-calculator-core → résultats
│   │   │   └── seed.rs
│   │   └── db/
│   │       ├── migrations/           ← .sql versionnés (sqlx migrate)
│   │       └── models.rs
│   └── Cargo.toml
│
├── tray/                             (nouveau — contrôleur system tray)
│   ├── src/main.rs                   ← tray-icon + winit, gestion services
│   └── Cargo.toml
│
├── dashboard/                        (nouveau — app Svelte 5)
│   ├── src/
│   │   ├── lib/
│   │   │   ├── api/          ← client TypeScript vers backend REST
│   │   │   ├── curve/        ← CoF Curve Builder
│   │   │   ├── methods/      ← toutes les méthodes FTP
│   │   │   ├── analytics/    ← NIM, RAROC, scénarios, vintage
│   │   │   └── export/       ← Excel, CSV, PDF
│   │   └── routes/
│   │       ├── +page.svelte        ← Dashboard principal
│   │       ├── curves/
│   │       ├── portfolio/
│   │       ├── executions/
│   │       ├── pricer/
│   │       ├── scenarios/
│   │       ├── nmd/
│   │       └── governance/
│   └── static/
│       └── templates/
│
├── installer/
│   ├── windows/
│   │   └── setup.iss             ← script Inno Setup
│   ├── macos/
│   │   └── bundle.sh             ← script App Bundle + dmg
│   └── linux/
│       ├── ftp-simulator.service ← systemd units
│       └── build-deb.sh
│
└── Cargo.toml                    ← workspace

---

## Priorités d'Implémentation

### Phase 1 — Fondation : Backend + PostgreSQL + Installeur de base
1. Crate `backend/` : Axum + sqlx + dépendance sur `ftp-calculator-core`
2. Migrations PostgreSQL 18 : toutes les tables (`rate_curves`, `portfolios`, `executions`…)
3. API REST minimale : CRUD courbes + portefeuilles + lancement calcul
4. Frontend Svelte minimal embarqué dans le binaire backend (`include_dir!`)
5. Calcul Stock + Flux natif via le kernel, enregistrement de l'exécution en DB
6. Installeur Windows slim (Inno Setup, ~5 MB) — télécharge PostgreSQL 18 depuis le serveur FTP configuré pendant l'installation
7. Tray controller minimal (start/stop/open)

### Phase 2 — Courbes et Reproductibilité
7. CoF Curve Builder avec 14 composantes → stockage en `rate_curves`
8. Versioning des courbes (draft → approved → archived)
9. Replay d'une exécution passée depuis son `execution_id`
10. Diff entre deux exécutions (quelle variation vient de la courbe vs du portefeuille ?)
11. Export/Import du fichier `.sqlite` complet

### Phase 3 — Richesse analytique
12. Décomposition NIM waterfall (BU actifs + BU passifs + Treasury)
13. Heatmaps branche × produit × vendeur
14. RAROC par ligne (Capital Charge paramétrable)
15. Scénarios de taux (6 scénarios BCBS, chocs parallèles et non-parallèles)

### Phase 4 — Méthodes complètes
16. Duration Method
17. Pool Method / Multiple Pool
18. Refinancing Method (UI construction des forward rates depuis spots)
19. Floating-Rate double profil (profil taux + profil liquidité)

### Phase 5 — Modules avancés
20. Pricer nouvelle origination (temps réel, calcul WASM instantané)
21. Modèle NMD + replicating portfolio optimizer
22. Analyse vintage (cohortes, courbes de survie)
23. Monte Carlo de taux avec seeds stockés en DB → reproductible
24. Mode pédagogique (formules pas à pas)

---

## Ce Que Cet Outil Permettrait Qu'Aucun Outil Standard ne Fait

1. **Toutes les méthodes FTP dans un seul outil** — aucun logiciel commercial ne couvre la totalité du spectre (pool → MMFTP → refinancing → replicating portfolio)
2. **Décomposition en 14 composantes visible** — les outils ALM commerciaux (Murex, Finastra Fusion) cachent cette décomposition dans des configurations opaques
3. **Kernel open-source, vérifiable** — on peut auditer exactement le calcul, contrairement aux boîtes noires commerciales
4. **100% client-side** — aucune donnée sensible ne quitte le navigateur, conformité RGPD par design
5. **Gratuit** — vs Oracle OFSAA, SAP IFRS, Finastra : licences à 7 chiffres
6. **Académique + pédagogique** — idéal pour la formation FTP en banque, les cours universitaires, les certifications (FRM, CFA)

---

*Ce document est le plan de conception. L'implémentation commence par le binding WASM du kernel existant.*

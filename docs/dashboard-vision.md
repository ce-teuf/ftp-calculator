# Dashboard FTP — Vision & Roadmap

## Concept central

Le dashboard est la pièce maîtresse de l'application. L'idée centrale est de permettre à l'utilisateur d'**explorer toutes les métriques FTP** (taux, marges, NIM, RAROC, encours…) selon **n'importe quelle vue organisationnelle**, avec une flexibilité totale sur l'ordre et la combinaison des niveaux hiérarchiques.

---

## 1. Graph organisationnel interactif (ECharts)

### Vision

Un graphe ECharts de type `graph` (force-directed ou hiérarchique) représentant l'ensemble de l'organisation bancaire. Chaque **nœud** est une entité, chaque **arête** est une relation de dépendance ou d'appartenance. Les nœuds affichent les métriques FTP en temps réel.

### Hiérarchie des entités

```
Siège (Headquarters)
├── Trésorerie US
│   └── Branch US
│       ├── BU 1
│       │   ├── Département A
│       │   ├── Département B
│       │   └── Sales (vendeurs)
│       │       └── Contrats (PAM, ANNUITE, MORTGAGE…)
│       └── BU 2 …
├── Trésorerie FR
│   └── Branch FR …
├── Trésorerie ES
│   └── Branch ES …
└── Trésorerie DE
    └── Branch DE …
```

Les données sources sont dans `data/datageneration_scripts/data_entities/`.

### Nœuds du graphe — propriétés

| Type de nœud   | Couleur suggérée | Métriques affichées                               |
|----------------|------------------|----------------------------------------------------|
| Siège          | Indigo (grand)   | Encours total, NIM global, FTP moyen global        |
| Trésorerie     | Violet           | Encours, NIM branche, FTP coût de funding          |
| Branch         | Bleu             | Encours, NIM, nb contrats, taux FTP moyen          |
| Business Unit  | Cyan             | Encours, NIM, nb vendeurs, nb contrats             |
| Département    | Vert             | Nb positions, NIM département                      |
| Sales (vendeur)| Ambre            | Encours portefeuille vendeur, NIM vendeur, RAROC   |
| Contrat        | Gris (petit)     | Notionnel, type, taux, maturité, FTP rate          |

### Invertibilité — le cœur de la fonctionnalité

L'utilisateur peut choisir **l'ordre de ventilation** des niveaux. Par exemple :

- Vue standard : Siège → Trésorerie → Branch → BU → Vendeur → Contrat
- Vue par produit : Produit (PAM / MORTGAGE…) → Branch → Vendeur
- Vue par métrique : NIM décroissant → Branch → BU
- Vue contributeur : Contrat → Vendeur → BU → Branch (bottom-up)

Cela revient à permettre à l'utilisateur de **reordonner ou filtrer les niveaux** du graphe, et le graphe se reconstruit dynamiquement avec les agrégations correspondantes.

### Interactions utilisateur

- **Clic sur un nœud** : zoom-in / drill-down vers les enfants
- **Double-clic** : expand/collapse les enfants directs
- **Hover** : tooltip riche avec toutes les métriques FTP du nœud
- **Sidebar de configuration** :
  - Ordre des niveaux (drag & drop)
  - Filtres (par branch, par devise, par type de contrat, par rating)
  - Métrique affichée sur les nœuds (NIM / FTP rate / Encours / RAROC)
  - Mode de layout (force-directed vs hiérarchique vs radial)
- **Barre de recherche** : trouver un vendeur, un contrat, une BU par nom/ID

### ECharts — type de graphe recommandé

```typescript
// Option de base pour le graphe organisationnel
const option = {
  series: [{
    type: 'graph',
    layout: 'force',           // ou 'none' pour hiérarchique manuel
    roam: true,                // pan + zoom
    focusNodeAdjacency: true,  // highlight voisins au hover
    force: {
      repulsion: 300,
      edgeLength: [80, 200],
    },
    nodes: [/* ... */],
    edges: [/* ... */],
    lineStyle: { curveness: 0.1, opacity: 0.6 },
    emphasis: { focus: 'adjacency' },
  }]
};
```

Pour le mode hiérarchique inversable, utiliser `layout: 'none'` avec des positions calculées en JS (algorithme Sugiyama ou d3-hierarchy adapté).

---

## 2. Vues analytiques du dashboard

En parallèle du graphe, des panneaux analytiques permettent d'explorer les métriques selon la dimension choisie.

### 2.1 Vue actuelle (déjà implémentée)
- KPIs globaux (encours, FTP moyen, int. FTP mensuel, nb exécutions)
- Décomposition NIM waterfall (revenus clients - coût FTP = NIM net)
- Heatmap taux FTP par méthode
- Heatmap NIM par produit / branche / vendeur
- Table des exécutions récentes

### 2.2 Vues à implémenter — par niveau organisationnel

#### Vue Branch
- Histogramme encours par branch (US / FR / ES / DE)
- NIM comparé par branch
- Part de chaque branch dans le FTP global

#### Vue Business Unit
- Scatter plot : encours BU vs NIM BU (bulle = nb contrats)
- Classement BU par RAROC

#### Vue Vendeur (Sales)
- Leaderboard NIM par vendeur (top 10 / bottom 10)
- Distribution des taux FTP accordés par vendeur (variance)

#### Vue Produit (Contract Type)
- Pie chart encours par type de contrat
- Box plot distribution des taux FTP par type
- Comparaison amortization schedules

#### Vue Courbe de taux
- Courbe forward actuelle vs historique
- Décomposition CoF (14 composantes superposées)
- Sensibilité FTP aux shifts de taux (BCBS)

#### Vue Scénarios
- Overlay de plusieurs scénarios sur le graphe organisationnel
- Delta NIM par branch sous chaque scénario

### 2.3 Filtres globaux (barre de contrôle)

Une barre de filtres persistante en haut du dashboard permettant de filtrer toutes les vues :

```
[Dataset: Q1 2026 ▼] [Branch: Toutes ▼] [Devise: EUR ▼] [Type: Tous ▼] [Méthode FTP: Toutes ▼]
```

Les filtres propagent à tous les panneaux via un store Svelte global.

---

## 3. Architecture technique

### Backend — endpoints nécessaires

```
GET /api/analytics/org-graph?dataset_id=&level_order=branch,bu,sales
  → nœuds + arêtes avec métriques FTP agrégées par niveau

GET /api/analytics/by-branch?dataset_id=&execution_id=
GET /api/analytics/by-bu?dataset_id=&execution_id=
GET /api/analytics/by-seller?dataset_id=&execution_id=
GET /api/analytics/by-product?dataset_id=&execution_id=
```

L'endpoint `org-graph` accepte un paramètre `level_order` qui définit l'ordre de groupement (inversabilité). Il agrège les métriques FTP des exécutions associées au dataset.

### Tables SQL impliquées

```sql
-- Jointure principale pour le graphe
SELECT
  s.seller_id, s.seller_name, s.bu_id,
  bu.bu_name, bu.branch_id,
  b.branch_code,
  c.contract_type, c.notional, c.interest_rate,
  -- métriques FTP depuis portfolio_positions + executions
  pp.profiles_json, pp.rates_json
FROM contracts c
JOIN dataset_items di ON di.entity_id = c.id AND di.entity_type = 'contract'
JOIN sales s ON s.seller_id = c.seller_id
JOIN business_units bu ON bu.bu_id = s.bu_id
JOIN branches b ON b.branch_id = bu.branch_id
WHERE di.dataset_id = $1
```

### Frontend — composants Svelte à créer

```
DashboardTab.svelte (shell)
├── DashboardFilters.svelte       ← barre de filtres globale
├── OrgGraph.svelte               ← graphe ECharts principal
│   ├── NodeTooltip.svelte        ← tooltip riche au hover
│   └── GraphControls.svelte      ← sidebar config (ordre niveaux, layout, métrique)
├── KpiBar.svelte                 ← 4 KPI cards (déjà fait)
├── NimWaterfall.svelte           ← décomposition NIM (déjà fait)
├── MethodHeatmap.svelte          ← heatmap méthodes (déjà fait)
├── NimHeatmap.svelte             ← heatmap NIM (déjà fait)
└── RecentExecutions.svelte       ← table exécutions (déjà fait)
```

### Store Svelte global pour les filtres

```typescript
// src/lib/stores/dashboard.ts
import { writable, derived } from 'svelte/store';

export const filters = writable({
  dataset_id:    null as string | null,
  branch:        'all',
  currency:      'all',
  contract_type: 'all',
  ftp_method:    'all',
});

export const levelOrder = writable([
  'branch', 'bu', 'department', 'seller', 'contract'
]);
```

---

## 4. Données nécessaires en base

Pour que le graphe soit complet, les tables suivantes doivent être peuplées :

| Table            | Script de génération                                 | Statut      |
|------------------|------------------------------------------------------|-------------|
| `contracts`      | `data_contracts/generate_contracts.py` → upload CSV  | ✅ Disponible |
| `dataset_items`  | Via upload ou génération                             | ✅ Disponible |
| `sales`          | `data_entities/generate_entities.py` → à importer   | ⚠️ Pas encore en DB |
| `business_units` | Idem                                                 | ⚠️ Pas encore en DB |
| `branches`       | Idem                                                 | ⚠️ Pas encore en DB |
| `treasuries`     | Idem                                                 | ⚠️ Pas encore en DB |

**Prochaine étape migration SQL** : créer les tables `sales`, `business_units`, `branches`, `treasuries` (migration 004) et les endpoints d'import CSV correspondants.

---

## 5. Priorités de développement

### Phase 1 — Données organisationnelles (prochaine)
1. Migration 004 : tables `branches`, `business_units`, `departments`, `treasuries`, `sales`
2. Upload CSV de ces entités via Datasets Creator (nouveaux types dans `entity_type`)
3. FK `contracts.seller_id → sales.seller_id` vérifiée en base

### Phase 2 — Graphe de base
4. Endpoint `/api/analytics/org-graph` avec agrégation configurable
5. Composant `OrgGraph.svelte` avec ECharts (layout force-directed, nœuds colorés par type)
6. Tooltip rich au hover (métriques FTP du nœud)

### Phase 3 — Invertibilité
7. `GraphControls.svelte` : drag & drop des niveaux, reconstruction dynamique du graphe
8. Modes de layout switchables (force / hiérarchique / radial)
9. Filtres globaux propagés

### Phase 4 — Vues analytiques complémentaires
10. Vue Branch, BU, Vendeur, Produit avec graphiques ECharts dédiés
11. Barre de filtres globale

---

## 6. Référence ECharts — exemples pertinents

- **Graph force-directed** : https://echarts.apache.org/examples/en/editor.html?c=graph
- **Graph hiérarchique (tree)** : https://echarts.apache.org/examples/en/editor.html?c=tree-basic
- **Graph radial** : https://echarts.apache.org/examples/en/editor.html?c=graph-circular-layout
- **Treemap** (alternative pour vue encours) : https://echarts.apache.org/examples/en/editor.html?c=treemap-simple
- **Sankey** (alternative pour flux FTP) : https://echarts.apache.org/examples/en/editor.html?c=sankey-simple

---

*Document créé le 2026-04-09. Mettre à jour au fur et à mesure des développements.*

# Dashboard d'exécution FTP — Plan des vues

> Document de référence pour l'implémentation du dashboard analytique dans `/executions`.
> Dernière mise à jour : 2026-04-12

---

## Pourquoi ce dashboard

Le simulateur FTP sert trois objectifs métier distincts :

1. **Pricing interne** — attribuer à chaque entité/portefeuille son coût de refinancement
   "juste" selon son profil d'amortissement et les conditions de marché à la date d'origination.
2. **Reporting ALM** — piloter l'évolution du book : encours, durée résiduelle moyenne (WAL),
   contribution FTP au PNB de la période.
3. **Analyse de sensibilité** — comparer scénarios (observed vs contrafactual vs projected),
   mesurer l'impact d'un choc de taux ou d'un changement de méthode (Stock vs Flux).

Le dashboard doit servir ces trois usages dans une seule page analytique, structurée en vues
thématiques accessibles par onglets. Chaque vue répond à une question métier précise.

---

## Données disponibles par pas de temps (output moteur)

Chaque `TimeStep` produit par le moteur Rust expose :

| Champ | Type | Description |
|-------|------|-------------|
| `date` | `string` | Mois YYYY-MM |
| `period_type` | `"observed" \| "projected" \| "contrafactual"` | Origine de la donnée |
| `kpis.total_outstanding` | `number` | Encours total en € |
| `kpis.weighted_ftp_rate` | `number` | Taux FTP pondéré (ratio décimal) |
| `kpis.ftp_interest_periodic` | `number` | Intérêts FTP du mois en € |
| `ftp_by_tenor` | `Record<bucket, number>` | Taux interpolé par tenor bucket |
| `profile` | `number[]` | Poids d'amortissement du schedule (Σ ≈ 1) |

---

## Vue I — KPI Synthèse *(bandeau permanent)*

**Question métier :** En un coup d'œil, quel est le résultat global de cette simulation ?

### Métriques affichées

| KPI | Calcul | Section |
|-----|--------|---------|
| Encours final observé | `last(outstanding)` sur observed | Snapshot |
| Encours projeté à horizon | `last(outstanding)` sur projected | Snapshot |
| Taux FTP moyen observé | Moyenne pondérée sur observed | Taux |
| Taux FTP moyen projeté | Moyenne pondérée sur projected | Taux |
| Intérêts FTP cumulés observés | `Σ ftp_interest_periodic` sur observed | P&L |
| Intérêts FTP cumulés projetés | `Σ ftp_interest_periodic` sur projected | P&L |
| WAL (durée résiduelle moyenne) | `Σ j * profile[j] / Σ profile[j]` au dernier pas | Duration |
| Méthode(s) utilisée(s) | Depuis `assignment.method` | Meta |

### UX
- Bandeau horizontal de tuiles, toujours visible au-dessus des onglets
- Tuiles cliquables → scroll vers l'onglet correspondant
- Code couleur : bleu = observé, orange = projeté, gris = méta

---

## Vue A — Timeline *(onglet "Timeline")*

**Question métier :** Comment évolue mon book et son coût FTP dans le temps ?

### Contenu
- **Axe X** : temps (mois YYYY-MM)
- **Axe Y gauche** : encours en € (barres, une couleur par assignment)
- **Axe Y droit** : taux FTP pondéré en % (lignes lissées)
- **Zone observée** : fond blanc/gris clair
- **Zone projetée** : fond jaune pâle (#FEF9C3 transparent)
- **Zone contrafactuelle** : fond rose pâle (si présente)
- **Ligne de séparation** : trait vertical plein à la date charnière observed→projected
- **Légende** : toggle par assignment

### ECharts
- `bar` series pour outstanding (yAxisIndex: 0)
- `line` series pour ftp_rate (yAxisIndex: 1, smooth: true)
- `markArea` pour les zones projected/contrafactual
- `markLine` pour le séparateur

### Pourquoi c'est important
Vue fondatrice. L'œil voit immédiatement où finit le réel et où commence le scénario.
Essentiel pour crédibiliser les projections en réunion ALCO.

---

## Vue B — Runoff par date *(onglet "Runoff")*

**Question métier :** Si je gèle mon book à la date T, comment s'amortit-il dans le futur ?

### Sous-vue B1 : Échelle d'amortissement (Ladder)

- **Sélecteur de dates** : chips cliquables sur toutes les dates disponibles
  (bleu = observed, orange = projected, violet = contrafactual)
- **Graphe** : barres groupées (ou empilées, toggle)
  - X : tenor bucket (1M, 2M, ..., 120M+)
  - Y : `outstanding(T) × profile[j]` en € — montant s'amortissant dans chaque bucket
  - Une série de couleur par date sélectionnée
- **Tooltip** : date sélectionnée + montant + % du total

### Sous-vue B2 : Intérêts FTP par horizon

- Même X (tenor bucket), même sélecteur de dates
- Y : `outstanding(T) × profile[j] × ftp_by_tenor[j] / 12` = intérêt FTP mensuel par bucket
- Vue "cash flow FTP" : combien génère chaque tranche d'amortissement

### ECharts
- `bar` series groupées (une par date sélectionnée)
- Tooltip formatter avec € et % du total

### Pourquoi c'est important
LA vue ALM par excellence. Permet de répondre à "si je regarde mon book au 31/12/2024,
combien me reste-t-il dans 1 an, 5 ans, 10 ans ?". Pilotage du risque de taux structurel.

---

## Vue C — Courbes FTP par tenor *(onglet "Courbes")*

**Question métier :** À quelle courbe FTP a été pricé mon book aux différentes dates ?

### Contenu
- Même sélecteur de dates que Vue B
- **Graphe** : courbes lignes
  - X : tenor bucket (1M → 120M+)
  - Y : `ftp_by_tenor[j]` en % — taux FTP à ce tenor pour cette date
  - Une courbe par date sélectionnée
- Overlay optionnel : courbe du taux de marché correspondant (si disponible)

### ECharts
- `line` series, une par date
- Marqueur de données sur les points de tenor effectifs
- Interpolation visuelle entre tenors (smooth: false pour voir les points réels)

### Pourquoi c'est important
Permet de voir comment la courbe FTP s'est déplacée : "en 2023-01 on payait 1.5% sur
5Y, en 2024-12 on paie 3.2%". Outil de benchmarking et de dialogue avec la trésorerie.

---

## Vue D — P&L FTP *(onglet "P&L")*

**Question métier :** Combien génère (ou coûte) le FTP chaque mois, et quel est le cumulé ?

### Contenu
- **Barres mensuelles** de `ftp_interest_periodic` (axe Y gauche)
  - Empilées par assignment si plusieurs
  - Couleur pleine = observed, hachuré ou pâle = projected
- **Ligne cumulative** (axe Y droit) : `Σ ftp_interest_periodic`
- **Séparateur** observed / projected
- **KPI en en-tête** : total observé | total projeté | run rate annuel (×12 du dernier mois)

### ECharts
- `bar` series empilées (une par assignment)
- `line` series pour le cumulatif (yAxisIndex: 1)
- `markArea` pour la zone projected

### Pourquoi c'est important
Question du CFO : "combien on gagne/paie en FTP ce mois-ci et sur l'année ?".
Le cumul valide la cohérence des projections de PNB.

---

## Vue E — Décomposition par composante *(onglet "Composantes")* `[PLANIFIÉ]`

**Question métier :** Quelle composante de taux contribue le plus au FTP (base rate, spread, liquidité) ?

### Contenu (nécessite backend)
- Pour chaque pas de temps : `ftp_rate = base_rate_contrib + credit_spread_contrib + liquidity_contrib + ...`
- Barres empilées avec une couleur par matrice source
- Légende : noms des matrices (`combination_matrix_ids` → noms)

### Changement backend requis
Le moteur doit stocker séparément la contribution de chaque matrice :
```rust
// Avant de sommer : stocker rates_by_matrix[matrix_name] = combined
"component_rates": { "Base Rate EUR": 0.025, "Credit Spread": 0.004, ... }
```

### Pourquoi c'est important
L'ALM doit expliquer *pourquoi* le FTP a changé. "30bp de hausse ce trimestre :
20bp viennent de la BCE, 10bp du liquidity premium." Indispensable pour le reporting réglementaire.

---

## Vue F — Heatmap taux *(onglet "Heatmap")*

**Question métier :** Comment se répartit le pricing FTP dans l'espace temps × tenor ?

### Contenu
- **Axe X** : dates (time steps)
- **Axe Y** : tenor buckets (1M → 120M+)
- **Couleur** : `ftp_by_tenor[date][tenor]` avec gradient froid → chaud
- Selector d'assignment si plusieurs
- `visualMap` : plage min/max automatique avec contrôle manuel

### ECharts
- `heatmap` series sur grille cartésienne
- `visualMap` piecewise ou continu
- Tooltip : date + tenor + taux en %

### Pourquoi c'est important
Vue d'ensemble instantanée : l'œil voit si les taux longs ont monté, si la courbe
s'est aplatie, si une période anomale existe. Impossible à voir sur des graphes individuels.

---

## Vue G — Stock vs Flux *(intégré à Timeline)*

**Question métier :** Mon book existant est-il pricé plus cher ou moins cher que les nouvelles originations ?

### Contenu
- Visible dans la Vue A (Timeline) si plusieurs assignments ont des méthodes différentes
- Assignment badge "Stock" ou "Flux" sur chaque série
- KPI dédié dans la bande synthèse : "Gap Stock-Flux : +X bp"

### Calcul du gap
- `gap(t) = avg_ftp_rate_flux(t) - avg_ftp_rate_stock(t)`
- Si gap > 0 : nouvelle prod plus chère que l'existant → pression sur le PNB futur
- Si gap < 0 : conditions actuelles meilleures que le book historique

### Pourquoi c'est important
Question fondamentale en ALM structurel. Quand Flux > Stock, le portefeuille se
renouvelle à des conditions moins favorables → signal de risque de taux de structure.

---

## Vue H — Données brutes *(onglet "Données")*

**Question métier :** Quel est le détail exact des calculs, mois par mois ?

### Contenu
- Tableau scrollable avec sticky header
- Colonnes : Date | Type | Encours | Taux FTP | Intérêts FTP | [tenors...]
- Code couleur par `period_type` (ligne observée, projetée, contrafactuelle)
- Export CSV (bouton) — génère un fichier avec toutes les colonnes
- Toggle pour afficher/masquer les colonnes de tenor

### Pourquoi c'est important
Les utilisateurs avancés (quant, risk) ont besoin du détail complet pour alimenter
d'autres modèles, faire du back-testing, ou auditer le calcul Rust.

---

## Priorité d'implémentation

| # | Vue | Onglet | Impact métier | Difficulté |
|---|-----|--------|--------------|------------|
| 1 | I — KPI synthèse | *(bandeau)* | ★★★ | ★ |
| 2 | A — Timeline obs/proj | Timeline | ★★★ | ★★ |
| 3 | B — Runoff ladder | Runoff | ★★★ | ★★ |
| 4 | C — Courbes FTP/tenor | Courbes | ★★★ | ★★ |
| 5 | D — P&L mensuel | P&L | ★★★ | ★★ |
| 6 | H — Données brutes | Données | ★★ | ★ |
| 7 | F — Heatmap | Heatmap | ★★ | ★★ |
| 8 | G — Stock vs Flux | *(dans Timeline)* | ★★ | ★ |
| 9 | E — Composantes | Composantes | ★★★ | ★★★ |

---

## Changements backend pour débloquer toutes les vues

### Déjà fait
- `method` par assignment dans le résultat
- Moteur dispatche vers `ComputeMethod::Stock` ou `ComputeMethod::Flux`

### À faire (petits ajouts)
1. **`period_type` par time step** — propagé depuis le vecteur d'encours
2. **`profile` par time step** — poids d'amortissement du schedule (pour Vue B)
3. **`component_rates` par time step** — contribution par matrice (pour Vue E, futur)

---

## Conventions UI

- **Bleu** (`#6366f1`) : observed, Stock
- **Orange** (`#f97316`) : projected
- **Violet** (`#7c3aed`) : contrafactual
- **Vert** (`#16a34a`) : Flux (nouvelle production)
- **Fond projeté** : `rgba(254,243,199,0.35)` (jaune pâle)
- **Fond contrafactuel** : `rgba(237,233,254,0.35)` (violet pâle)
- Séparateur observed/projected : ligne verticale `#f97316` en tirets

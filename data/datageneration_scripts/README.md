# FTP Curve Data Generation Scripts

Scripts Python pour générer des données factices (fake data) permettant de construire les courbes FTP lorsque les données réelles ne sont pas accessibles au public.

## Installation

```bash
pip install numpy pandas
```

## Structure

```
datageneration_scripts/
├── generate_all.py              # Script principal (complet)
├── generated_curves.json        # Sortie: toutes les composantes de courbes
├── sample_portfolio.json        # Sortie: portefeuille d'exemple

├── data_entities/               # Entités organisationnelles
│   ├── generate_entities.py
│   ├── branches.csv             # 4 branches (US, FR, ES, DE)
│   ├── business_units.csv       # 17 business units
│   ├── departments.csv          # 68 departments
│   ├── treasuries.csv           # 4 tresoreries (1 par branche)
│   └── sales.csv                # 255 vendeurs

├── data_contracts/              # Contrats (actifs & passifs)
│   └── generate_contracts.py
│   └── contracts.csv             # Contrats generés (PAM, ANNUITE, MORTGAGE, etc.)

├── data_rate_series/            # Series de taux
│   ├── generate_rate_series.py
│   ├── current_curves.json      # Courbes spots & forwards (SOFR, €STR, SONIA, EURIBOR)
│   ├── historical_*.csv         # Series historiques (3 ans)
│   ├── volatility_surface.json
│   └── basis_curves.json

└── data_schedules/              # Schedules & profils comportementaux
    ├── generate_schedules.py
    ├── nmd_behavioral_profiles.json   # 7 modeles NMD
    ├── schedule_linear_60m.csv
    ├── schedule_annuity_120m.csv
    ├── schedule_bullet_24m.csv
    ├── revolving_profile.json
    ├── credit_line_profile.json
    └── decaying_pool_profile.json
```

## Utilisation

### 1. Entités organisationnelles

```bash
cd data_entities && python3 generate_entities.py
```

**Sortie**:
- `branches.csv` - 4 branches: US (New York), FR (Paris), ES (Madrid), DE (Frankfurt)
- `business_units.csv` - 17 business units par branche
- `departments.csv` - 68 departments (Sales, Operations, Risk, Compliance)
- `treasuries.csv` - 1 trésorerie par branche
- `sales.csv` - 255 vendeurs avec email, target, seniority

**Relations**:
```
Branch (1) → Business Units (N)
Branch (1) → Treasury (1)
Business Unit (1) → Sales (N)
Business Unit (1) → Departments (N)
```

### 2. Contrats

```bash
cd data_contracts && python3 generate_contracts.py [nombre]
```

**Types de contrats**:
- **Actifs** (85%): PAM, ANNUITE, MORTGAGE, BULLET, REVOLVER, COMMERCIAL_LOAN
- **Passifs** (15%): DEMAND_DEPOSIT, SAVINGS, TERM_DEPOSIT, CERTIFICATE_OF_DEPOSIT

**Paramètres inclus** (prêts pour QuantLib):
- `settlement_date`, `maturity_date`, `tenor_months`
- `payment_frequency`: monthly, quarterly, semestrial, annual
- `day_count`: 30/360, ACT/360, ACT/365, ACT/ACT
- `amortization_type`: linear, constant_installment, bullet
- `prepayment_allowed`, `prepayment_penalty`

### 3. Series de taux

```bash
cd data_rate_series && python3 generate_rate_series.py
```

**Courbes**:
- SOFR (USD) - Base rate US
- €STR (EUR) - Base rate Euro
- SONIA (GBP) - Base rate UK
- EURIBOR (EUR)

**Tenors**: 1D, 1W, 2W, 1M, 2M, 3M, 6M, 9M, 1Y, 2Y, 3Y, 5Y, 7Y, 10Y, 15Y, 20Y, 30Y

**Fichiers**:
- `current_curves.json` - Spot + Forward rates
- `historical_*.csv` - 3 ans de données quotidienne
- `volatility_surface.json` - Surface de volatilité pour swaptions
- `basis_curves.json` - Basis EUR/USD, EUR/GBP, USD/GBP

### 4. Schedules & Profils comportementaux

```bash
cd data_schedules && python3 generate_schedules.py
```

**Profils NMD** (Non-Maturity Deposits):
```
profile[t] = core_ratio + (1 - core_ratio) × exp(-λ × t)
```

| Model | λ (decay) | Core Ratio | WAL |
|-------|-----------|------------|-----|
| Retail Demand Deposits Core | 0.05 | 75% | ~60 mois |
| Retail Demand Deposits Volatile | 0.15 | 40% | ~20 mois |
| Retail Savings Account | 0.08 | 60% | ~40 mois |
| Corporate Operating Account | 0.12 | 55% | ~30 mois |
| Corporate Cash Pool | 0.06 | 70% | ~50 mois |
| SME Deposits | 0.10 | 50% | ~35 mois |

**Schedules contractuels**:
- `schedule_linear_60m.csv` - Amortissement linéaire 60 mois
- `schedule_annuity_120m.csv` - Annuité constante 120 mois
- `schedule_bullet_24m.csv` - Bullet 24 mois

**Profils comportementaux**:
- Revolving facility: utilization saisonnière + bruit
- Credit line: drawdown progressif
- Decaying pool: décroissance exponentielle (pour securitization)

## Seed pour reproductibilité

Tous les scripts utilisent un seed fixe (`random.seed(42)`). Pour générer des données différentes, modifier le seed dans chaque script.

## Dépendances Optionnelles

Pour la génération avancée des contrats avec les vrais paramètres QuantLib:

```bash
pip install QuantLib
```

Sinon, les scripts utilisent une génération simplifiée sans QuantLib.

## Schéma relationnel des données

```
contracts
├── seller_id → sales.seller_id
├── branch_code → branches.branch_code
└── treasury_id → treasuries.treasury_id (pour les dépôts)

sales
├── bu_id → business_units.bu_id
└── branch_id → branches.branch_id

business_units
└── branch_id → branches.branch_id
```

Les contrats actifs (loans) sont associés à un `seller_id` (vendeur), ce qui permet de remonter à la Business Unit et à la Branch.

Les contrats passifs (dépôts) sont associés à un `treasury_id`, permettant d'imputer le funding aux trésorerie des branches.
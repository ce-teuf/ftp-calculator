# FTP Simulator

Application web complète de **calcul et pilotage du Funds Transfer Pricing (FTP)** pour les équipes ALM/Trésorerie bancaires.

Moteur de calcul en **Rust**, dashboard en **Svelte 5**, base de données **PostgreSQL 18**.

---

## Architecture

```
ftp-calculator/
├── app/
│   ├── backend/          # API REST Rust/Axum (binaire autonome, frontend embarqué)
│   └── dashboard/        # UI Svelte 5 + Vite (dev: HMR ; prod: embarqué dans le binaire)
├── crates-core/
│   ├── ftp-calculator-core/           # Moteur de calcul Rust (9 méthodes FTP)
│   ├── ftp-calculator-bindings-c/     # Bindings C → Excel Add-In
│   └── ftp-calculator-bindings-pyo3/  # Bindings Python (PyO3)
├── installer/
│   ├── ftp-installer-helper/  # Téléchargeur FTP (télécharge PostgreSQL à l'install)
│   ├── build-deb.sh           # Paquet Debian slim
│   ├── windows/setup.iss      # Installeur Inno Setup (Windows)
│   └── macos/bundle.sh        # App Bundle + DMG (macOS)
├── data/dev/seed.sql     # Jeu de données de démonstration (dev local, sans FTP)
├── .env.dev              # Variables d'environnement de développement
├── .env.prod.example     # Template de configuration de production
├── Makefile              # Orchestration dev + prod
└── .github/workflows/    # Pipeline CI/CD (Windows / macOS / Linux Debian 13)
```

---

## Méthodes FTP implémentées

| Méthode | Description |
|---|---|
| Stock | Taux de marché anti-diagonaux, stock amorti |
| Flux | Profil de liquidité, taux de marché par tranche |
| Matched Maturity | Courbe spot interpolée à la duration exacte |
| Pool | Taux pool pondéré par duration moyenne |
| Optionnel | Inclut la valeur des options de remboursement anticipé |
| Refinancement | Taux de refinancement court terme à la maturité |
| Taux Variable | Double profil taux + liquidité |
| Marchés | Taux de marché direct (OIS/IBOR) |
| Comportemental | Runoff comportemental (NMD, épargne) |

---

## Démarrage rapide — Développement

> **Prérequis :** Rust stable, Node.js 20+, **Docker** (pour PostgreSQL)

```bash
# 1. Démarrer PostgreSQL dans Docker
make dev-db

# 2. Charger les données de démonstration (10 positions, 3 courbes)
make dev-seed

# 3. Lancer le backend (terminal 1)
make dev-backend      # → http://localhost:3000

# 4. Lancer le frontend Vite avec HMR (terminal 2)
make dev-frontend     # → http://localhost:5173

# Ou tout en un avec tmux
make dev-tmux
```

En mode dev :
- **PostgreSQL tourne dans Docker** (`docker-compose.dev.yml`) — pas d'installation locale requise
- Backend et frontend tournent localement (rechargement automatique)
- Pas de serveur FTP — les données sont dans `data/dev/seed.sql`
- Le frontend tourne sur Vite (:5173) avec proxy `/api → :3000`

---

## Démarrage rapide — Production

> **Prérequis sur la machine cible :** Docker Engine (>= 23.0) avec Docker Compose v2

```bash
# 1. Compiler le binaire release (frontend Svelte embarqué)
make prod-build

# 2. Générer le paquet Debian
make prod-deb         # → dist/ftp-simulator_X.Y.Z_amd64.deb

# 3. Installer sur la machine cible
sudo dpkg -i dist/ftp-simulator_X.Y.Z_amd64.deb
# → vérifie Docker, démarre le conteneur PostgreSQL, démarre le backend
```

En mode prod :
- **PostgreSQL tourne dans Docker** (image `postgres:17`, volume persistant `ftp-simulator-pgdata`)
- Le mot de passe DB est généré aléatoirement à l'installation (`/etc/ftp-simulator/db_password`)
- Le frontend est compilé et embarqué dans le binaire Rust (`include_dir!`) — un seul binaire autonome
- L'installeur **vérifie que Docker est installé** et bloque si ce n'est pas le cas

---

## Référence Makefile

### Développement

| Commande | Description |
|---|---|
| `make dev-up` | Initialise la DB + affiche les instructions |
| `make dev-db` | Crée la base PostgreSQL de dev |
| `make dev-seed` | Charge `data/dev/seed.sql` |
| `make dev-backend` | Lance le backend Rust (port 3000) |
| `make dev-frontend` | Lance Vite HMR (port 5173) |
| `make dev-tmux` | Lance les deux dans des panneaux tmux |
| `make dev-reset` | Remet la base à zéro + reseed |
| `make dev-stop` | Arrête backend + session tmux |

### Production

| Commande | Description |
|---|---|
| `make prod-build` | Compile release (frontend embarqué) |
| `make prod-deb` | Génère le paquet `.deb` (Debian 13) |
| `make prod-check` | Vérifie `.env.prod` + code Rust |
| `make prod-run` | Lance le binaire release localement |

### Tests & qualité

| Commande | Description |
|---|---|
| `make test` | Tous les tests du workspace |
| `make unit` | Tests unitaires core |
| `make integration` | Tests d'intégration |
| `make check` | Clippy + fmt |
| `make coverage` | Rapport tarpaulin (HTML) |
| `make ci` | Pipeline CI local (check + test) |

### Bindings (Python / C / Excel)

| Commande | Description |
|---|---|
| `make build-core` | Compile `ftp-calculator-core` |
| `make build-c-bindings` | `.so/.dll` pour l'add-in Excel |
| `make build-py-bindings` | Wheel Python (maturin) |

---

## Dashboard — Onglets

| Onglet | Contenu |
|---|---|
| **Dashboard** | NIM waterfall agrégée, heatmap par branche/produit/vendeur, export JSON |
| **Courbes** | Bibliothèque de courbes de taux + CoF Curve Builder 14 composantes |
| **Portefeuille** | CRUD positions, RAROC par position, export Excel |
| **Exécutions** | Historique, replay, diff A/B, export Excel multi-onglets |
| **Pricer** | Calcul à la volée sur une position unique |
| **Scénarios** | 6 chocs BCBS (parallel ±100/200 bps, bear flattening, bull steepening…) |
| **NMD** | Calibration λ OLS, WAL, profil combiné (core + volatile), sauvegarde runoff model |
| **Gouvernance** | Approbation ALCO des courbes, piste d'audit des exécutions |

---

## Configuration

### `.env.dev` (développement)

```env
DATABASE_URL=postgresql://ftp_dev:ftp_dev@127.0.0.1:5432/ftp_simulator_dev
LISTEN_ADDR=127.0.0.1:3000
RUST_LOG=info,ftp_backend=debug
```

### `.env.prod` (production — à créer depuis `.env.prod.example`)

```env
DATABASE_URL=postgresql://ftp:CHANGE_ME@127.0.0.1:5432/ftp_simulator
LISTEN_ADDR=127.0.0.1:3000
RUST_LOG=warn
FTP_SOURCE_URL=ftp://ftp.monorganisation.com/ftp-simulator
```

---

## CI/CD

Le pipeline `.github/workflows/release.yml` produit sur chaque tag :

- `ftp-simulator-X.Y.Z-windows-setup.exe` — installeur Inno Setup slim (télécharge PG18 via FTP)
- `ftp-simulator-X.Y.Z-macos.dmg` — App Bundle macOS (menu-bar only, aarch64)
- `ftp-simulator_X.Y.Z_amd64.deb` — paquet Debian 13 slim

Les artefacts sont uploadés sur le serveur FTP de l'organisation avec un `latest.json` (SHA256 + URLs).

---

## Bindings Python / Excel (héritage)

Le moteur de calcul est aussi exposé en Python et via un Add-In Excel :

```python
from ftp_calculator import FtpCalculator
calc = FtpCalculator(outstanding, profiles, rates)
calc.compute("stock")
print(calc.ftp_rate)
```

```bash
make build-py-bindings   # génère la wheel Python
make build-c-bindings    # génère le .so/.dll pour l'add-in Excel
```

---

## Licence

MIT OR Apache-2.0

## Auteur

Charles Teuf

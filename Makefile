# ==============================================================================
# FTP Simulator — Makefile
# ==============================================================================
#
# ENVIRONNEMENTS
#   dev   PostgreSQL local, frontend Vite HMR, pas de FTP, données dans data/dev/
#   prod  Build release, frontend embarqué, installeur, FTP serveur pour les data
#
# DÉMARRAGE RAPIDE
#   make dev-up       Démarre tout l'environnement de développement
#   make dev-seed     Charge les données de démonstration (local, sans FTP)
#   make dev-stop     Arrête PostgreSQL de dev
#
#   make prod-build   Compile le binaire release (frontend embarqué)
#   make prod-deb     Génère le paquet Debian slim (.deb)
#
# ==============================================================================

# ── Variables ──────────────────────────────────────────────────────────────────

CARGO        = cargo
SQLX_OFFLINE = true
BACKEND_DIR  = app/backend
FRONTEND_DIR = app/dashboard
INSTALLER_DIR= installer
DATA_DIR     = data/dev

# Environnement de développement
DEV_DB_NAME  = ftp_simulator_dev
DEV_DB_USER  = ftp_dev
DEV_DB_PASS  = ftp_dev
DEV_DB_PORT  = 5432
DEV_DB_URL   = postgresql://$(DEV_DB_USER):$(DEV_DB_PASS)@127.0.0.1:$(DEV_DB_PORT)/$(DEV_DB_NAME)
DATASETS_DIR = $(shell pwd)/data/datageneration_scripts/datasets

# Couleurs
GREEN  = \033[0;32m
RED    = \033[0;31m
YELLOW = \033[0;33m
BLUE   = \033[0;34m
CYAN   = \033[0;36m
NC     = \033[0m

.PHONY: all help \
        dev-up dev-db dev-migrate dev-backend dev-frontend dev-seed dev-stop dev-reset dev-logs \
        prod-build prod-deb prod-check \
        test unit integration check lint fmt \
        build-core build-py-bindings build-c-bindings \
        docs-serve docs-deploy \
        clean clean-dev

# ── Cible par défaut ───────────────────────────────────────────────────────────

all: help

# ==============================================================================
# ENVIRONNEMENT DEV
# ==============================================================================
# - PostgreSQL tourne en local (pg_ctl ou service système)
# - Données de seed dans data/dev/seed.sql (pas de serveur FTP)
# - Backend démarre avec cargo run (SQLX_OFFLINE=true)
# - Frontend démarre avec npm run dev (Vite HMR sur :5173, proxy /api → :3000)

## Démarre l'environnement de développement complet
dev-up: dev-db
	@echo "$(CYAN)==> Environnement de développement$(NC)"
	@echo "$(BLUE)    Backend  : http://localhost:3000  (cargo run)$(NC)"
	@echo "$(BLUE)    Frontend : http://localhost:5173  (Vite HMR)$(NC)"
	@echo "$(BLUE)    DB       : postgresql://localhost:$(DEV_DB_PORT)/$(DEV_DB_NAME)$(NC)"
	@echo ""
	@echo "$(YELLOW)Lancez dans deux terminaux séparés :$(NC)"
	@echo "  make dev-backend    # Terminal 1 — API Rust"
	@echo "  make dev-frontend   # Terminal 2 — Svelte HMR"
	@echo ""
	@echo "Ou tout en un avec tmux :"
	@echo "  make dev-tmux"

## Démarre PostgreSQL de dev dans Docker (crée le conteneur si absent)
dev-db:
	@echo "$(BLUE)==> PostgreSQL (dev) via Docker$(NC)"
	@docker compose -f docker-compose.dev.yml up -d
	@echo "$(YELLOW)    Attente que PostgreSQL soit prêt...$(NC)"
	@until docker compose -f docker-compose.dev.yml exec -T db \
	    pg_isready -U $(DEV_DB_USER) -d $(DEV_DB_NAME) -q 2>/dev/null; do sleep 1; done
	@echo "$(GREEN)✓ PostgreSQL prêt sur 127.0.0.1:$(DEV_DB_PORT)$(NC)"

## Lance le backend en mode développement (rechargement automatique avec cargo-watch)
dev-backend:
	@echo "$(BLUE)==> Backend (dev) sur http://localhost:3000$(NC)"
	@cd $(BACKEND_DIR) && \
	  DATABASE_URL=$(DEV_DB_URL) DATASETS_DIR=$(DATASETS_DIR) \
	  SQLX_OFFLINE=$(SQLX_OFFLINE) RUST_LOG=info,ftp_backend=debug \
	  $(CARGO) run 2>&1

## Lance le frontend en mode développement (Vite HMR + proxy → :3000)
dev-frontend:
	@echo "$(BLUE)==> Frontend (dev) sur http://localhost:5173$(NC)"
	@cd $(FRONTEND_DIR) && npm run dev

## Lance backend + frontend dans des panneaux tmux
dev-tmux:
	@which tmux > /dev/null || (echo "$(RED)tmux non installé$(NC)" && exit 1)
	@tmux new-session -d -s ftp-dev -n backend \
	  "cd $(BACKEND_DIR) && DATABASE_URL=$(DEV_DB_URL) SQLX_OFFLINE=$(SQLX_OFFLINE) cargo run; read"
	@tmux new-window -t ftp-dev -n frontend \
	  "cd $(FRONTEND_DIR) && npm run dev; read"
	@tmux attach -t ftp-dev
	@echo "$(CYAN)Ctrl+B puis D pour détacher, Ctrl+B X pour fermer un panneau$(NC)"

## Applique les migrations SQL (crée les tables) sans démarrer le backend
dev-migrate:
	@echo "$(BLUE)==> Application des migrations$(NC)"
	@for f in $(BACKEND_DIR)/src/db/migrations/*.sql; do \
	  echo "    $$f"; \
	  docker compose -f docker-compose.dev.yml exec -T db \
	    psql -U $(DEV_DB_USER) -d $(DEV_DB_NAME) -q < $$f; \
	done
	@echo "$(GREEN)✓ Migrations appliquées$(NC)"

## Charge les données de démonstration dans la DB Docker
dev-seed: dev-migrate
	@echo "$(BLUE)==> Chargement du seed de développement$(NC)"
	@docker compose -f docker-compose.dev.yml exec -T db \
	    psql -U $(DEV_DB_USER) -d $(DEV_DB_NAME) < $(DATA_DIR)/seed.sql
	@echo "$(GREEN)✓ Données chargées dans $(DEV_DB_NAME)$(NC)"

## Affiche les logs du backend (si lancé via systemd)
dev-logs:
	@journalctl -fu ftp-simulator-app 2>/dev/null || \
	  echo "$(YELLOW)Backend lancé manuellement — pas de journalctl disponible$(NC)"

## Remet la base de dev à zéro (supprime le volume Docker + recrée)
dev-reset:
	@echo "$(YELLOW)==> Réinitialisation de la base de dev$(NC)"
	@docker compose -f docker-compose.dev.yml down -v
	@$(MAKE) dev-db dev-seed
	@echo "$(GREEN)✓ Base réinitialisée et seedée$(NC)"

## Arrête les processus de développement locaux (backend, frontend, DB Docker)
dev-stop:
	@echo "$(YELLOW)==> Arrêt de l'environnement de dev$(NC)"
	@pkill -f "ftp-backend" 2>/dev/null && echo "$(GREEN)✓ Backend arrêté$(NC)" || echo "Backend non actif"
	@tmux kill-session -t ftp-dev 2>/dev/null || true
	@docker compose -f docker-compose.dev.yml stop && echo "$(GREEN)✓ PostgreSQL (Docker) arrêté$(NC)" || true

# ==============================================================================
# ENVIRONNEMENT PROD
# ==============================================================================
# - Frontend embarqué dans le binaire (include_dir!)
# - PostgreSQL téléchargé depuis le serveur FTP de l'organisation
# - Installeurs : .deb (Linux) + .exe (Windows via CI) + .dmg (macOS via CI)

## Compile le binaire de production (frontend Svelte embarqué)
prod-build: prod-frontend
	@echo "$(BLUE)==> Build release (prod)$(NC)"
	@SQLX_OFFLINE=$(SQLX_OFFLINE) $(CARGO) build --release -p ftp-backend -p ftp-installer-helper
	@echo "$(GREEN)✓ Binaires compilés dans target/release/$(NC)"
	@echo "    ftp-backend              — serveur API + frontend statique"
	@echo "    ftp-installer-helper     — téléchargeur FTP"

## Compile le frontend Svelte en mode production
prod-frontend:
	@echo "$(BLUE)==> Build frontend (prod)$(NC)"
	@cd $(FRONTEND_DIR) && npm ci --silent && npm run build
	@echo "$(GREEN)✓ dist/ généré$(NC)"

## Génère le paquet Debian slim (.deb)
prod-deb: prod-build
	@echo "$(BLUE)==> Génération du paquet .deb$(NC)"
	@bash $(INSTALLER_DIR)/build-deb.sh
	@echo "$(GREEN)✓ Paquet disponible dans dist/$(NC)"

## Vérifie la configuration de production (sans lancer)
prod-check:
	@echo "$(BLUE)==> Vérification configuration prod$(NC)"
	@[ -f .env.prod ] && echo "$(GREEN)✓ .env.prod présent$(NC)" || \
	  (echo "$(RED)✗ .env.prod manquant — copier .env.prod.example$(NC)" && exit 1)
	@grep -q 'FTP_SOURCE_URL' .env.prod && \
	  echo "$(GREEN)✓ FTP_SOURCE_URL configuré$(NC)" || \
	  echo "$(YELLOW)⚠ FTP_SOURCE_URL non configuré dans .env.prod$(NC)"
	@SQLX_OFFLINE=$(SQLX_OFFLINE) $(CARGO) check -p ftp-backend -p ftp-installer-helper 2>&1 | \
	  grep -E "^error" || echo "$(GREEN)✓ Code Rust valide$(NC)"

## Lance le binaire de production localement (pour tester avant déploiement)
prod-run:
	@[ -f .env.prod ] || (echo "$(RED).env.prod manquant$(NC)" && exit 1)
	@echo "$(BLUE)==> Lancement prod sur http://localhost:3000$(NC)"
	@env $$(cat .env.prod | grep -v '^#' | xargs) ./target/release/ftp-backend

# ==============================================================================
# TESTS & QUALITÉ
# ==============================================================================

## Lance tous les tests du workspace
test:
	@echo "$(BLUE)==> Tests$(NC)"
	@$(CARGO) test --workspace 2>&1
	@echo "$(GREEN)✓ Tous les tests sont passés$(NC)"

## Tests du core uniquement
unit:
	@$(CARGO) test -p ftp-calculator-core

## Tests d'intégration
integration:
	@$(CARGO) test -p ftp-calculator-core --test integration_tests

## Vérification statique (clippy + fmt)
check:
	@echo "$(BLUE)==> Clippy$(NC)"
	@$(CARGO) clippy --workspace -- -D warnings
	@echo "$(BLUE)==> Format$(NC)"
	@$(CARGO) fmt --all -- --check
	@echo "$(GREEN)✓ Code valide$(NC)"

lint:
	@$(CARGO) clippy --workspace -- -D warnings

fmt:
	@$(CARGO) fmt --all

## Couverture de code
coverage:
	@cargo tarpaulin --workspace --ignore-tests --out Html
	@echo "$(GREEN)✓ Rapport dans tarpaulin-report.html$(NC)"

# ==============================================================================
# BUILD DES BINDINGS (legacy — pour Python + Excel)
# ==============================================================================

UNAME := $(shell uname)
ifeq ($(UNAME), Linux)
  LIB_NAME  := libftp_calculator_bindings_c.so
  VENV_BIN  := .venv/bin
else ifeq ($(UNAME), Darwin)
  LIB_NAME  := libftp_calculator_bindings_c.dylib
  VENV_BIN  := .venv/bin
else
  LIB_NAME  := ftp_calculator_bindings_c.dll
  VENV_BIN  := .venv/Scripts
endif

build-core:
	@$(CARGO) build --release -p ftp-calculator-core

build-c-bindings:
	@echo "$(BLUE)==> Bindings C$(NC)"
	@cd crates-core/ftp-calculator-bindings-c && $(CARGO) build --release
	@mkdir -p excel-addin/Interop
	@cp target/release/$(LIB_NAME) excel-addin/Interop/
	@echo "$(GREEN)✓ $(LIB_NAME) copié dans excel-addin/Interop/$(NC)"

build-py-bindings:
	@echo "$(BLUE)==> Bindings Python$(NC)"
	@cd python-lib && ../$(VENV_BIN)/maturin build --release
	@echo "$(GREEN)✓ Wheel Python générée$(NC)"

# ==============================================================================
# DOCUMENTATION
# ==============================================================================

docs-serve:
	@echo "$(BLUE)==> Documentation MkDocs sur http://localhost:8000$(NC)"
	@$(VENV_BIN)/mkdocs serve -f mkdocs.yml

docs-deploy:
	@$(VENV_BIN)/mkdocs gh-deploy --force -f mkdocs.yml
	@echo "$(GREEN)✓ Documentation déployée sur GitHub Pages$(NC)"

# ==============================================================================
# SETUP
# ==============================================================================

## Installe tous les outils de développement
setup-dev:
	@echo "$(BLUE)==> Installation des outils de développement$(NC)"
	@rustup component add clippy rustfmt
	@cargo install cargo-watch cargo-tarpaulin 2>/dev/null || true
	@npm install --prefix $(FRONTEND_DIR)
	@test -d .venv || python -m venv .venv
	@$(VENV_BIN)/pip install maturin mkdocs-material mkdocstrings mkdocstrings-python pdoc 2>/dev/null || true
	@echo "$(GREEN)✓ Environnement de développement prêt$(NC)"
	@echo ""
	@echo "Démarrez avec :  make dev-up"

# ==============================================================================
# NETTOYAGE
# ==============================================================================

## Nettoyage des artefacts de build
clean:
	@echo "$(YELLOW)==> Nettoyage$(NC)"
	@$(CARGO) clean
	@rm -rf dist/
	@rm -rf $(FRONTEND_DIR)/dist/
	@echo "$(GREEN)✓ Nettoyé$(NC)"

## Supprime le conteneur et le volume Docker de dev
clean-dev:
	@docker compose -f docker-compose.dev.yml down -v 2>/dev/null || true
	@echo "$(GREEN)✓ Conteneur et volume PostgreSQL dev supprimés$(NC)"

# ==============================================================================
# CI/CD LOCALE
# ==============================================================================

## Simulation complète du pipeline CI
ci: check test
	@echo "$(GREEN)✓ Pipeline CI local OK$(NC)"

# ==============================================================================
# AIDE
# ==============================================================================

help:
	@echo ""
	@echo "$(CYAN)╔══════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║             FTP Simulator — Guide des commandes          ║$(NC)"
	@echo "$(CYAN)╚══════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(GREEN)── DÉVELOPPEMENT (sans FTP, données locales) ──$(NC)"
	@echo "  make dev-up          Démarre PostgreSQL Docker + affiche les instructions"
	@echo "  make dev-db          Démarre PostgreSQL dans Docker (docker-compose.dev.yml)"
	@echo "  make dev-backend     Lance le backend Rust localement (port 3000)"
	@echo "  make dev-frontend    Lance Vite HMR localement (port 5173, proxy → 3000)"
	@echo "  make dev-tmux        Lance backend + frontend dans des panneaux tmux"
	@echo "  make dev-seed        Charge le jeu de données de démo dans le conteneur DB"
	@echo "  make dev-reset       Supprime le volume Docker + recrée + reseed"
	@echo "  make dev-stop        Arrête backend + tmux + conteneur PostgreSQL"
	@echo ""
	@echo "$(GREEN)── PRODUCTION (build release, installeur, FTP serveur) ──$(NC)"
	@echo "  make prod-build      Compile release (frontend embarqué)"
	@echo "  make prod-deb        Génère le paquet .deb slim Debian 13"
	@echo "  make prod-check      Vérifie .env.prod + code Rust"
	@echo "  make prod-run        Lance le binaire release localement (.env.prod)"
	@echo ""
	@echo "$(GREEN)── TESTS & QUALITÉ ──$(NC)"
	@echo "  make test            Tous les tests workspace"
	@echo "  make unit            Tests unitaires core"
	@echo "  make integration     Tests d'intégration"
	@echo "  make check           Clippy + fmt"
	@echo "  make coverage        Rapport tarpaulin (HTML)"
	@echo ""
	@echo "$(GREEN)── BUILD BINDINGS (Python / C / Excel) ──$(NC)"
	@echo "  make build-core      Compile ftp-calculator-core"
	@echo "  make build-c-bindings  .so/.dll pour l'add-in Excel"
	@echo "  make build-py-bindings Wheel Python (maturin)"
	@echo ""
	@echo "$(GREEN)── DOCUMENTATION ──$(NC)"
	@echo "  make docs-serve      Serveur MkDocs local (port 8000)"
	@echo "  make docs-deploy     Déploiement GitHub Pages"
	@echo ""
	@echo "$(GREEN)── MAINTENANCE ──$(NC)"
	@echo "  make setup-dev       Installe tous les outils de développement"
	@echo "  make clean           Supprime artefacts build"
	@echo "  make ci              Pipeline CI local (check + test)"
	@echo ""
	@echo "$(CYAN)Fichiers de configuration :$(NC)"
	@echo "  .env.dev             Variables d'env de développement (DB locale)"
	@echo "  .env.prod.example    Template pour la production (copier → .env.prod)"
	@echo "  data/dev/seed.sql    Données de démonstration (10 positions, 3 courbes)"
	@echo ""

# ── Alias de rétrocompatibilité ────────────────────────────────────────────────
docs:       docs-serve
docs-serve: docs-serve
deploy:     prod-deb
build-all:  build-core build-c-bindings build-py-bindings
setup:      setup-dev

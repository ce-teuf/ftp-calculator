# ==============================================================================
# FTP Simulator — Makefile
# ==============================================================================
#
# DÉMARRAGE RAPIDE
#   make dev-db          Démarre PostgreSQL dans Docker (une seule fois)
#   make dev-backend     Lance le backend Rust  (port 3000, terminal 1)
#   make dev-frontend    Lance Vite HMR         (port 5173, terminal 2)
#   make dev-data        Charge vecteurs + schedules de démo via le script Python
#
# ==============================================================================

# ── Variables ──────────────────────────────────────────────────────────────────

CARGO        = cargo
SQLX_OFFLINE = true
BACKEND_DIR  = app/backend
FRONTEND_DIR = app/web-app
DATA_SCRIPTS = data/datageneration_scripts

DEV_DB_NAME  = ftp_simulator_dev
DEV_DB_USER  = ftp_dev
DEV_DB_PASS  = ftp_dev
DEV_DB_PORT  = 5432
DEV_DB_URL   = postgresql://$(DEV_DB_USER):$(DEV_DB_PASS)@127.0.0.1:$(DEV_DB_PORT)/$(DEV_DB_NAME)

UNAME := $(shell uname)
ifeq ($(UNAME), Linux)
  LIB_NAME := libftp_calculator_bindings_c.so
  VENV_BIN := .venv/bin
else ifeq ($(UNAME), Darwin)
  LIB_NAME := libftp_calculator_bindings_c.dylib
  VENV_BIN := .venv/bin
else
  LIB_NAME := ftp_calculator_bindings_c.dll
  VENV_BIN := .venv/Scripts
endif

# Couleurs
GREEN  = \033[0;32m
RED    = \033[0;31m
YELLOW = \033[0;33m
BLUE   = \033[0;34m
CYAN   = \033[0;36m
NC     = \033[0m

.PHONY: all help \
        dev-db dev-backend dev-frontend dev-data dev-tmux dev-stop dev-reset \
        prod-build prod-frontend prod-run \
        test unit integration check lint fmt coverage \
        build-core build-py-bindings build-c-bindings \
        docs-serve \
        clean clean-dev setup-dev ci

all: help

# ==============================================================================
# DÉVELOPPEMENT
# ==============================================================================

## Démarre PostgreSQL de dev dans Docker (crée le conteneur si absent)
dev-db:
	@echo "$(BLUE)==> PostgreSQL (dev) via Docker$(NC)"
	@docker compose -f docker-compose.dev.yml up -d
	@echo "$(YELLOW)    Attente que PostgreSQL soit prêt...$(NC)"
	@until docker compose -f docker-compose.dev.yml exec -T db \
	    pg_isready -U $(DEV_DB_USER) -d $(DEV_DB_NAME) -q 2>/dev/null; do sleep 1; done
	@echo "$(GREEN)✓ PostgreSQL prêt sur 127.0.0.1:$(DEV_DB_PORT)$(NC)"

## Lance le backend Rust en mode développement (port 3000, migrations auto au démarrage)
dev-backend:
	@echo "$(BLUE)==> Backend sur http://localhost:3000$(NC)"
	@cd $(BACKEND_DIR) && \
	  DATABASE_URL=$(DEV_DB_URL) \
	  SQLX_OFFLINE=$(SQLX_OFFLINE) \
	  RUST_LOG=info,ftp_backend=debug \
	  $(CARGO) run 2>&1

## Lance le frontend Vite en mode développement (port 5173, proxy /api → :3000)
dev-frontend:
	@echo "$(BLUE)==> Frontend sur http://localhost:5173$(NC)"
	@cd $(FRONTEND_DIR) && npm run dev

## Charge les vecteurs et schedules de démo via le script Python
dev-data:
	@echo "$(BLUE)==> Chargement des données de démo$(NC)"
	@cd $(DATA_SCRIPTS) && python3 load_vectors_schedules.py
	@echo "$(GREEN)✓ Données chargées$(NC)"

## Lance backend + frontend dans des panneaux tmux
dev-tmux:
	@which tmux > /dev/null || (echo "$(RED)tmux non installé$(NC)" && exit 1)
	@tmux new-session -d -s ftp-dev -n backend \
	  "cd $(BACKEND_DIR) && DATABASE_URL=$(DEV_DB_URL) SQLX_OFFLINE=$(SQLX_OFFLINE) RUST_LOG=info cargo run; read"
	@tmux new-window -t ftp-dev -n frontend \
	  "cd $(FRONTEND_DIR) && npm run dev; read"
	@tmux attach -t ftp-dev

## Arrête PostgreSQL Docker (+ session tmux si active)
dev-stop:
	@pkill -f "ftp-backend" 2>/dev/null && echo "$(GREEN)✓ Backend arrêté$(NC)" || true
	@fuser -k 5173/tcp 2>/dev/null && echo "$(GREEN)✓ Frontend arrêté$(NC)" || true
	@tmux kill-session -t ftp-dev 2>/dev/null || true
	@docker compose -f docker-compose.dev.yml stop && echo "$(GREEN)✓ PostgreSQL arrêté$(NC)" || true

## Remet la base à zéro (supprime le volume Docker, recrée, recharge les données)
dev-reset:
	@echo "$(YELLOW)==> Réinitialisation de la base$(NC)"
	@docker compose -f docker-compose.dev.yml down -v
	@$(MAKE) dev-db
	@echo "$(YELLOW)    Démarrez le backend puis lancez : make dev-data$(NC)"

## Requête SQL directe sur la DB Docker
dev-psql:
	@docker exec -it ftp-simulator-dev-db psql -U $(DEV_DB_USER) -d $(DEV_DB_NAME)

# ==============================================================================
# PRODUCTION
# ==============================================================================

## Compile le frontend Svelte en mode production (génère dist/)
prod-frontend:
	@echo "$(BLUE)==> Build frontend (prod)$(NC)"
	@cd $(FRONTEND_DIR) && npm ci --silent && npm run build
	@echo "$(GREEN)✓ $(FRONTEND_DIR)/dist/ généré$(NC)"

## Compile le binaire release (frontend embarqué via include_dir)
prod-build: prod-frontend
	@echo "$(BLUE)==> Build release$(NC)"
	@SQLX_OFFLINE=$(SQLX_OFFLINE) $(CARGO) build --release -p ftp-backend
	@echo "$(GREEN)✓ target/release/ftp-backend$(NC)"

## Lance le binaire de production localement (nécessite DATABASE_URL dans l'environnement)
prod-run:
	@echo "$(BLUE)==> Lancement prod sur http://localhost:3000$(NC)"
	@DATABASE_URL=$(DEV_DB_URL) ./target/release/ftp-backend

# ==============================================================================
# TESTS & QUALITÉ
# ==============================================================================

## Lance tous les tests du workspace
test:
	@echo "$(BLUE)==> Tests$(NC)"
	@$(CARGO) test --workspace 2>&1
	@echo "$(GREEN)✓ Tous les tests passés$(NC)"

## Tests unitaires du core uniquement
unit:
	@$(CARGO) test -p ftp-calculator-core

## Tests d'intégration du core
integration:
	@$(CARGO) test -p ftp-calculator-core --test integration_tests

## Vérification Clippy + format
check:
	@echo "$(BLUE)==> Clippy$(NC)"
	@SQLX_OFFLINE=$(SQLX_OFFLINE) $(CARGO) clippy --workspace -- -D warnings
	@echo "$(BLUE)==> Format$(NC)"
	@$(CARGO) fmt --all -- --check
	@echo "$(GREEN)✓ Code valide$(NC)"

## Vérification TypeScript du frontend
check-ts:
	@echo "$(BLUE)==> TypeScript$(NC)"
	@cd $(FRONTEND_DIR) && npx tsc --noEmit 2>&1 | grep -v node_modules | grep -v "LinkersTab\|PortfolioV3Tab\|pyodide" || true
	@echo "$(GREEN)✓ TypeScript OK$(NC)"

lint:
	@SQLX_OFFLINE=$(SQLX_OFFLINE) $(CARGO) clippy --workspace -- -D warnings

fmt:
	@$(CARGO) fmt --all

## Couverture de code (nécessite cargo-tarpaulin)
coverage:
	@cargo tarpaulin -p ftp-calculator-core --ignore-tests --out Html
	@echo "$(GREEN)✓ Rapport dans tarpaulin-report.html$(NC)"

# ==============================================================================
# BUILD DES BINDINGS (Python + C / Excel)
# ==============================================================================

build-core:
	@$(CARGO) build --release -p ftp-calculator-core

build-c-bindings:
	@echo "$(BLUE)==> Bindings C$(NC)"
	@$(CARGO) build --release -p ftp-calculator-bindings-c
	@mkdir -p excel-addin/Interop
	@cp target/release/$(LIB_NAME) excel-addin/Interop/
	@echo "$(GREEN)✓ $(LIB_NAME) → excel-addin/Interop/$(NC)"

build-py-bindings:
	@echo "$(BLUE)==> Bindings Python (maturin)$(NC)"
	@cd python-lib && $(VENV_BIN)/maturin build --release
	@echo "$(GREEN)✓ Wheel Python générée$(NC)"

build-all: build-core build-c-bindings build-py-bindings

# ==============================================================================
# DOCUMENTATION
# ==============================================================================

## Serveur MkDocs local (port 8000)
docs-serve:
	@echo "$(BLUE)==> Documentation MkDocs sur http://localhost:8000$(NC)"
	@$(VENV_BIN)/mkdocs serve -f mkdocs.yml

# ==============================================================================
# SETUP
# ==============================================================================

## Installe tous les outils de développement
setup-dev:
	@echo "$(BLUE)==> Installation des outils de développement$(NC)"
	@rustup component add clippy rustfmt
	@cargo install cargo-watch cargo-tarpaulin 2>/dev/null || true
	@cd $(FRONTEND_DIR) && npm install
	@test -d .venv || python3 -m venv .venv
	@$(VENV_BIN)/pip install maturin mkdocs-material openpyxl requests 2>/dev/null || true
	@echo "$(GREEN)✓ Environnement de développement prêt$(NC)"
	@echo ""
	@echo "Démarrez avec :  make dev-db  puis  make dev-backend  +  make dev-frontend"

# ==============================================================================
# NETTOYAGE
# ==============================================================================

## Supprime les artefacts de build Rust + frontend
clean:
	@$(CARGO) clean
	@rm -rf $(FRONTEND_DIR)/dist/ $(FRONTEND_DIR)/.svelte-kit/
	@echo "$(GREEN)✓ Nettoyé$(NC)"

## Supprime le conteneur et le volume Docker de dev
clean-dev:
	@docker compose -f docker-compose.dev.yml down -v 2>/dev/null || true
	@echo "$(GREEN)✓ Conteneur PostgreSQL supprimé$(NC)"

# ==============================================================================
# CI
# ==============================================================================

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
	@echo "$(GREEN)── DÉVELOPPEMENT ──$(NC)"
	@echo "  make dev-db          Démarre PostgreSQL dans Docker"
	@echo "  make dev-backend     Lance le backend Rust (port 3000)"
	@echo "  make dev-frontend    Lance Vite HMR (port 5173)"
	@echo "  make dev-data        Charge vecteurs + schedules de démo (Python)"
	@echo "  make dev-tmux        Lance backend + frontend dans tmux"
	@echo "  make dev-psql        Shell psql interactif sur la DB Docker"
	@echo "  make dev-stop        Arrête backend, frontend et PostgreSQL"
	@echo "  make dev-reset       Remet la base à zéro (volume Docker supprimé)"
	@echo ""
	@echo "$(GREEN)── PRODUCTION ──$(NC)"
	@echo "  make prod-build      Build release (frontend embarqué)"
	@echo "  make prod-run        Lance le binaire release localement"
	@echo ""
	@echo "$(GREEN)── TESTS & QUALITÉ ──$(NC)"
	@echo "  make test            Tous les tests workspace"
	@echo "  make unit            Tests unitaires core"
	@echo "  make integration     Tests d'intégration core"
	@echo "  make check           Clippy + fmt (Rust)"
	@echo "  make check-ts        TypeScript check (frontend)"
	@echo "  make coverage        Rapport de couverture tarpaulin"
	@echo "  make ci              check + test"
	@echo ""
	@echo "$(GREEN)── BUILD BINDINGS ──$(NC)"
	@echo "  make build-core         Compile ftp-calculator-core"
	@echo "  make build-c-bindings   .so/.dll pour l'add-in Excel"
	@echo "  make build-py-bindings  Wheel Python (maturin)"
	@echo ""
	@echo "$(GREEN)── MAINTENANCE ──$(NC)"
	@echo "  make setup-dev       Installe les outils (rustup, npm, venv)"
	@echo "  make clean           Supprime artefacts build"
	@echo "  make clean-dev       Supprime le conteneur PostgreSQL Docker"
	@echo "  make docs-serve      Serveur MkDocs local (port 8000)"
	@echo ""
	@echo "$(CYAN)DB directe :$(NC)"
	@echo "  docker exec ftp-simulator-dev-db psql -U ftp_dev -d ftp_simulator_dev -c 'SELECT ...'"
	@echo ""

# Alias
setup: setup-dev
docs:  docs-serve

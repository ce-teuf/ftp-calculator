# Variables
CARGO = cargo
MATURIN = maturin
DOTNET = dotnet
TEST_FLAGS = --color=always
NOCAPTURE_FLAGS = -- --nocapture
BENCH_FLAGS = -- --bench
PROJECT_NAME = $(shell grep '^name' Cargo.toml | head -1 | cut -d '"' -f 2)

# Chemins des projets
CRATES_DIR = crates
CORE_DIR = $(CRATES_DIR)/ftp_core
C_BINDINGS_DIR = $(CRATES_DIR)/ftp_core_bindings_c
PY_BINDINGS_DIR = $(CRATES_DIR)/ftp_core_bindings_pyo3
EXCEL_DIR = excel-addin
DOCS_DIR = docs
SCRIPTS_DIR = scripts

# Couleurs pour l'affichage
GREEN = \033[0;32m
RED = \033[0;31m
YELLOW = \033[0;33m
BLUE = \033[0;34m
NC = \033[0m

.PHONY: all test unit integration detailed clean help bench doc tarpaulin docs \
        build-c-bindings build-py-bindings build-docs build-excel-addin \
        deploy-docs setup-dev

# Cible par défaut
all: test

# ============================================================================ #
# TESTS
# ============================================================================ #

# Tous les tests
test: unit integration
	@echo "$(GREEN)✓ Tous les tests sont passés !$(NC)"

# Tests unitaires seulement
unit:
	@echo "$(BLUE)Exécution des tests unitaires...$(NC)"
	@$(CARGO) test $(TEST_FLAGS)

# Tests d'intégration seulement

integration:
	@echo "$(BLUE)Exécution des tests d'intégration...$(NC)"
	@$(CARGO) test --test integration_tests $(TEST_FLAGS)
	@$(CARGO) test --test matrix_operations $(TEST_FLAGS)

# Tests avec output détaillé
detailed:
	@echo "$(BLUE)Exécution des tests avec output détaillé...$(NC)"
	@$(CARGO) test $(NOCAPTURE_FLAGS)

# ============================================================================ #
# BUILD DES BINDINGS ET ARTEFACTS
# ============================================================================ #

UNAME := $(shell uname)

ifeq ($(UNAME), Linux)
LIB_NAME := libftp_core_bindings_c.so
VENV_BIN := .venv/bin
else ifeq ($(UNAME), Darwin)
LIB_NAME := libftp_core_bindings_c.dylib
VENV_BIN := .venv/bin
else
LIB_NAME := ftp_core_bindings_c.dll
VENV_BIN := .venv/Scripts
endif

# Builder les bindings C (pour Excel)
build-c-bindings:
	@echo "$(BLUE)Construction des bindings C...$(NC)"
	@cd $(C_BINDINGS_DIR) && $(CARGO) build --release
	@echo "$(GREEN)✓ Bindings C construits$(NC)"
	@echo "$(BLUE)Copie des artefacts...$(NC)"
	@echo "$(GREEN)✓ Lib selectionnee: $(LIB_NAME)$(NC)"
	@mkdir -p ../excel-addin/Interop/
	@cp target/release/$(LIB_NAME) excel-addin/Interop/
	@echo "$(GREEN)✓ Artefacts copiés$(NC)"

# Builder les bindings Python
build-py-bindings:
	@echo "$(BLUE)Construction des bindings Python...$(NC)"
	@cd python-lib && ../$(VENV_BIN)/maturin build --release
	@echo "$(GREEN)✓ Bindings Python construits$(NC)"

# Builder l'add-in Excel (nécessite Windows et .NET)
build-excel-addin:
	@echo "$(BLUE)Construction de l'add-in Excel...$(NC)"
	@cd $(EXCEL_DIR)/ftp_addin && $(DOTNET) build --configuration Release
	@echo "$(GREEN)✓ Add-in Excel construit$(NC)"

# Builder tous les bindings
build-all: build-c-bindings build-py-bindings
	@echo "$(GREEN)✓ Tous les bindings construitsx$(NC)"

# ============================================================================ #
# DOCUMENTATION
# ============================================================================ #

# Builder la documentation locale
build_docs:
	@echo "$(BLUE)Construction de la documentation Rust...$(NC)"
	@$(CARGO) doc --target-dir $(DOCS_DIR)/site/rust-generated/ --workspace --no-deps --open
	@$(VENV_BIN)/python -m pdoc -o $(DOCS_DIR)/site/python-generated/ python-lib/src/

# Servir la documentation localement
serve-docs:
	@echo "$(BLUE)Lancement du serveur de documentation...$(NC)"
	@cd $(DOCS_DIR)/site/ && ../../$(VENV_BIN)/python -m http.server 8000 --bind 127.0.0.1


# Déployer la documentation (pour GitHub Pages)
deploy-docs:
	@echo "$(BLUE)Déploiement de la documentation...$(NC)"
	@cd $(DOCS_DIR) && mkdocs gh-deploy --force
	@echo "$(GREEN)✓ Documentation déployée$(NC)"

# Documentation Rust seulement
doc:
	@echo "$(BLUE)Génération de la documentation Rust...$(NC)"
	@$(CARGO) doc --open

# ============================================================================ #
# UTILITAIRES ET MAINTENANCE
# ============================================================================ #

# Nettoyage complet
clean:
	@echo "$(YELLOW)Nettoyage du projet...$(NC)"
	@$(CARGO) clean
	@rm -rf target
	@rm -rf $(DOCS_DIR)/site/python-generated
	@rm -rf $(DOCS_DIR)/site/rust-generated
	@rm -rf $(EXCEL_DIR)/Interop/*
	@echo "$(GREEN)✓ Projet nettoyé$(NC)"


# Installation des dépendances de développement
setup-dev:
	@echo "$(BLUE)Installation des outils de développement...$(NC)"
	@rustup component add clippy
	@rustup component add rustfmt
	@cargo install cargo-tarpaulin 2>/dev/null || echo "$(YELLOW)tarpaulin déjà installé ou échec d'installation$(NC)"
	@test -d .venv || python -m venv .venv
	@$(VENV_BIN)/pip install maturin mkdocs-material mkdocstrings mkdocstrings-python pdoc 2>/dev/null || echo "$(YELLOW)Outils Python déjà installés$(NC)"
	@cd python-lib && ../$(VENV_BIN)/pip install -e ".[dev]"
	@echo "$(GREEN)✓ Environnement de développement configuré$(NC)"

# Vérification du code sans exécution des tests
check:
	@echo "$(BLUE)Vérification du code...$(NC)"
	@$(CARGO) check
	@$(CARGO) clippy
	@$(CARGO) fmt --all -- --check

# ============================================================================ #
# BENCHMARKS ET ANALYSE
# ============================================================================ #

# Benchmarks (si vous en avez)
bench:
	@echo "$(BLUE)Exécution des benchmarks...$(NC)"
	@$(CARGO) bench $(BENCH_FLAGS)

# Couverture de code avec tarpaulin
tarpaulin:
	@echo "$(BLUE)Vérification de l'installation de tarpaulin...$(NC)"
	@which cargo-tarpaulin > /dev/null 2>&1 || (echo "$(RED)Tarpaulin n'est pas installé. Installation...$(NC)" && cargo install cargo-tarpaulin)
	@echo "$(BLUE)Génération du rapport de couverture...$(NC)"
	@cargo tarpaulin --ignore-tests --out Html

# ============================================================================ #
# CIBLES SPÉCIFIQUES ET DÉBOGAGE
# ============================================================================ #

# Test avec filtrage (pour exécuter un test.yml spécifique)
test-%:
	@echo "$(BLUE)Exécution des tests correspondant à '$*'...$(NC)"
	@$(CARGO) test --test $* $(TEST_FLAGS)

# Version verbose pour le débogage
verbose: TEST_FLAGS += --verbose
verbose: test

# Version release pour les tests de performance
release-test:
	@echo "$(BLUE)Exécution des tests en mode release...$(NC)"
	@$(CARGO) test --release $(TEST_FLAGS)

# Installation de toutes les dépendances (setup complet)
setup: setup-dev
	@echo "$(BLUE)Installation des dépendances Rust...$(NC)"
	@$(CARGO) fetch
	@echo "$(GREEN)✓ Setup complet terminé$(NC)"

# ============================================================================ #
# CI/CD SIMULATION
# ============================================================================ #

# Simulation du pipeline CI
ci: check test build-all build-docs
	@echo "$(GREEN)✓ Pipeline CI simulé avec succès$(NC)"

# Simulation du pipeline de release
release: test build-all tarpaulin
	@echo "$(GREEN)✓ Pipeline de release simulé avec succès$(NC)"

# ============================================================================ #
# AIDE
# ============================================================================ #

help:
	@echo "$(GREEN)Makefile pour le projet $(PROJECT_NAME)$(NC)"
	@echo ""
	@echo "Cibles disponibles:"
	@echo ""
	@echo "  $(GREEN)TEST$(NC)"
	@echo "  all        - Exécute tous les tests (cible par défaut)"
	@echo "  test       - Tests unitaires et d'intégration"
	@echo "  unit       - Tests unitaires seulement"
	@echo "  integration- Tests d'intégration seulement"
	@echo "  detailed   - Tests avec output détaillé"
	@echo "  check      - Vérification du code (clippy + fmt)"
	@echo ""
	@echo "  $(GREEN)BUILD$(NC)"
	@echo "  build-c-bindings - Construit les bindings C (Excel)"
	@echo "  build-py-bindings- Construit les bindings Python"
	@echo "  build-excel      - Construit l'add-in Excel (.NET)"
	@echo "  build-all        - Construit tous les bindings"
	@echo ""
	@echo "  $(GREEN)DOCUMENTATION$(NC)"
	@echo "  build-docs  - Construit la documentation MkDocs"
	@echo "  serve-docs  - Lance le serveur de documentation"
	@echo "  deploy-docs - Déploie sur GitHub Pages"
	@echo "  doc         - Documentation Rust uniquement"
	@echo ""
	@echo "  $(GREEN)ANALYSE$(NC)"
	@echo "  bench      - Benchmarks"
	@echo "  tarpaulin  - Rapport de couverture de code"
	@echo ""
	@echo "  $(GREEN)MAINTENANCE$(NC)"
	@echo "  clean      - Nettoyage complet"
	@echo "  setup-dev  - Installe les outils de développement"
	@echo "  setup      - Setup complet du projet"
	@echo ""
	@echo "  $(GREEN)CI/CD$(NC)"
	@echo "  ci         - Simulation du pipeline CI"
	@echo "  release    - Simulation du pipeline de release"
	@echo ""
	@echo "  $(GREEN)DÉBOGAGE$(NC)"
	@echo "  verbose    - Tests en mode verbeux"
	@echo "  release-test - Tests en mode release"
	@echo "  test-*     - Tests spécifiques (ex: test-integration_tests)"
	@echo ""
	@echo "Exemples:"
	@echo "  make                       # Exécute tous les tests"
	@echo "  make build-all             # Construit tous les bindings"
	@echo "  make ci                    # Simulation CI complète"
	@echo "  make build-docs && serve-docs # Documentation locale"

# Gestion des releases
release-prepare:
	@echo "$(BLUE)Préparation de la release...$(NC)"
	@$(VENV_BIN)/python scripts/release.py prepare

release-version:
	@echo "$(BLUE)Affichage de la version...$(NC)"
	@$(VENV_BIN)/python scripts/release.py version

release-bump-patch:
	@echo "$(BLUE)Création d'une release patch...$(NC)"
	@$(VENV_BIN)/python scripts/release.py release --bump patch

release-bump-minor:
	@echo "$(BLUE)Création d'une release mineure...$(NC)"
	@$(VENV_BIN)/python scripts/release.py release --bump minor

release-bump-major:
	@echo "$(BLUE)Création d'une release majeure...$(NC)"
	@$(VENV_BIN)/python scripts/release.py release --bump major

release-dry-run:
	@echo "$(BLUE)Simulation d'une release patch (dry-run)...$(NC)"
	@$(VENV_BIN)/python scripts/release.py release --bump patch --dry-run

# Alias
release: release-bump-patch

# Alias pour la rétrocompatibilité
docs: build-docs
docs-serve: serve-docs
docs-deploy: deploy-docs
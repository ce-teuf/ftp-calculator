# FTP Simulator

Full-stack **Funds Transfer Pricing (FTP) simulator** for ALM/Treasury teams.

Computation engine in **Rust**, web interface in **SvelteKit 5**, database **PostgreSQL 18**.

> **Web application**: the simulator UI is under ongoing construction. Core computation engine, API, and bindings are stable.

---

## Repository layout

```
ftp-calculator/
├── app/
│   ├── backend/          # REST API — Rust / Axum / SQLx
│   └── web-app/          # SvelteKit 5 frontend (Vite, ECharts, Lucide)
├── crates-core/
│   ├── ftp-calculator-core/           # FTP computation engine (Rust)
│   ├── ftp-calculator-bindings-c/     # C bindings → Excel Add-In
│   └── ftp-calculator-bindings-pyo3/  # Python bindings (PyO3)
├── python-lib/           # ftp_calculator Python package (wraps PyO3 bindings)
├── excel-addin/          # Excel-DNA Add-In (.NET / C#)
├── data/
│   └── datageneration_scripts/   # Demo data loader (Python)
├── docs/                 # Internal documentation
├── docker-compose.dev.yml
├── Makefile
└── mkdocs.yml
```

---

## FTP methods implemented

| Method | Description |
|---|---|
| **Stock** | Stock amortization profiles — prices the existing book by extracting variable-stock installments via the anti-diagonal; CoF locked at each cohort's origination period |
| **Flux** | New-production method — derives new originations from the stock evolution, applies production amortization profiles, locks CoF at origination; gives the marginal FTP rate per cohort |

Both methods operate at **cohort level**: the outstanding vector represents the observed portfolio stock; each row corresponds to an observation period. The aggregate FTP rate at any point is the installment-weighted average of all active cohorts' locked-in rates.

---

## Application modules

| # | Module | Frontend route | Backend API |
|---|--------|----------------|-------------|
| 1 | Rate matrices | `/rate-matrices` | `/api/rate-matrices`, `/api/risk-types` |
| 2 | Hypercubes | `/hypercubes` | `/api/hypercubes` |
| 3 | Portfolios | `/portfolios` | `/api/portfolios`, `/api/outstanding-vectors`, `/api/amort-schedules` |
| 4 | Study units | `/study-units` | `/api/study-units` |
| 5 | Studies | `/studies` | `/api/studies` |
| 6 | Executions | `/executions` | `/api/executions` |

---

## Quick start — Development

**Prerequisites:** Rust stable, Node.js 20+, Docker

```bash
# 1. Start PostgreSQL in Docker
make dev-db

# 2. Start the backend (terminal 1)
make dev-backend      # → http://localhost:3000

# 3. Start the frontend with HMR (terminal 2)
make dev-frontend     # → http://localhost:5173

# Load demo data (vectors + schedules) after the backend is running
make dev-data

# Or run backend + frontend together in tmux
make dev-tmux
```

In dev mode:
- PostgreSQL runs in Docker (`docker-compose.dev.yml`) — no local installation required
- Backend and frontend run locally with auto-reload
- Frontend proxies `/api` → `http://localhost:3000/api`
- Demo data loaded via `data/datageneration_scripts/load_vectors_schedules.py`

**DB connection string:**
```
postgresql://ftp_dev:ftp_dev@127.0.0.1:5432/ftp_simulator_dev
```

Query the DB directly:
```bash
docker exec ftp-simulator-dev-db psql -U ftp_dev -d ftp_simulator_dev -c "SELECT ..."
# or interactively:
make dev-psql
```

---

## Quick start — Production

```bash
# Build the frontend, then compile the release binary (frontend embedded via include_dir)
make prod-build       # → target/release/ftp-backend

# Run the release binary locally (requires DATABASE_URL)
make prod-run
```

---

## Makefile reference

### Development

| Command | Description |
|---|---|
| `make dev-db` | Start PostgreSQL container (creates it if absent) |
| `make dev-backend` | Run Rust backend (port 3000, auto-migrations on start) |
| `make dev-frontend` | Run Vite HMR (port 5173, proxies /api) |
| `make dev-data` | Load demo vectors + schedules via Python script |
| `make dev-tmux` | Run backend + frontend in tmux panes |
| `make dev-psql` | Interactive psql shell on the Docker DB |
| `make dev-stop` | Stop backend, frontend, and PostgreSQL |
| `make dev-reset` | Drop and recreate the DB volume |

### Production

| Command | Description |
|---|---|
| `make prod-build` | Build frontend + release binary (frontend embedded) |
| `make prod-run` | Run release binary locally |

### Tests & quality

| Command | Description |
|---|---|
| `make test` | All workspace tests |
| `make unit` | Core unit tests only |
| `make integration` | Core integration tests |
| `make check` | Clippy + rustfmt check |
| `make check-ts` | TypeScript check (frontend) |
| `make coverage` | Tarpaulin coverage report (HTML) |
| `make ci` | check + test |

### Bindings (Python / C / Excel)

| Command | Description |
|---|---|
| `make build-core` | Compile `ftp-calculator-core` |
| `make build-c-bindings` | `.so` / `.dll` for the Excel Add-In |
| `make build-py-bindings` | Python wheel (maturin) |

---

## Python bindings

```python
from ftp_calculator import FtpCalculator

calc = FtpCalculator(outstanding, profiles, rates)
calc.compute("stock")   # or "flux"
print(calc.ftp_rate)
```

```bash
make build-py-bindings   # generates the Python wheel
```

---

## Excel Add-In

The Excel Add-In (Excel-DNA, .NET) exposes the engine as worksheet functions:

- `FTP_COMPUTE_STOCK(outstanding, profiles, rates)` — run the stock method
- `FTP_COMPUTE_FLUX(outstanding, profiles, rates)` — run the flux method
- Individual getters: `FTP_STOCK_AMORT`, `FTP_FTP_RATE`, `FTP_FTP_INT`, etc.

The native library (`.so` / `.dll`) is built with `make build-c-bindings` and placed under `excel-addin/Interop/`.

---

## DB schema

Migrations run automatically at backend startup (`app/backend/src/db/migrations/`):

| File | Content |
|---|---|
| `001_init.sql` | `risk_types`, `rate_matrices` |
| `002_hypercubes.sql` | `hypercubes`, `hypercube_matrices` |
| `003_portfolios.sql` | `portfolios`, `outstanding_vectors`, `amort_schedules`, join tables |
| `004_study_units.sql` | `study_units`, `study_unit_assignments` |
| `005_studies.sql` | `studies`, `study_study_units` |
| `006_executions.sql` | `executions`, `execution_results` |

---

## Development setup from scratch

```bash
make setup-dev   # installs rustup components, npm deps, Python venv + maturin
```

---

## License

MIT OR Apache-2.0

## Author

Charles Teuf

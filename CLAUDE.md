# FTP Simulator — Claude Code Reference

## Project overview

Full-stack FTP (Funds Transfer Pricing) simulator for ALM/Treasury teams.

- **Backend**: Rust (Axum + SQLx + PostgreSQL) — `app/backend/`
- **Frontend**: SvelteKit 5 (runes, ECharts, Lucide) — `app/web-app/`
- **Core lib**: Rust crate with C/Python bindings — `crates-core/`
- **Dev DB**: PostgreSQL 18 in Docker (`ftp-simulator-dev-db`)

## Dev environment

```bash
make dev-db        # start PostgreSQL container (must be running first)
make dev-backend   # cargo run on port 3000 (in a terminal)
make dev-frontend  # npm run dev on port 5173 (in another terminal)
```

Frontend proxies `/api` → `http://localhost:3000/api`.

DB: `postgresql://ftp_dev:ftp_dev@127.0.0.1:5432/ftp_simulator_dev`

Query DB directly:
```bash
docker exec ftp-simulator-dev-db psql -U ftp_dev -d ftp_simulator_dev -c "SELECT ..."
```

Reload test data (vectors + schedules):
```bash
cd data/datageneration_scripts && python3 load_vectors_schedules.py
```

## Architecture — modules

| # | Module | Frontend route | Backend API |
|---|--------|---------------|-------------|
| 1 | Rate matrices | `/rate-matrices` | `/api/rate-matrices`, `/api/risk-types` |
| 2 | Hypercubes | `/hypercubes` | `/api/hypercubes` |
| 3 | Portfolios | `/portfolios` | `/api/portfolios`, `/api/outstanding-vectors`, `/api/amort-schedules` |
| 4 | Study units | `/study-units` | `/api/study-units` |
| 5 | Studies | `/studies` | `/api/studies` |
| 6 | Executions | `/executions` | `/api/executions` |

## Key domain concepts

- **Rate matrix**: sparse tenor grid (e.g. 1M/3M/6M/1Y) interpolated at query time (linear/cubic/flat_forward)
- **Outstanding vector**: time series of loan balances per month (observed + projected), uploaded as XLSX with `date_month | period_type | value`
- **Amortization schedule**: full monthly repayment profile per origination date — XLSX with `date_month | period_type | 1 | 2 | … | N` (N = max maturity months, sum per row ≈ 1.0). NOT sparse tenors — all months required.
- **Portfolio pair**: (vector, schedule) couple used as FTP calculation input
- **Study unit**: one computation unit — one portfolio pair + one hypercube + parameters
- **Study**: collection of study units, produces execution inputs
- **Execution**: runs the FTP engine, writes results to DB

## DB schema files

`app/backend/src/db/migrations/`:
- `001_init.sql` — risk_types, rate_matrices
- `002_hypercubes.sql` — hypercubes, hypercube_matrices
- `003_portfolios.sql` — portfolios, outstanding_vectors, amort_schedules, portfolio_vectors/schedules/pairs
- `004_study_units.sql` — study_units, study_unit_assignments
- `005_studies.sql` — studies, study_study_units
- `006_executions.sql` — executions, execution_results

Migrations run automatically at backend startup via SQLx.

## Frontend conventions

- **Svelte 5 runes**: `$state`, `$effect`, `$derived` — no `$:` reactive statements
- **ECharts**: always `tick().then(() => { ... })` before `echarts.init()` on a `bind:this` element; dispose on cleanup with `return () => { chart.dispose() }`
- **API client**: `src/lib/api/client.ts` — typed functions, all routes defined there
- **Icons**: `@lucide/svelte` — `import { Plus, X, ... } from '@lucide/svelte'`
- **Styles**: scoped `<style>` per component, shared vars in `app.css`

## Backend conventions

- API handlers in `app/backend/src/api/*.rs`, registered in `mod.rs`
- IDs are `TEXT` (UUID strings generated in Rust)
- JSON stored as `TEXT` columns (not `JSONB`) — deserialised in application layer
- `tenor_to_months()` in `compute/interpolate.rs` handles both named tenors ("1M", "3Y") and plain numbers ("1", "24", "120")
- Period types: `observed` | `projected` | `contrafactual`

## TypeScript check

```bash
cd app/web-app && npx tsc --noEmit 2>&1 | grep -v node_modules
```

Known pre-existing errors (ignore):
- `pyodide.worker.js` — implicit `any` in .js file
- `LinkersTab.svelte`, `PortfolioV3Tab.svelte` — reference removed exports (legacy files)

## Cargo workspace

Root `Cargo.toml` — members: `app/backend`, `crates-core/*`, `python-lib`, `excel-addin`

```bash
SQLX_OFFLINE=true cargo check -p ftp-backend   # check backend only
SQLX_OFFLINE=true cargo build -p ftp-backend   # build backend
cargo test -p ftp-calculator-core              # core unit tests
```

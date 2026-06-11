# Performance Tests

Docker-based benchmark harness that runs all 13 Petstore language implementations against the shared PostgreSQL instance and measures RPS, latency percentiles, CPU, RAM, and Postgres statistics.

## Prerequisites

| Tool | Purpose |
|---|---|
| Docker (Unix socket at `/var/run/docker.sock`) | Builds and runs stack containers; runs k6 as a container |
| Python 3 (stdlib only) | `sampler.py` and `report.py` |
| `jq` | JSON parsing in `pg-stats.sh` |
| `psql` client (`postgresql-client`) | Postgres snapshot/reset in `pg-stats.sh` |
| k6 | Pulled automatically as `grafana/k6:latest` — no local install required |

The shared Postgres container (`speed-test-postgres`) must be running before any run.

## Quick Start

```bash
# 1. Start the database
cd database
docker compose up -d
./create-databases.sh
./apply-schemas.sh

# 2. Smoke-test a single stack
cd performance-tests
VUS=3 DURATION=15s ./run.sh go naive

# 3. Generate a comparison report
python3 report.py
```

## Full Benchmark Run

```bash
# All stacks, both variants
VUS=20 DURATION=60s ./run.sh all

# All stacks, one variant
VUS=20 DURATION=60s ./run.sh all naive
VUS=20 DURATION=60s ./run.sh all optimized

# Single stack, both variants
VUS=20 DURATION=60s ./run.sh go

# Comma-separated stacks
VUS=20 DURATION=60s ./run.sh go,springboot,python optimized
```

Stack IDs (from `stacks.json`): `go`, `springboot`, `helidon`, `quarkus`, `nodejs`, `python`, `rust`, `csharp`, `laravel`, `rails`, `ktor`, `phoenix`.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `VUS` | `20` | k6 virtual users |
| `DURATION` | `60s` | k6 run duration (e.g. `30s`, `2m`) |
| `APP_CPUS` | `2` | CPU limit for the app container |
| `APP_MEMORY` | `512m` | Memory limit for the app container |
| `PGHOST` | `localhost` | Postgres host reachable from the **host** machine |
| `PGPORT` | `5434` | Postgres port on the host |
| `PGUSER` | `myuser` | Postgres user |
| `PGPASSWORD` | `mypassword` | Postgres password |
| `LARAVEL_APP_KEY` | — | Required for Laravel; see Per-Stack Notes below |
| `RAILS_SECRET_KEY_BASE` | — | Required for Rails optimized variant |
| `PHOENIX_SECRET_KEY_BASE` | — | Optional for Phoenix |
| `NO_BUILD` | `0` | Set to `1` to skip `docker build` and use existing images |
| `KEEP_RESULTS` | `0` | Set to `1` to keep the results directory on failure |

## Per-Stack Notes

### Python (FastAPI)

FastAPI's generated OAuth2 stub enforces a Bearer token at the framework level. `stacks.json` sets `"auth_header": "Bearer benchmark-token"`, which `run.sh` passes to k6 as the `AUTH_HEADER` env var. All k6 requests include `Authorization: Bearer benchmark-token`.

### Node.js (Express)

`express-openapi-validator` enforces the `api_key` security scheme on `GET /store/inventory`. k6's `crud.js` always sends `"api_key": "benchmark"` in headers.

### Laravel (PHP)

`APP_KEY` is required for Laravel to start. Set `LARAVEL_APP_KEY` before benchmarking:

```bash
# From inside the laravel project directory:
php artisan key:generate --show
# Copy the output (e.g. base64:abc123...) and export it:
export LARAVEL_APP_KEY="base64:abc123..."
VUS=20 DURATION=60s ./run.sh laravel
```

### Rails (Ruby)

- Runs as `RAILS_ENV=development` to avoid an ActionCable eager-load error in the generated production config.
- `stacks.json` sets `entrypoint_override` and `cmd_override` to bypass `docker-entrypoint.sh`, which calls `bin/rails db:migrate` — a step that fails because the generated app doesn't use ActiveRecord.
- `Dockerfile.optimized` uses production mode and may still hit the ActionCable issue; the same overrides apply.

### JVM stacks (Spring Boot, Helidon, Quarkus, Ktor, Phoenix)

Docker builds take 2–3 minutes (Maven/Gradle dependency download, compilation). These stacks have `"readiness_timeout": 120` in `stacks.json`. If a stack still times out, set `READINESS_TIMEOUT=180` before the run.

## Output Files

Each run produces a directory at `results/<stack>-<variant>-<timestamp>/`:

| File | Contents |
|---|---|
| `run-meta.json` | Stack id, variant, VUs, duration, CPU/memory limits, timestamp |
| `build.log` | Full `docker build` output |
| `docker-stats.csv` | 1-second-interval CPU %, RAM, network I/O, block I/O for the app container and the Postgres container |
| `pg-before.json` | Snapshot of `pg_stat_database`, `pg_stat_io`, and `pg_stat_statements` (top 50 by exec time) taken before k6 starts |
| `pg-after.json` | Same snapshot taken after k6 finishes |
| `pg-delta.json` | Field-by-field delta between before and after |
| `k6-summary.json` | k6 `--summary-export` output: RPS, latency (avg, p95, p99), error rate, request counts |
| `k6.log` | k6 stdout/stderr |
| `container.log` | App container logs (`docker logs`) |

`results/` is gitignored.

## Report

After one or more runs, generate a comparison:

```bash
python3 report.py
```

This reads every `results/*/run-meta.json` directory and writes two files:

- `results/comparison.csv` — machine-readable, one row per run
- `results/comparison.md` — Markdown table with columns: stack, variant, RPS, avg/p95/p99 latency (ms), error rate (%), CPU avg/peak (%), RAM avg/peak (MB), PG transactions, blocks read, blocks hit

Optional arguments:

```bash
python3 report.py --results-dir /path/to/results --output-csv out.csv --output-md out.md
```

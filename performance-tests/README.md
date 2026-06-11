# Petstore Performance Benchmark Harness

A language-agnostic Docker harness that benchmarks all 13 Petstore server implementations (Go, Java Spring Boot, Java Helidon, Java Quarkus, Node.js, Python, Rust, C# ASP.NET Core, PHP Laravel, Ruby Rails, Kotlin Ktor, Elixir Phoenix) against the same PostgreSQL instance, collecting RPS, latency percentiles, CPU, RAM, and Postgres statistics.

Each stack ships a `Dockerfile` (**naive** — stock dependencies, no tuning) and a `Dockerfile.optimized` (**optimized** — multi-stage build, slim base image, JVM flags, connection pool tuning, etc.). Both variants are benchmarked so you can see the raw performance floor and what careful optimization buys.

---

## Stacks

| Stack ID | Language | Framework |
|---|---|---|
| `go` | Go | Gin |
| `springboot` | Java 17 | Spring Boot 3.3 |
| `helidon` | Java 21 | Helidon MP 4 |
| `quarkus` | Java 25 | Quarkus 3.36 (Gradle) |
| `nodejs` | Node.js | Express |
| `python` | Python | FastAPI |
| `rust` | Rust | hyper |
| `csharp` | C# | ASP.NET Core 8 |
| `laravel` | PHP | Laravel |
| `rails` | Ruby | Rails |
| `ktor` | Kotlin | Ktor |
| `phoenix` | Elixir | Phoenix |

---

## Prerequisites

| Tool | Install hint | Why |
|---|---|---|
| Docker (with `/var/run/docker.sock`) | [docker.com](https://docs.docker.com/engine/install/) | Builds and runs stack containers; k6 runs as a container — no local install needed |
| Python 3 (stdlib only) | system package | `sampler.py` (resource polling) and `report.py` (comparison table) |
| `jq` | `apt install jq` / `brew install jq` | Postgres stats JSON parsing in `pg-stats.sh` |
| `psql` client | `apt install postgresql-client` | Postgres snapshot/reset |
| Node.js 18+ *(optional)* | [nodejs.org](https://nodejs.org) | Only needed to run the visual results viewer |

---

## Quick Start

```bash
# 1. Start the shared Postgres container
cd /path/to/speed-test/database
docker compose up -d
./create-databases.sh   # idempotent — safe to re-run
./apply-schemas.sh

# 2. Smoke-test a single stack with low load (fast — ~30 seconds including build)
cd /path/to/speed-test/performance-tests
VUS=3 DURATION=15s ./run.sh go naive

# 3. Check the results
ls results/go-naive-*/

# 4. Generate a text comparison table
python3 report.py
```

---

## Running Benchmarks

All commands run from the `performance-tests/` directory.

### Single stack, one variant

```bash
./run.sh go naive
./run.sh python optimized
```

### Single stack, both variants

```bash
./run.sh go          # runs naive then optimized
```

### Multiple stacks

```bash
./run.sh go,springboot,python naive
./run.sh go,springboot,python optimized
```

### All stacks

```bash
# Both variants — takes ~1.5–2 hours at defaults (JVM stacks build slowly)
VUS=20 DURATION=60s ./run.sh all

# One variant only
VUS=20 DURATION=60s ./run.sh all naive
VUS=20 DURATION=60s ./run.sh all optimized
```

### Skip the Docker build (re-use existing images)

```bash
NO_BUILD=1 VUS=20 DURATION=60s ./run.sh go naive
```

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `VUS` | `20` | k6 virtual users (concurrent load) |
| `DURATION` | `60s` | k6 run duration — e.g. `30s`, `2m`, `5m` |
| `APP_CPUS` | `2` | CPU limit applied to each app container |
| `APP_MEMORY` | `512m` | Memory limit applied to each app container |
| `PGHOST` | `localhost` | Postgres host reachable from the **host** machine |
| `PGPORT` | `5434` | Postgres port on the host (the compose stack binds `5434`) |
| `PGUSER` | `myuser` | Postgres user |
| `PGPASSWORD` | `mypassword` | Postgres password |
| `LARAVEL_APP_KEY` | — | **Required** for Laravel; generate once (see below) |
| `RAILS_SECRET_KEY_BASE` | — | Required for Rails optimized variant |
| `PHOENIX_SECRET_KEY_BASE` | — | Optional for Phoenix |
| `NO_BUILD` | `0` | Set to `1` to skip `docker build` and use existing images |
| `KEEP_RESULTS` | `0` | Set to `1` to preserve the results directory even on failure |
| `READINESS_TIMEOUT` | `90` | Seconds to wait for a container to become ready (JVM stacks override this to `120` in `stacks.json`) |

---

## Per-Stack Notes

### Laravel (PHP) — `LARAVEL_APP_KEY` required

Laravel refuses to start without `APP_KEY`. Generate it once from the Laravel project:

```bash
cd /path/to/speed-test/php/laravel
php artisan key:generate --show
# Prints: base64:abc123...
export LARAVEL_APP_KEY="base64:abc123..."
```

Then run:

```bash
LARAVEL_APP_KEY="base64:abc123..." VUS=20 DURATION=60s ./run.sh laravel
```

### Rails (Ruby) — development mode

The generated `docker-entrypoint.sh` calls `bin/rails db:migrate`, which fails because the app does not use ActiveRecord. `stacks.json` bypasses this via `entrypoint_override`/`cmd_override`. The app runs in `RAILS_ENV=development` to avoid an ActionCable eager-load crash in the generated production config.

If you run the optimized variant you may also need `RAILS_SECRET_KEY_BASE`:

```bash
export RAILS_SECRET_KEY_BASE=$(ruby -rsecurerandom -e 'puts SecureRandom.hex(64)')
```

### Python (FastAPI) — Bearer token

FastAPI's generated OAuth2 stub requires `Authorization: Bearer <token>`. `stacks.json` sets `"auth_header": "Bearer benchmark-token"`, which is automatically passed to k6. No action needed.

### Node.js (Express) — api_key header

`express-openapi-validator` enforces the `api_key` security scheme. k6 always sends `"api_key": "benchmark"` in every request. No action needed.

### JVM stacks (Spring Boot, Helidon, Quarkus, Ktor, Phoenix) — slow builds

Maven/Gradle download dependencies on the first `docker build`. Expect 2–5 minutes per build on a cold cache. These stacks have `"readiness_timeout": 120` in `stacks.json`. If a stack still times out, raise the limit:

```bash
READINESS_TIMEOUT=180 ./run.sh springboot optimized
```

---

## Output Files

Each run creates a directory at `results/<stack>-<variant>-<timestamp>/`:

| File | Contents |
|---|---|
| `run-meta.json` | Stack id, variant, VUs, duration, CPU/memory limits, timestamp |
| `build.log` | Full `docker build` output |
| `docker-stats.csv` | 1-second-interval CPU %, RAM (MiB), net I/O, block I/O for the app container and the Postgres container, for the full duration of the k6 run |
| `pg-before.json` | Snapshot of `pg_stat_database`, `pg_stat_io`, and top-50 `pg_stat_statements` rows (by total exec time) taken **before** k6 starts |
| `pg-after.json` | Same snapshot taken **after** k6 finishes |
| `pg-delta.json` | Field-by-field difference between before and after |
| `k6-summary.json` | k6 `--summary-export` output: RPS, latency (avg/p50/p90/p95/p99/max), error rate, request counts, per-check results |
| `k6.log` | Full k6 stdout/stderr |
| `container.log` | App container stdout/stderr (`docker logs`) |

`results/` is gitignored.

---

## Generating a Comparison Report

```bash
python3 report.py
```

Reads every `results/*/run-meta.json` directory and writes:

- `results/comparison.csv` — machine-readable, one row per run
- `results/comparison.md` — Markdown table with: stack, variant, RPS, avg/p95/p99 latency (ms), error rate (%), CPU avg/peak (%), RAM avg/peak (MiB), PG transactions, PG blocks read/hit

Optional arguments:

```bash
python3 report.py --results-dir /path/to/results --output-csv out.csv --output-md out.md
```

---

## Visual Results Viewer

A React + Vite SPA lets you explore individual runs and compare up to 4 stacks side-by-side with charts.

```bash
cd viewer
npm install
npm run dev
# Open http://localhost:5173
```

The data generator (`scripts/build-data.mjs`) runs automatically before `dev` and `build`. After adding new benchmark runs, restart `npm run dev` (or run `node scripts/build-data.mjs` manually) to refresh the data.

### Pages

**Single Run** (`/`) — pick a stack and run; shows:
- Stat cards: RPS, avg/p95/p99 latency, error rate, total requests, CPU peak, RAM peak
- Latency breakdown bar chart (avg, p50, p90, p95, p99, max)
- CPU and RAM time-series over the run
- Per-endpoint k6 check pass/fail counts
- PostgreSQL counters delta

**Compare** (`/compare`) — select up to 4 stack/run combinations; shows:
- RPS, error rate, latency percentiles, CPU, RAM, and Postgres counters as grouped bar charts
- CPU and RAM time-series as line overlays

See `viewer/README.md` for full details.

---

## Tips for Reproducible Results

- **Warm the Docker image cache** — run each stack twice with `NO_BUILD=1` on the second run; the first run pays build costs that inflate wall-clock time.
- **Use the same limits** — always set `APP_CPUS` and `APP_MEMORY` explicitly when comparing stacks, otherwise Docker inherits the host's full resources.
- **Use `DURATION=120s` or longer** — JVM stacks need 20–30 s to JIT-compile hot paths; a 60 s run still sees warm-up overhead in the metrics.
- **Run stacks sequentially, not in parallel** — `run.sh` runs one container at a time. The Postgres container is shared; concurrent app containers fight for the same DB.
- **Check `k6.log` for threshold violations** — a non-zero error rate is usually a concurrency race on `MAX(id)+1` ID generation, not a harness bug. See the Known Quirks section in `AGENTS.md`.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `Container did not become ready` | JVM stack slow build or port conflict | Check `container.log`; raise `READINESS_TIMEOUT` |
| `Postgres container not running` | Compose stack not started | `cd database && docker compose up -d` |
| `Docker network not found` | Compose stack not started | Same as above — the network is created by `docker compose up` |
| `100% http_req_failed` for Rust | Pre-existing DB connectivity issue in the naive Rust server | Known; Rust build succeeds, runtime has a DB init bug |
| k6 reports high error rate (10–25%) | Delete concurrency race — multiple VUs delete the same row | Expected; see AGENTS.md Known Quirks |
| Laravel container exits immediately | `APP_KEY` not set | Export `LARAVEL_APP_KEY` (see Per-Stack Notes above) |
| JVM build fails in Docker | `org.gradle.java.home` pointing to host path | Already fixed in repo; verify `java/quarkus/gradle.properties` doesn't contain it |

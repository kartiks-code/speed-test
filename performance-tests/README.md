# Petstore Performance Benchmark Harness

A language-agnostic Docker harness that benchmarks all 12 Petstore server implementations (Go, Java Spring Boot, Java Helidon, Java Quarkus, Node.js, Python, Rust, C# ASP.NET Core, PHP Laravel, Ruby Rails, Kotlin Ktor, Elixir Phoenix) against the same PostgreSQL instance, collecting RPS, latency percentiles, CPU, RAM, and Postgres statistics.

Each stack ships a `Dockerfile` (**naive** — stock dependencies, no tuning) and a `Dockerfile.optimized` (**optimized** — multi-stage build, slim base image, JVM flags, connection pool tuning, etc.). Both variants are benchmarked so you can see the raw performance floor and what careful optimization buys.

Three components work together:

| Component | Path | Role |
|---|---|---|
| Harness | `run.sh`, `stacks.json`, `k6/`, `sampler.py`, `pg-stats.sh` | Build, run, load-test, collect metrics |
| Viewer | `viewer/` | React SPA to browse and compare results |
| Control server | `server/` | Local API + queue for triggering runs from the viewer UI |

---

## Stacks

| Stack ID | Language | Framework |
|---|---|---|
| `go` | Go | Gin |
| `springboot` | Java 25 | Spring Boot 3.5 |
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
| Node.js 18+ *(optional)* | [nodejs.org](https://nodejs.org) | Viewer SPA and control server (not needed for CLI-only `./run.sh`) |

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
| `K6_SCRIPT_NAME` | `crud.js` | k6 script filename inside `k6/` (use `crud-mix.js` for weighted operation mix) |
| `MIX_CREATE` | `25` | Relative weight for Create (only when `K6_SCRIPT_NAME=crud-mix.js`) |
| `MIX_READ` | `25` | Relative weight for Read |
| `MIX_UPDATE` | `25` | Relative weight for Update |
| `MIX_DELETE` | `25` | Relative weight for Delete |
| `DOCKERFILE_OVERRIDE` | — | Dockerfile path relative to `build_context`; bypasses naive/optimized mapping; variant label used as-is in results dir |
| `SUITE_NAME` | — | Optional label grouping this run with others (stored in `run-meta.json`; searchable in Compare and Manage Runs) |
| `CONTROL_PORT` | `5179` | Control server listen port (see Control Server below) |

---

## k6 Load Scripts

Two scripts live in `k6/`:

| Script | Used by | Behavior |
|---|---|---|
| `crud.js` | `./run.sh` (default) | Fixed full CRUD cycle on every iteration — original benchmark script |
| `crud-mix.js` | Control server / viewer UI | Weighted random operation per iteration; seeds a shared pet pool in `setup()` |

Run the mix script from the CLI:

```bash
K6_SCRIPT_NAME=crud-mix.js MIX_CREATE=10 MIX_READ=50 MIX_UPDATE=25 MIX_DELETE=15 \
  VUS=20 DURATION=60s ./run.sh go naive
```

Weights are relative integers (not percentages); they are normalised at startup. The control server always uses `crud-mix.js` with mix weights from the Run Tests form.

---

## Benchmark Suites

A **suite** is an optional label shared by multiple runs so you can compare or manage them as a batch. Suite names are stored in each run's `run-meta.json` as `"suite"`.

**From the CLI** — tag individual runs:

```bash
SUITE_NAME=jvm-comparison-june VUS=20 DURATION=60s ./run.sh springboot,quarkus optimized
```

**From the viewer** — use **Run Tests** (`/run`) to queue a cartesian product of selected stacks × variants under one suite name (e.g. 3 stacks × 2 variants = 6 queued jobs). Jobs still execute sequentially; the suite name is forwarded as `SUITE_NAME` to each `run.sh` invocation.

**After the fact** — **Manage Runs** (`/manage`) can assign a suite label to existing runs, or **Compare** (`/compare`) can load all runs in a suite by name.

---

## Host Port Assignments

Each stack publishes a unique host port so containers can run side-by-side for debugging. Internally every app still listens on **8080**; `run.sh` still runs one benchmark at a time and tears down before the next.

| Stack | Naive | Optimized |
|---|---|---|
| go | 8081 | 8082 |
| springboot | 8083 | 8084 |
| helidon | 8085 | 8086 |
| quarkus | 8087 | 8088 |
| nodejs | 8089 | 8090 |
| python | 8091 | 8092 |
| rust | 8093 | 8094 |
| csharp | 8095 | 8096 |
| laravel | 8097 | 8098 |
| rails | 8099 | 8100 |
| ktor | 8101 | 8102 |
| phoenix | 8103 | 8104 |

Port pairs are defined in `stacks.json` (`host_port` / `host_port_optimized`). New stacks start at 8105.

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

### Rails (Ruby) — per-variant environment

The generated `docker-entrypoint.sh` calls `bin/rails db:migrate`, which fails because the app does not use ActiveRecord. `stacks.json` bypasses this via `entrypoint_override`/`cmd_override`. The naive variant runs in `RAILS_ENV=development`; the optimized variant runs in `RAILS_ENV=production` with eager loading (set via the `env_optimized` field in `stacks.json`, which is merged over `env` for optimized runs only).

The optimized variant requires `RAILS_SECRET_KEY_BASE`:

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
| `run-meta.json` | Stack id, label, variant, `dockerfile`, `db_name`, `host_port`, VUs, duration, CPU/memory limits, timestamp, `k6_script`, `mix` weights, optional `suite`, and `timing` (`startup_seconds`, k6/sampler window timestamps) |
| `build.log` | Full `docker build` output |
| `docker-stats.csv` | 1-second-interval CPU %, RAM (MiB), net I/O, block I/O for the app container and the Postgres container while k6 runs (after server readiness; `crud-mix` pet seeding happens before the sampler starts) |
| `pg-before.json` | Snapshot of `pg_stat_database`, `pg_stat_io`, and top-50 `pg_stat_statements` rows (by total exec time) taken **after** the server is ready and **before** k6 starts |
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

A React + Vite SPA for exploring results, comparing runs, and queuing new benchmarks from the browser.

### View-only

```bash
cd viewer
npm install
npm run dev
# Open http://localhost:5173
```

### With run triggering (Run Tests + Queue pages)

```bash
# Option A — two terminals
cd server && npm install && npm start          # http://127.0.0.1:5179
cd viewer && npm run dev                         # http://localhost:5173

# Option B — one command from viewer/
npm run dev:all
```

Vite proxies `/api` to the control server on `127.0.0.1:5179`.

The data generator (`scripts/build-data.mjs`) runs automatically before `dev` and `build`. It scans `results/` and emits `viewer/public/data/index.json` plus per-run JSON (including `k6_script`, `mix`, and `suite` from `run-meta.json`). After CLI benchmark runs, restart `npm run dev` or run `node scripts/build-data.mjs`. Runs triggered via the control server regenerate data automatically and push a `data_updated` SSE event so open pages reload without a manual refresh.

### Pages

**Single Run** (`/`) — pick a stack and run; shows:
- Stat cards: RPS, avg/p95/p99 latency, error rate, total requests, CPU peak, RAM peak
- Meta badges: stack, variant, VUs, duration, CPUs, memory, timestamp — plus **Script** and **Mix** when the run used `crud-mix.js`
- Latency breakdown bar chart (avg, p50, p90, p95, p99, max)
- Resource summary table (CPU avg/peak, RAM avg/peak, net, block I/O)
- CPU and RAM time-series over the run
- Per-endpoint k6 check pass/fail counts
- PostgreSQL counters delta

**Compare** (`/compare`) — pick up to 6 stack/run combinations manually (stack → duration → variant → timestamp), or search for a **suite** to load all runs from a benchmark batch. Shows RPS, error rate, latency percentiles, CPU, RAM, and Postgres counters as grouped bar charts, plus CPU/RAM time-series overlays. When a suite has more than 6 stack×variant combos, results paginate (6 per page). Use **Change groups** to assign combos to pages; layout persists in browser `localStorage`.

**Manage Runs** (`/manage`) — filter runs by suite, stack, duration, variant, VUs, and time range (24h / 7d / 30d presets or custom UTC dates). Multi-select rows to **Name as suite**, **Delete selected**, or open a suite in Compare. The **Suites** panel can dissolve a suite (remove labels, keep runs) or delete all runs in a suite. Browse/filter works view-only; mutations require the control server.

**Run Tests** (`/run`) — requires control server; form to queue a **benchmark suite**: suite name, multi-select stacks and variants (naive/optimized/custom Dockerfiles discovered live), duration, VUs, and CRUD-mix sliders. Schedules `stacks × variants` jobs sequentially via `crud-mix.js`. Redirects to Queue on submit.

**Queue** (`/queue`) — requires control server; live job list with status badges (Pending / Running / Done / Failed / Canceled), suite name per job, streaming `run.sh` logs via SSE, cancel for pending jobs, and **View results →** deep-link on completion.

See `viewer/README.md` for full details.

---

## Control Server

A local Express server (`server/`) that powers **Run Tests**, **Queue**, and **Manage Runs** mutations. Jobs run strictly one at a time; stdout/stderr stream to the browser via SSE.

```bash
cd server
npm install
npm start
# Listens on http://127.0.0.1:5179 (override with CONTROL_PORT)
```

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/stacks` | All stacks with discovered Dockerfile variants |
| `GET` | `/api/queue` | Current job queue |
| `POST` | `/api/queue` | Enqueue one run or a named suite (stacks × variants batch) |
| `DELETE` | `/api/queue/:id` | Cancel a pending job |
| `POST` | `/api/runs/assign-suite` | Assign a suite label to existing runs |
| `DELETE` | `/api/runs` | Delete completed run directories |
| `DELETE` | `/api/suites/:name` | Dissolve suite labels or delete all runs in a suite (`?action=delete-runs`) |
| `GET` | `/api/events` | SSE stream (`queue_update`, `log`, `data_updated`) |

See `server/README.md` for request bodies and event types.

Secret env vars (`LARAVEL_APP_KEY`, `RAILS_SECRET_KEY_BASE`, etc.) must be set in the shell that starts the control server — they are forwarded to spawned `run.sh` processes.

---

## Tips for Reproducible Results

- **Warm the Docker image cache** — run each stack twice with `NO_BUILD=1` on the second run; the first run pays build costs that inflate wall-clock time.
- **Use the same limits** — always set `APP_CPUS` and `APP_MEMORY` explicitly when comparing stacks, otherwise Docker inherits the host's full resources.
- **Use `DURATION=120s` or longer** — JVM stacks need 20–30 s to JIT-compile hot paths; a 60 s run still sees warm-up overhead in the metrics.
- **Run stacks sequentially via `run.sh`** — the orchestrator runs one container at a time and tears down before the next. Host ports allow manual side-by-side containers for debugging, but concurrent benchmarks against the shared Postgres will skew results.
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

# Performance Tests — Agent Guide

Docker-based benchmark harness for the Petstore multi-language speed test. Builds each stack as a Docker image, starts it against the shared Postgres container, runs a k6 CRUD load test, and collects RPS, latency, CPU, RAM, and Postgres statistics. All 12 stacks are supported, each in a `naive` variant (stock Dockerfile) and an `optimized` variant (`Dockerfile.optimized`).

See `README.md` for operator-facing usage (prerequisites, quick start, environment variables, host ports, k6 scripts). See `viewer/README.md` for the React SPA; see `server/README.md` for the control-server API.

## File Roles

| File | Role |
|---|---|
| `run.sh` | Orchestrator: build → start container → readiness poll → pg snapshot → sampler → k6 → teardown → pg delta → run-meta.json |
| `stacks.json` | Per-stack config: build context, Dockerfile paths, env vars, `base_path`, `readiness_path`, `readiness_timeout`, optional `entrypoint_override` and `cmd_override` |
| `sampler.py` | Polls Docker Engine API via Unix socket every 1 s → writes `docker-stats.csv` for the app container + Postgres container |
| `pg-stats.sh` | Snapshots and diffs `pg_stat_database`, `pg_stat_io`, `pg_stat_statements`; also resets stats counters and enables the extension |
| `k6/crud.js` | Fixed full CRUD cycle script (original); unchanged |
| `k6/crud-mix.js` | Weighted operation mix script; driven by `MIX_CREATE/READ/UPDATE/DELETE` env vars; seeds a pet pool in `setup()` then picks one operation per VU iteration by normalized weight |
| `report.py` | Reads all `results/*/` directories → writes `results/comparison.csv` and `results/comparison.md` |
| `results/` | One directory per run, named `<stack>-<variant>-<timestamp>`; gitignored |
| `server/` | Local Node control server (Express); exposes queue, run-management, and SSE APIs; spawns `run.sh` |
| `server/queue.mjs` | In-memory FIFO queue; `enqueue` / `enqueueBatch`; spawns `run.sh` with `K6_SCRIPT_NAME=crud-mix.js` and `SUITE_NAME` |
| `server/runs.mjs` | Suite assignment, run deletion, suite dissolve/delete (mutates `run-meta.json` and result dirs on disk) |
| `server/dataRefresh.mjs` | Runs `build-data.mjs` and broadcasts `data_updated` SSE after queue or run-management changes |
| `viewer/` | React + Vite SPA for browsing results and triggering new runs; run `npm install && npm run dev` |
| `viewer/scripts/build-data.mjs` | Node generator: scans `results/`, emits `viewer/public/data/index.json` + per-run JSON; runs automatically via `predev`/`prebuild` |

## How `run.sh` Works

1. Parse stack id and variant (`naive` / `optimized`) from CLI arguments.
2. Read `stacks.json` via `jq` to get `build_context`, `db_name`, `base_path`, `readiness_path`, `readiness_timeout`, and any overrides.
3. Determine the Dockerfile: use `DOCKERFILE_OVERRIDE` env if set; otherwise map `optimized`→`Dockerfile.optimized`, anything else→`Dockerfile` and normalise variant to `naive`.
4. `docker build` the stack's Dockerfile (skipped if `NO_BUILD=1`); stream output to `build.log`.
5. Write a temp env-file from `stacks.json`'s `env` block (skip `PLACEHOLDER_*` values); inject secrets from caller's environment (`LARAVEL_APP_KEY`, `RAILS_SECRET_KEY_BASE`, `PHOENIX_SECRET_KEY_BASE`).
6. `docker run -d` the image on the `database_default` network with CPU/memory limits and the env-file; apply `entrypoint_override` / `cmd_override` if present in `stacks.json`.
7. Poll the readiness URL (`http://<container>:8080<readiness_path>`) via `curlimages/curl` on the same network until HTTP 200 or timeout.
8. Reset Postgres stats and take `pg-before.json` snapshot via `pg-stats.sh`.
9. Start `sampler.py` in the background.
10. Run k6 (`grafana/k6:latest`) on the same network, mounting `k6/${K6_SCRIPT_NAME}` and the results directory; pass `BASE_URL`, `BASE_PATH`, `VUS`, `DURATION`, `AUTH_HEADER` (if set), and `MIX_CREATE/READ/UPDATE/DELETE`. Exit code is non-fatal (`|| true`).
11. Kill `sampler.py`.
12. Take `pg-after.json` snapshot and compute `pg-delta.json`.
13. Save `container.log` from `docker logs`.
14. `docker rm -f` the app container.
15. Write `run-meta.json` with stack id, label, variant, `dockerfile`, `db_name`, `host_port`, limits, timestamp, `k6_script`, `mix` object, and optional `suite` (from `SUITE_NAME` env).

### k6 script selection

| Invocation | Default script | Notes |
|---|---|---|
| `./run.sh` (CLI) | `crud.js` | Set `K6_SCRIPT_NAME=crud-mix.js` and `MIX_*` weights to use the weighted mix |
| Control server (`server/queue.mjs`) | `crud-mix.js` | Mix weights come from the POST `/api/queue` body or Run Tests form defaults (25/25/25/25) |

`crud.js` runs a fixed full CRUD cycle every iteration. `crud-mix.js` seeds pets in `setup()`, then picks one weighted operation per iteration.

### Additional environment variables

| Variable | Default | Description |
|---|---|---|
| `K6_SCRIPT_NAME` | `crud.js` | k6 script filename inside `k6/` |
| `MIX_CREATE` | `25` | Relative weight for Create operations |
| `MIX_READ` | `25` | Relative weight for Read operations |
| `MIX_UPDATE` | `25` | Relative weight for Update operations |
| `MIX_DELETE` | `25` | Relative weight for Delete operations |
| `DOCKERFILE_OVERRIDE` | _(empty)_ | Explicit Dockerfile path relative to `build_context`; variant arg used as-is |
| `SUITE_NAME` | _(empty)_ | Optional label grouping this run with others in the same benchmark suite |

## Benchmark Suites

Suite names tie multiple runs together for Compare and Manage Runs. They are written to `run-meta.json` as `"suite"` (null when unset).

| Source | How |
|---|---|
| CLI | `SUITE_NAME=my-suite ./run.sh go,springboot naive` |
| Control server single job | Optional `suiteName` in `POST /api/queue` body |
| Control server batch | `POST /api/queue` with `suiteName`, `stackIds[]`, `variants[]` — enqueues cartesian product via `enqueueBatch()` |
| Retroactive | `POST /api/runs/assign-suite` or Manage Runs UI |

The Run Tests page (`viewer/src/routes/RunTests.jsx`) always uses batch enqueue. Manage Runs and Compare read `suite` from generated viewer data (`build-data.mjs` passes it through from `run-meta.json`).

## `stacks.json` Schema

Each entry is an object with these fields:

| Field | Required | Description |
|---|---|---|
| `id` | yes | Stack identifier used in CLI args and directory names |
| `label` | yes | Human-readable name for reports |
| `build_context` | yes | Path to the project directory **relative to repo root** (e.g. `"go/go-gin-server"`) |
| `dockerfile` | yes | Naive Dockerfile name (almost always `"Dockerfile"`) |
| `dockerfile_optimized` | yes | Optimized Dockerfile name (almost always `"Dockerfile.optimized"`) |
| `db_name` | yes | Postgres database name to truncate and snapshot |
| `host_port` | yes | Host port published for the naive container (`-p host_port:8080`); allows all stacks to run simultaneously |
| `host_port_optimized` | yes | Host port published for the optimized container; must differ from `host_port` |
| `base_path` | yes | URL prefix for all API routes (e.g. `"/api/v3"` or `""`) |
| `readiness_path` | yes | Path appended to `http://<container>:8080` for the readiness probe |
| `readiness_timeout` | no | Seconds to wait for readiness (default: `READINESS_TIMEOUT` env var, default 90) |
| `env` | yes | Object of env vars injected into the container; use `PLACEHOLDER_*` values to omit a key at runtime |
| `auth_header` | no | Value for the `AUTH_HEADER` env var passed to k6 (e.g. `"Bearer benchmark-token"`) |
| `entrypoint_override` | no | Replaces the container `ENTRYPOINT` |
| `cmd_override` | no | JSON array replacing the container `CMD` |
| `notes` | no | Free-text notes for humans |

## Host Port Assignments

Each stack gets two unique host ports (naive / optimized) so all 24 containers can run simultaneously. All containers still bind internally on **8080**; only the host-side mapping differs.

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

When adding a new stack, pick the next two available ports starting from 8105 and set `host_port` / `host_port_optimized` in `stacks.json`.

## Adding a New Stack

1. Add a `Dockerfile` (and optionally `Dockerfile.optimized`) to the project directory.
2. Add an entry to `stacks.json` with all required fields. Set `POSTGRES_HOST` to `"speed-test-postgres"` and `POSTGRES_PORT` to `"5432"` (the in-network address of the Postgres container). Choose the next available `host_port` / `host_port_optimized` pair from the table above (start at 8105).
3. Ensure the project's database exists: `cd database && ./create-databases.sh && ./apply-schemas.sh`.
4. Smoke-test:
   ```bash
   VUS=3 DURATION=15s ./run.sh <stack_id> naive
   ```
5. Check `results/<stack_id>-naive-<ts>/k6-summary.json` for errors. Check `container.log` if the container failed to start.

## Viewer (`viewer/`)

A React + Vite SPA for exploring results visually and triggering new runs.

**Start view-only:** `cd viewer && npm install && npm run dev`

**Start with run triggering:** `npm run dev:all` from `viewer/` (starts control server + Vite), or run `npm run server` / `cd server && npm start` separately.

The `predev`/`prebuild` scripts call `scripts/build-data.mjs`, which scans `../results/` and emits `viewer/public/data/index.json` plus per-run JSON files (including `k6_script`, `mix`, and `suite` from `run-meta.json`). These generated files are gitignored.

Pages: **Single Run** (`/`), **Compare** (`/compare`), **Manage Runs** (`/manage`), **Run Tests** (`/run`), **Queue** (`/queue`).

On **Compare**, suites with more than 6 stack×variant combos are paginated (6 charts per page). Custom page assignments persist in browser `localStorage` via **Change groups**.

**Manage Runs** filters by suite/stack/duration/variant/VUs/time range, supports multi-select, retroactive suite naming, run deletion, and suite dissolve/delete (requires control server for mutations).

The **Run Tests** and **Queue** pages require the control server. Vite proxies all `/api` requests to `127.0.0.1:5179`.

When editing the viewer:
- All result data is read from `public/data/` at runtime — never from the filesystem directly.
- After CLI benchmark runs, regenerate data: `node scripts/build-data.mjs` or restart `npm run dev`. Control-server jobs and run-management API calls trigger `build-data.mjs` automatically.
- Compare suite pagination state lives in browser `localStorage` (`viewer/src/lib/suiteGroups.js`).
- `viewer/node_modules/`, `viewer/dist/`, and `viewer/public/data/` are all gitignored.
- `results/` is gitignored at the harness root (`performance-tests/.gitignore`).

## Control Server (`server/`)

A local Express server that powers the run-triggering UI and run-management mutations. See `server/README.md` for the full API reference.

**Start:** `cd server && npm install && npm start`

Key behaviours:
- Jobs are processed strictly one at a time (sequential queue).
- `POST /api/queue` accepts either a single `{ stackId, variant, ... }` or a batch `{ suiteName, stackIds, variants, ... }`.
- Each job spawns `run.sh <stackId> <variant>` with `K6_SCRIPT_NAME=crud-mix.js`, mix weights, optional `DOCKERFILE_OVERRIDE`, and optional `SUITE_NAME`.
- stdout/stderr are broadcast as SSE `log` events; queue state changes emit `queue_update`.
- Run management (`runs.mjs`): assign-suite, delete runs, dissolve/delete suites — each calls `refreshViewerAndBroadcast()`.
- On job completion or run mutation, `build-data.mjs` runs automatically, then a `data_updated` SSE event is broadcast so Single Run and Compare pages reload without a manual refresh.
- The server is local-only (`127.0.0.1`), no authentication.
- `CONTROL_PORT` env var overrides the default port `5179`.
- Secret env vars must be present in the server's environment (forwarded to child `run.sh` processes).

## Known Quirks (Already Fixed in Code)

These are in place; do not revert them.

**Python — OAuth2 Bearer header.** FastAPI's generated OAuth2 stub requires `Authorization: Bearer <token>`. `stacks.json` sets `"auth_header": "Bearer benchmark-token"`; `run.sh` passes it to k6 as `AUTH_HEADER`; `crud.js` merges it into all request headers.

**Node.js — api_key header.** `express-openapi-validator` enforces the `api_key` security scheme. `crud.js` always includes `"api_key": "benchmark"` in headers for all requests.

**Rails — development mode + entrypoint override.** The generated `docker-entrypoint.sh` calls `db:migrate` (fails without ActiveRecord) and `RAILS_ENV=production` triggers an ActionCable eager-load error. `stacks.json` sets `entrypoint_override: "bin/rails"` and `cmd_override: ["server", "-b", "0.0.0.0", "-p", "8080"]` to bypass both. `RAILS_ENV=development` avoids the ActionCable issue.

**JVM stacks — readiness timeout.** Spring Boot, Helidon, Quarkus, Ktor, and Phoenix take 2–3 minutes to build and start. All five have `"readiness_timeout": 120` in `stacks.json`.

**k6 — uploadImage path.** The spec operation is `uploadFile` but the route is `/pet/{petId}/uploadImage`. `crud.js` uses `/uploadImage`.

**k6 — threshold exit code.** k6 exits non-zero when thresholds are violated. `run.sh` appends `|| true` so a single stack's threshold failure does not abort the suite.

**k6 — summary file permissions.** k6's Docker image runs as non-root and cannot create files in mounted host volumes. `run.sh` does `touch + chmod 666` on `k6-summary.json` before starting k6.

**Delete concurrency 404s.** Multiple VUs race on `MAX(id)+1` ID assignment and may attempt to delete the same row. A small percentage of delete calls returning 404 is expected at the default VU count; it is benchmark data, not a harness bug. Stacks that do a pre-existence check before deleting are more susceptible; idempotent deletes (delete directly and ignore rows-affected) eliminate the race.

**stacks.json `build_context` paths.** Paths are relative to the repo root, not to `performance-tests/`. Use `"go/go-gin-server"`, not `"../go/go-gin-server"`.

**Host ports vs sequential runs.** `stacks.json` assigns unique `host_port` / `host_port_optimized` per stack so all 24 variants can bind simultaneously for manual debugging. `run.sh` still runs one benchmark at a time and removes the container before the next — do not run concurrent `run.sh` processes against the shared Postgres for comparable results.

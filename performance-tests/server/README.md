# Perf-Test Control Server

A small local Node.js server that powers the **Run Tests** and **Queue** pages in the viewer SPA. It exposes a JSON/SSE API that lets the browser trigger benchmark runs, queue them for sequential execution, and stream live `run.sh` output back to the browser.

## Prerequisites

- Node.js 18+
- All `performance-tests/` prerequisites met (Docker, jq, psql, Python 3)
- The shared Postgres container running (`cd database && docker compose up -d`)

## Quick start

```bash
cd performance-tests/server
npm install
npm start
# Server listens on http://127.0.0.1:5179
```

Or from the `viewer/` directory (both server + Vite in one command):

```bash
cd performance-tests/viewer
npm run dev:all
```

## API reference

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/stacks` | All stacks with discovered Dockerfile variants |
| `GET` | `/api/queue` | Current job queue (all statuses) |
| `POST` | `/api/queue` | Enqueue a new run |
| `DELETE` | `/api/queue/completed` | Remove finished jobs from the queue (done, failed, canceled); does not delete result dirs |
| `DELETE` | `/api/queue/:id` | Cancel a pending job |
| `POST` | `/api/runs/assign-suite` | Assign a suite name to existing runs |
| `DELETE` | `/api/runs` | Delete completed run result directories |
| `DELETE` | `/api/suites/:name` | Dissolve suite (remove labels) or delete all runs (`?action=delete-runs`) |
| `GET` | `/api/events` | SSE stream of `queue_update` and `log` events |

### POST /api/queue body

**Single run** (legacy):

```json
{
  "stackId": "go",
  "variant": "naive",
  "suiteName": "optional-suite-label",
  "durationSec": 60,
  "vus": 20,
  "mix": { "create": 25, "read": 50, "update": 15, "delete": 10 },
  "dockerfileOverride": ""
}
```

**Named suite** (cartesian product of stacks × variants):

```json
{
  "suiteName": "jvm-comparison-june",
  "stackIds": ["go", "springboot", "quarkus"],
  "variants": ["naive", "optimized"],
  "durationSec": 60,
  "vus": 20,
  "mix": { "create": 25, "read": 50, "update": 15, "delete": 10 }
}
```

Returns `{ suiteName, count, jobs }` with `count = stackIds.length × variants.length`.

All fields except `stackId`/`variant` (single) or `suiteName`/`stackIds`/`variants` (batch) are optional (defaults: 60s, 20 VUs, equal 25/25/25/25 mix). Suite name is stored in each run's `run-meta.json` and searchable in the viewer.

### POST /api/runs/assign-suite body

Assign a suite label to existing runs (retroactive grouping):

```json
{
  "runIds": ["go-naive-20260612T010735Z", "springboot-naive-20260612T011219Z"],
  "suiteName": "my-retro-suite"
}
```

### DELETE /api/runs body

Permanently delete one or more completed runs:

```json
{ "runIds": ["go-naive-20260612T010735Z"] }
```

### DELETE /api/suites/:name

- Default (`action` omitted): **dissolve** — remove the suite label from all matching runs; run data remains.
- `?action=delete-runs`: delete all result directories belonging to the suite.

Returns `{ action, suiteName, count|deleted, runIds }`. Regenerates viewer data and emits `data_updated` on SSE.

### SSE event types

- `queue_update` — queue state array (job metadata only; logs omitted); emitted on any status change
- `log` — `{ jobId, line }` — one stdout/stderr line from `run.sh`
- `data_updated` — `{ runId }` — emitted after `build-data.mjs` finishes; the viewer reloads `public/data/` automatically

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CONTROL_PORT` | `5179` | Port the server binds to |

All Postgres and secret env vars supported by `run.sh` (e.g. `LARAVEL_APP_KEY`, `RAILS_SECRET_KEY_BASE`) must be set in the environment that starts this server — they are forwarded to the spawned `run.sh` process.

## How it works

1. A job is added to an in-memory FIFO queue.
2. The queue manager picks the next `pending` job and spawns `run.sh <stackId> <variant>` with `K6_SCRIPT_NAME=crud-mix.js` and the mix weights as env vars.
3. stdout/stderr are broadcast as SSE `log` events in real time.
4. On completion, `build-data.mjs` is run to regenerate `viewer/public/data/` so the result appears immediately in the Single Run and Compare pages.
5. The job's `runId` is set to the newly created results directory name for deep-linking.

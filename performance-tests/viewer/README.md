# Petstore Benchmark Viewer

A React + Vite single-page app for exploring and comparing benchmark run results, and for triggering new runs through the browser UI.

## Prerequisites

- Node.js 18+
- For viewing existing results: at least one completed run in `performance-tests/results/`
- For running new tests: all `performance-tests/` prerequisites (Docker, jq, psql, Python 3, shared Postgres container running)

## Quick start

### View-only (no control server needed)

```bash
cd performance-tests/viewer
npm install
npm run dev
```

Open the URL printed by Vite (typically `http://localhost:5173`).

### With the control server (Run Tests + Queue pages)

```bash
# Terminal 1 — control server
cd performance-tests/server
npm install
npm start

# Terminal 2 — viewer
cd performance-tests/viewer
npm run dev
```

Or use the combined shortcut:

```bash
cd performance-tests/viewer
npm run dev:all
```

The control server listens on `http://127.0.0.1:5179`. Vite proxies all `/api` requests to it automatically.

## Pages

### Single Run (`/`)

Pick a stack, duration, and specific run. Displays:

- Stat cards: RPS, avg/p95/p99 latency, error rate, total requests, CPU peak, RAM peak
- Meta badges: Stack, Variant, VUs, Duration, CPUs, Memory, Timestamp — plus **Script** and **Mix** badges when the run used `crud-mix.js`
- Latency breakdown bar chart (avg, p50, p90, p95, p99, max)
- Resource summary table (CPU avg/peak, RAM avg/peak, net, block I/O)
- CPU and RAM time-series charts
- Per-endpoint checks (passes vs fails per k6 check)
- PostgreSQL counters delta

### Compare (`/compare`)

Select up to 6 stack/run combinations manually (stack → duration → **variant** → timestamp), or search for a **suite** to compare all runs from a benchmark batch.

When a suite has more than 6 stack×variant combos, results are split into pages of 6. Use **Prev/Next** or numbered page tabs to navigate. **Change groups** opens a modal where you assign each combo to a page; the layout is saved in browser localStorage and reused whenever you load that suite again.

Each row shows a brief summary (VUs, seconds, CRUD mix) for the selected run. Each series gets a distinct color. Grouped bar and line-overlay charts for RPS, error rate, latency percentiles, CPU, RAM, and Postgres counters.

### Manage Runs (`/manage`) — mutations require control server

Filter and bulk-manage completed benchmark results:

- **Filters** — same dimensions as Compare (suite search, stack, duration, variant) plus VU count, time presets (24h / 7d / 30d), and custom UTC date range
- **Multi-select** — select all filtered, select filtered only, or pick individual rows
- **Name as suite** — assign a suite label to selected runs after the fact
- **Delete selected** — permanently remove chosen run directories
- **Suites panel** — dissolve a suite (remove labels, keep runs) or delete all runs in a suite; quick link to Compare

Browse and filter work view-only; assign/delete/dissolve require the control server (`npm run server` or `npm run dev:all`).

### Run Tests (`/run`) — requires control server

Form to configure and queue a benchmark run:

- **Stack** — pick from all 12 stacks (loaded live from the control server)
- **Variant** — all `Dockerfile*` files in the stack's directory are discovered automatically (`naive`, `optimized`, or custom)
- **Duration** — in seconds (5–3600)
- **Virtual Users** — k6 VU count
- **Operation Mix** — four sliders (Create / Read / Update / Delete) with normalized percentage preview and visual bar. Uses `k6/crud-mix.js` under the hood; `k6/crud.js` is unchanged.

Submitting adds the job to the queue and redirects to the Queue page.

### Queue (`/queue`) — requires control server

Live queue view:

- Status badges (Pending / Running / Done / Failed / Canceled)
- Running job pulses with an animated border
- Click any job to show its log in the side panel
- Log panel with auto-scroll toggle streams `run.sh` stdout/stderr in real time via SSE
- Cancel buttons for pending jobs
- "View results →" deep-link on completed jobs (opens Single Run for that run)

## Production build

```bash
npm run build
# output in dist/
npx serve dist
```

The control server must still be running separately for the Run Tests and Queue pages to work.

## Data pipeline

```
results/<run>/  →  scripts/build-data.mjs  →  public/data/index.json
                                            →  public/data/runs/<run_id>.json
```

`build-data.mjs` now also passes `k6_script` and `mix` through to the index and per-run JSON so they appear in the Single Run badges.

When the control server finishes a queued run, it regenerates `public/data/` and pushes a `data_updated` SSE event. The viewer subscribes globally and reloads the index (and any open run detail) automatically.

`public/data/`, `node_modules/`, and `dist/` are all gitignored.

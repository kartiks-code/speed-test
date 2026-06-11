# Petstore Benchmark Viewer

A React + Vite single-page app for exploring and comparing benchmark run results.

## Prerequisites

- Node.js 18+
- Benchmark results in `performance-tests/results/` (at least one completed run with `run-meta.json`)

## Quick start

```bash
cd performance-tests/viewer
npm install
npm run dev
```

Open the URL printed by Vite (typically `http://localhost:5173`).

The data generator (`scripts/build-data.mjs`) runs automatically before `dev` and `build`. It scans `../results/` and writes static JSON to `public/data/`. Re-run `npm run dev` (or `node scripts/build-data.mjs` manually) after adding new benchmark runs.

## Pages

### Single Run (`/`)

Pick a stack from the first dropdown, then a specific run from the second. Charts shown:

- Stat cards: RPS, avg/p95/p99 latency, error rate, total requests, CPU peak, RAM peak
- Latency breakdown bar chart (avg, p50, p90, p95, p99, max)
- Resource summary table (CPU avg/peak, RAM avg/peak, net, block I/O)
- CPU usage over run (time series)
- RAM usage over run (time series)
- Per-endpoint checks (passes vs fails per k6 check)
- PostgreSQL counters delta (xacts, blks read/hit, tup counts)

### Compare (`/compare`)

Select up to 4 stack/run combinations. Each series gets a distinct color. Charts:

- RPS comparison
- Error rate comparison
- Latency percentiles (p50/p90/p95/p99) — grouped bars
- CPU usage (avg/peak) — grouped bars
- RAM usage (avg/peak) — grouped bars
- PostgreSQL counters — grouped bars
- CPU usage over time — line overlay
- RAM usage over time — line overlay

## Production build

```bash
npm run build
# output in dist/
npx serve dist
```

## Data pipeline

```
results/<run>/  →  scripts/build-data.mjs  →  public/data/index.json
                                            →  public/data/runs/<run_id>.json
```

`public/data/` and `node_modules/` and `dist/` are gitignored.

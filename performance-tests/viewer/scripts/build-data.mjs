#!/usr/bin/env node
/**
 * build-data.mjs — scans ../results/ and emits public/data/index.json
 * plus public/data/runs/<run_id>.json for each completed run.
 *
 * Mirrors the parsing logic in performance-tests/report.py.
 * No external dependencies; uses Node built-ins only.
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RESULTS_DIR = path.resolve(__dirname, "../../results");
const OUT_DIR = path.resolve(__dirname, "../public/data");
const RUNS_DIR = path.join(OUT_DIR, "runs");

// ── helpers ──────────────────────────────────────────────────────────────────

function readJson(p) {
  try {
    return JSON.parse(fs.readFileSync(p, "utf8"));
  } catch {
    return null;
  }
}

function parseCsv(text) {
  const lines = text.trim().split("\n");
  if (lines.length < 2) return [];
  const headers = lines[0].split(",").map((h) => h.trim());
  return lines.slice(1).map((line) => {
    const vals = line.split(",");
    const row = {};
    headers.forEach((h, i) => (row[h] = vals[i]?.trim() ?? ""));
    return row;
  });
}

function mean(arr) {
  return arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : null;
}
function max(arr) {
  return arr.length ? Math.max(...arr) : null;
}
function round(v, d = 3) {
  return v == null ? null : Math.round(v * 10 ** d) / 10 ** d;
}

// ── k6 summary ───────────────────────────────────────────────────────────────

function parseK6Summary(runDir) {
  const data = readJson(path.join(runDir, "k6-summary.json"));
  if (!data) return {};

  const metrics = data.metrics || {};

  const mval = (key, stat) => metrics[key]?.[stat] ?? null;

  // http_req_failed: passes = failed requests, fails = successful (inverted k6 semantics)
  const failedM = metrics.http_req_failed || {};
  const passes = failedM.passes || 0;
  const fails = failedM.fails || 0;
  const totalForRate = passes + fails;
  const errorRate = totalForRate > 0 ? round(passes / totalForRate, 6) : null;

  // per-endpoint check counts from root_group.checks
  const checks = data.root_group?.checks || {};
  const endpoints = Object.values(checks).map((c) => ({
    name: c.name,
    passes: c.passes ?? 0,
    fails: c.fails ?? 0,
  }));

  return {
    k6_rps: mval("http_reqs", "rate"),
    k6_avg_ms: mval("http_req_duration", "avg"),
    k6_p50_ms: mval("http_req_duration", "med"),
    k6_p90_ms: mval("http_req_duration", "p(90)"),
    k6_p95_ms: mval("http_req_duration", "p(95)"),
    k6_p99_ms: mval("http_req_duration", "p(99)"),
    k6_max_ms: mval("http_req_duration", "max"),
    k6_error_rate: errorRate,
    k6_total_requests: mval("http_reqs", "count"),
    endpoints,
  };
}

// ── docker-stats CSV ──────────────────────────────────────────────────────────

function parseDockerStats(runDir, appContainer) {
  let text;
  try {
    text = fs.readFileSync(path.join(runDir, "docker-stats.csv"), "utf8");
  } catch {
    return {};
  }

  const rows = parseCsv(text).filter((r) => r.container === appContainer);
  if (!rows.length) return {};

  const cpuVals = [];
  const memVals = [];
  const netRxVals = [];
  const netTxVals = [];
  const blkRVals = [];
  const blkWVals = [];
  const tsVals = [];

  for (const r of rows) {
    const cpu = parseFloat(r.cpu_percent);
    const mem = parseFloat(r.mem_usage_mb);
    const netRx = parseFloat(r.net_rx_mb);
    const netTx = parseFloat(r.net_tx_mb);
    const blkR = parseFloat(r.blk_read_mb);
    const blkW = parseFloat(r.blk_write_mb);
    if (isNaN(cpu)) continue;
    cpuVals.push(cpu);
    memVals.push(mem);
    netRxVals.push(netRx);
    netTxVals.push(netTx);
    blkRVals.push(blkR);
    blkWVals.push(blkW);
    tsVals.push(r.timestamp);
  }

  // Build time-series: seconds from first sample
  let firstMs = null;
  const timeseries = rows
    .filter((r) => r.container === appContainer)
    .map((r, i) => {
      const ts = new Date(r.timestamp).getTime();
      if (firstMs == null) firstMs = ts;
      return {
        t: round((ts - firstMs) / 1000, 1),
        cpu_pct: round(parseFloat(r.cpu_percent), 2),
        mem_mb: round(parseFloat(r.mem_usage_mb), 2),
        net_rx_mb: round(parseFloat(r.net_rx_mb), 3),
        net_tx_mb: round(parseFloat(r.net_tx_mb), 3),
      };
    })
    .filter((p) => !isNaN(p.cpu_pct));

  return {
    cpu_avg_pct: round(mean(cpuVals)),
    cpu_peak_pct: round(max(cpuVals)),
    mem_avg_mb: round(mean(memVals)),
    mem_peak_mb: round(max(memVals)),
    net_rx_total_mb: round(max(netRxVals)),
    net_tx_total_mb: round(max(netTxVals)),
    blk_read_total_mb: round(max(blkRVals)),
    blk_write_total_mb: round(max(blkWVals)),
    timeseries,
  };
}

// ── pg delta ─────────────────────────────────────────────────────────────────

function parsePgDelta(runDir) {
  const data = readJson(path.join(runDir, "pg-delta.json"));
  if (!data) return {};
  const d = data.db_stats_delta || {};
  return {
    pg_xact_commit: d.xact_commit ?? null,
    pg_xact_rollback: d.xact_rollback ?? null,
    pg_blks_read: d.blks_read ?? null,
    pg_blks_hit: d.blks_hit ?? null,
    pg_tup_inserted: d.tup_inserted ?? null,
    pg_tup_updated: d.tup_updated ?? null,
    pg_tup_deleted: d.tup_deleted ?? null,
    pg_tup_fetched: d.tup_fetched ?? null,
    pg_temp_files: d.temp_files ?? null,
    pg_deadlocks: d.deadlocks ?? null,
  };
}

// ── load one run directory ────────────────────────────────────────────────────

function loadRun(runDir) {
  const metaPath = path.join(runDir, "run-meta.json");
  if (!fs.existsSync(metaPath)) return null;

  const meta = readJson(metaPath);
  if (!meta) return null;

  const stackId = meta.stack_id || "";
  const variant = meta.variant || "";
  const appContainer = `speed-test-app-${stackId}-${variant}`;

  const k6 = parseK6Summary(runDir);
  const { timeseries, ...resources } = parseDockerStats(runDir, appContainer);
  const pg = parsePgDelta(runDir);

  const summary = {
    run_id: meta.run_id || path.basename(runDir),
    stack_id: stackId,
    label: meta.label || stackId,
    variant,
    timestamp: meta.timestamp || "",
    vus: meta.vus ?? null,
    duration: meta.duration || "",
    app_cpus: meta.app_cpus ?? null,
    app_memory: meta.app_memory || "",
    k6_rps: k6.k6_rps,
    k6_avg_ms: k6.k6_avg_ms,
    k6_p50_ms: k6.k6_p50_ms,
    k6_p90_ms: k6.k6_p90_ms,
    k6_p95_ms: k6.k6_p95_ms,
    k6_p99_ms: k6.k6_p99_ms,
    k6_max_ms: k6.k6_max_ms,
    k6_error_rate: k6.k6_error_rate,
    k6_total_requests: k6.k6_total_requests,
    cpu_avg_pct: resources.cpu_avg_pct,
    cpu_peak_pct: resources.cpu_peak_pct,
    mem_avg_mb: resources.mem_avg_mb,
    mem_peak_mb: resources.mem_peak_mb,
  };

  const detail = {
    meta: {
      run_id: summary.run_id,
      stack_id: stackId,
      label: summary.label,
      variant,
      timestamp: summary.timestamp,
      vus: summary.vus,
      duration: summary.duration,
      app_cpus: summary.app_cpus,
      app_memory: summary.app_memory,
    },
    k6: {
      rps: k6.k6_rps,
      avg_ms: k6.k6_avg_ms,
      p50_ms: k6.k6_p50_ms,
      p90_ms: k6.k6_p90_ms,
      p95_ms: k6.k6_p95_ms,
      p99_ms: k6.k6_p99_ms,
      max_ms: k6.k6_max_ms,
      error_rate: k6.k6_error_rate,
      total_requests: k6.k6_total_requests,
    },
    endpoints: k6.endpoints || [],
    pg,
    resources,
    timeseries: timeseries || [],
  };

  return { summary, detail };
}

// ── main ─────────────────────────────────────────────────────────────────────

function main() {
  if (!fs.existsSync(RESULTS_DIR)) {
    console.warn(`[build-data] Results directory not found: ${RESULTS_DIR} — creating empty index.`);
    fs.mkdirSync(OUT_DIR, { recursive: true });
    fs.mkdirSync(RUNS_DIR, { recursive: true });
    fs.writeFileSync(
      path.join(OUT_DIR, "index.json"),
      JSON.stringify({ generatedAt: new Date().toISOString(), runs: [] }, null, 2)
    );
    return;
  }

  fs.mkdirSync(OUT_DIR, { recursive: true });
  fs.mkdirSync(RUNS_DIR, { recursive: true });

  const entries = fs
    .readdirSync(RESULTS_DIR, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .map((e) => e.name)
    .sort();

  const summaries = [];

  for (const name of entries) {
    const runDir = path.join(RESULTS_DIR, name);
    const result = loadRun(runDir);
    if (!result) continue;

    const { summary, detail } = result;
    summaries.push(summary);

    const detailPath = path.join(RUNS_DIR, `${summary.run_id}.json`);
    fs.writeFileSync(detailPath, JSON.stringify(detail, null, 2));
    console.log(`[build-data] wrote runs/${summary.run_id}.json`);
  }

  const index = { generatedAt: new Date().toISOString(), runs: summaries };
  fs.writeFileSync(path.join(OUT_DIR, "index.json"), JSON.stringify(index, null, 2));
  console.log(`[build-data] index.json — ${summaries.length} run(s)`);
}

main();

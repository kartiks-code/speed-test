#!/usr/bin/env python3
"""
report.py — Aggregate all benchmark run results into a comparison CSV and Markdown table.

Usage:
    python3 report.py [--results-dir <path>] [--output-csv <path>] [--output-md <path>]

Defaults:
    --results-dir  performance-tests/results/
    --output-csv   performance-tests/results/comparison.csv
    --output-md    performance-tests/results/comparison.md

Each subdirectory in results/ that contains run-meta.json is treated as a run.
Files consumed per run:
    run-meta.json    — stack id, variant, vus, duration, limits
    k6-summary.json  — RPS, latency percentiles, error rate (k6 --summary-export format)
    docker-stats.csv — per-second CPU%, RAM, network, block IO per container
    pg-delta.json    — DB xacts, blks_read, blks_hit, tup_inserted/updated/deleted
"""

import argparse
import csv
import json
import math
import os
import sys
from pathlib import Path
from statistics import mean, median


# ── k6 summary parsing ────────────────────────────────────────────────────────

def parse_k6_summary(path: Path) -> dict:
    """Parse k6 --summary-export JSON.

    k6 emits metrics as flat objects directly under "metrics": each metric's
    stats (avg, p(95), rate, count, …) are top-level keys of the metric object.
    """
    try:
        with open(path) as f:
            data = json.load(f)
    except Exception:
        return {}

    if not data:
        return {}

    metrics = data.get("metrics", {})

    def mval(key, stat):
        return metrics.get(key, {}).get(stat)

    # http_req_failed is a Rate metric; k6 serialises it as {passes, fails}
    # where passes = failed requests, fails = successful requests (confusingly).
    failed_m = metrics.get("http_req_failed", {})
    passes = failed_m.get("passes", 0)
    fails  = failed_m.get("fails", 0)
    total_for_rate = passes + fails
    error_rate = round(passes / total_for_rate, 6) if total_for_rate > 0 else None

    return {
        "k6_rps":            mval("http_reqs", "rate"),
        "k6_p50_ms":         mval("http_req_duration", "med"),
        "k6_p90_ms":         mval("http_req_duration", "p(90)"),
        "k6_p95_ms":         mval("http_req_duration", "p(95)"),
        "k6_p99_ms":         mval("http_req_duration", "p(99)"),
        "k6_avg_ms":         mval("http_req_duration", "avg"),
        "k6_max_ms":         mval("http_req_duration", "max"),
        "k6_error_rate":     error_rate,
        "k6_total_requests": mval("http_reqs",         "count"),
    }


# ── docker stats CSV parsing ───────────────────────────────────────────────────

def parse_docker_stats(path: Path, app_container: str) -> dict:
    """Summarise per-second docker stats CSV for the app container."""
    cpu_vals = []
    mem_vals = []
    net_rx_vals = []
    net_tx_vals = []
    blk_read_vals = []
    blk_write_vals = []

    try:
        with open(path, newline="") as f:
            reader = csv.DictReader(f)
            for row in reader:
                if row.get("container", "").strip() != app_container:
                    continue
                try:
                    cpu_vals.append(float(row["cpu_percent"]))
                    mem_vals.append(float(row["mem_usage_mb"]))
                    net_rx_vals.append(float(row["net_rx_mb"]))
                    net_tx_vals.append(float(row["net_tx_mb"]))
                    blk_read_vals.append(float(row["blk_read_mb"]))
                    blk_write_vals.append(float(row["blk_write_mb"]))
                except (ValueError, KeyError):
                    continue
    except FileNotFoundError:
        return {}

    def safe_mean(lst):
        return round(mean(lst), 3) if lst else None

    def safe_max(lst):
        return round(max(lst), 3) if lst else None

    # Network and block IO are cumulative counters — take max as total transferred
    return {
        "cpu_avg_pct":    safe_mean(cpu_vals),
        "cpu_peak_pct":   safe_max(cpu_vals),
        "mem_avg_mb":     safe_mean(mem_vals),
        "mem_peak_mb":    safe_max(mem_vals),
        "net_rx_total_mb": safe_max(net_rx_vals),
        "net_tx_total_mb": safe_max(net_tx_vals),
        "blk_read_total_mb":  safe_max(blk_read_vals),
        "blk_write_total_mb": safe_max(blk_write_vals),
    }


# ── pg delta parsing ──────────────────────────────────────────────────────────

def parse_pg_delta(path: Path) -> dict:
    try:
        with open(path) as f:
            data = json.load(f)
    except Exception:
        return {}

    delta = data.get("db_stats_delta", {}) or {}
    return {
        "pg_xact_commit":   delta.get("xact_commit"),
        "pg_xact_rollback": delta.get("xact_rollback"),
        "pg_blks_read":     delta.get("blks_read"),
        "pg_blks_hit":      delta.get("blks_hit"),
        "pg_tup_inserted":  delta.get("tup_inserted"),
        "pg_tup_updated":   delta.get("tup_updated"),
        "pg_tup_deleted":   delta.get("tup_deleted"),
        "pg_tup_fetched":   delta.get("tup_fetched"),
        "pg_temp_files":    delta.get("temp_files"),
        "pg_deadlocks":     delta.get("deadlocks"),
    }


# ── load one run directory ─────────────────────────────────────────────────────

def load_run(run_dir: Path) -> dict | None:
    meta_path = run_dir / "run-meta.json"
    if not meta_path.exists():
        return None

    with open(meta_path) as f:
        meta = json.load(f)

    stack_id = meta.get("stack_id", "")
    variant  = meta.get("variant", "")
    app_container = f"speed-test-app-{stack_id}-{variant}"

    row = {
        "run_id":      meta.get("run_id", run_dir.name),
        "stack":       meta.get("label", stack_id),
        "variant":     variant,
        "vus":         meta.get("vus"),
        "duration":    meta.get("duration"),
        "app_cpus":    meta.get("app_cpus"),
        "app_memory":  meta.get("app_memory"),
        "timestamp":   meta.get("timestamp"),
    }

    row.update(parse_k6_summary(run_dir / "k6-summary.json"))
    row.update(parse_docker_stats(run_dir / "docker-stats.csv", app_container))
    row.update(parse_pg_delta(run_dir / "pg-delta.json"))

    return row


# ── columns ───────────────────────────────────────────────────────────────────

COLUMNS = [
    "run_id", "stack", "variant", "timestamp",
    "vus", "duration", "app_cpus", "app_memory",
    # k6
    "k6_rps", "k6_avg_ms", "k6_p50_ms", "k6_p90_ms", "k6_p95_ms", "k6_p99_ms",
    "k6_max_ms", "k6_error_rate", "k6_total_requests",
    # docker
    "cpu_avg_pct", "cpu_peak_pct",
    "mem_avg_mb", "mem_peak_mb",
    "net_rx_total_mb", "net_tx_total_mb",
    "blk_read_total_mb", "blk_write_total_mb",
    # postgres
    "pg_xact_commit", "pg_xact_rollback",
    "pg_blks_read", "pg_blks_hit",
    "pg_tup_inserted", "pg_tup_updated", "pg_tup_deleted", "pg_tup_fetched",
    "pg_temp_files", "pg_deadlocks",
]


# ── markdown table rendering ──────────────────────────────────────────────────

DISPLAY_COLUMNS = [
    ("stack",         "Stack"),
    ("variant",       "Variant"),
    ("k6_rps",        "RPS"),
    ("k6_avg_ms",     "Avg ms"),
    ("k6_p95_ms",     "p95 ms"),
    ("k6_p99_ms",     "p99 ms"),
    ("k6_error_rate", "Err%"),
    ("cpu_avg_pct",   "CPU avg%"),
    ("cpu_peak_pct",  "CPU peak%"),
    ("mem_avg_mb",    "RAM avg MB"),
    ("mem_peak_mb",   "RAM peak MB"),
    ("pg_xact_commit","PG xacts"),
    ("pg_blks_read",  "PG blks read"),
    ("pg_blks_hit",   "PG blks hit"),
]


def fmt(val) -> str:
    if val is None:
        return "—"
    if isinstance(val, float):
        return f"{val:.2f}"
    return str(val)


def render_markdown(rows: list[dict]) -> str:
    if not rows:
        return "_No results found._\n"

    headers = [col for _, col in DISPLAY_COLUMNS]
    keys    = [key for key, _ in DISPLAY_COLUMNS]

    lines = []
    lines.append("| " + " | ".join(headers) + " |")
    lines.append("| " + " | ".join(["---"] * len(headers)) + " |")

    for row in sorted(rows, key=lambda r: (r.get("stack", ""), r.get("variant", ""))):
        cells = [fmt(row.get(k)) for k in keys]
        lines.append("| " + " | ".join(cells) + " |")

    return "\n".join(lines) + "\n"


# ── main ──────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Aggregate benchmark results into CSV and Markdown")
    parser.add_argument("--results-dir", default=None, help="Path to results/ directory")
    parser.add_argument("--output-csv",  default=None, help="Output CSV path")
    parser.add_argument("--output-md",   default=None, help="Output Markdown path")
    args = parser.parse_args()

    script_dir   = Path(__file__).parent
    results_dir  = Path(args.results_dir) if args.results_dir else script_dir / "results"
    output_csv   = Path(args.output_csv)  if args.output_csv  else results_dir / "comparison.csv"
    output_md    = Path(args.output_md)   if args.output_md   else results_dir / "comparison.md"

    if not results_dir.exists():
        print(f"[report] Results directory not found: {results_dir}", file=sys.stderr)
        sys.exit(1)

    rows = []
    for entry in sorted(results_dir.iterdir()):
        if not entry.is_dir():
            continue
        row = load_run(entry)
        if row:
            rows.append(row)

    if not rows:
        print(f"[report] No completed run directories found in {results_dir}", file=sys.stderr)
        sys.exit(1)

    # Write CSV
    output_csv.parent.mkdir(parents=True, exist_ok=True)
    with open(output_csv, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=COLUMNS, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)
    print(f"[report] CSV  → {output_csv}")

    # Write Markdown
    md = f"# Petstore Benchmark Results\n\n"
    md += f"_Generated from {len(rows)} run(s) in `{results_dir}`_\n\n"
    md += render_markdown(rows)
    md += "\n\n## Column Definitions\n\n"
    md += "| Column | Description |\n|---|---|\n"
    md += "| RPS | HTTP requests/second (k6 `http_reqs` rate) |\n"
    md += "| Avg ms / p95 ms / p99 ms | Request duration percentiles from k6 |\n"
    md += "| Err% | HTTP error rate (non-2xx) |\n"
    md += "| CPU avg% / peak% | Container CPU usage averaged/peaked over the run |\n"
    md += "| RAM avg MB / peak MB | Container RSS memory during the run |\n"
    md += "| PG xacts | Postgres transactions committed (pg_stat_database delta) |\n"
    md += "| PG blks read / hit | Postgres block reads from disk vs buffer cache |\n"

    with open(output_md, "w") as f:
        f.write(md)
    print(f"[report] MD   → {output_md}")
    print(f"[report] {len(rows)} run(s) aggregated.")


if __name__ == "__main__":
    main()

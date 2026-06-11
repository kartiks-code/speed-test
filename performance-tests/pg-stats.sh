#!/usr/bin/env bash
# pg-stats.sh — Snapshot, diff, and reset Postgres statistics for a single database.
#
# Usage:
#   pg-stats.sh snapshot <db_name> <output_file>   # capture current stats to JSON
#   pg-stats.sh delta    <before_file> <after_file> <output_file>  # compute deltas
#   pg-stats.sh reset    <db_name>                 # reset stats counters for this DB
#   pg-stats.sh enable-extension                   # create pg_stat_statements extension
#
# Environment variables (forwarded to psql):
#   PGHOST, PGPORT, PGUSER, PGPASSWORD  (defaults: localhost 5432 myuser mypassword)
#
# The snapshot output is a JSON object with keys:
#   db_stats    — row from pg_stat_database for the given db_name
#   io_stats    — rows from pg_stat_io for the given db_name's backends (PG17+)
#   statements  — top-50 rows from pg_stat_statements ordered by total_exec_time
#   captured_at — ISO timestamp

set -euo pipefail

: "${PGHOST:=localhost}"
: "${PGPORT:=5434}"
: "${PGUSER:=myuser}"
: "${PGPASSWORD:=mypassword}"
export PGPASSWORD

PSQL="psql -h $PGHOST -p $PGPORT -U $PGUSER -At"
PSQL_JSON="psql -h $PGHOST -p $PGPORT -U $PGUSER -At -c"

# ── helpers ────────────────────────────────────────────────────────────────

psql_json_query() {
    # Run a query that returns a single JSON value (use row_to_json / json_agg)
    local db="$1"
    local query="$2"
    psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$db" -At -c "$query" 2>/dev/null || echo "null"
}

# ── subcommands ────────────────────────────────────────────────────────────

cmd_snapshot() {
    local db_name="$1"
    local output="$2"

    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    # pg_stat_database for this database
    local db_stats
    db_stats=$(psql_json_query "postgres" "
        SELECT row_to_json(s) FROM (
            SELECT datname, numbackends, xact_commit, xact_rollback,
                   blks_read, blks_hit, tup_returned, tup_fetched,
                   tup_inserted, tup_updated, tup_deleted,
                   temp_files, temp_bytes, deadlocks,
                   blk_read_time, blk_write_time
            FROM pg_stat_database
            WHERE datname = '${db_name}'
        ) s;
    ")

    # pg_stat_io (PG17 only — graceful fallback to null)
    local io_stats
    io_stats=$(psql_json_query "postgres" "
        SELECT COALESCE(json_agg(s), '[]'::json) FROM (
            SELECT backend_type, object, context, reads, read_time,
                   writes, write_time, writebacks, writeback_time,
                   extends, extend_time, hits, evictions, reuses, fsyncs, fsync_time
            FROM pg_stat_io
            WHERE backend_type IN ('client backend', 'autovacuum worker', 'background writer', 'checkpointer')
            ORDER BY backend_type, object, context
        ) s;
    " 2>/dev/null || echo "null")

    # pg_stat_statements top 50 by total exec time (only if extension exists)
    local statements
    statements=$(psql_json_query "$db_name" "
        SELECT COALESCE(json_agg(s), '[]'::json) FROM (
            SELECT left(query, 120) AS query_snippet,
                   calls, total_exec_time, mean_exec_time,
                   rows, shared_blks_hit, shared_blks_read,
                   shared_blks_written, local_blks_hit, local_blks_read,
                   temp_blks_read, temp_blks_written
            FROM pg_stat_statements
            ORDER BY total_exec_time DESC
            LIMIT 50
        ) s;
    " 2>/dev/null || echo "null")

    jq -n \
        --arg captured_at "$ts" \
        --argjson db_stats "$db_stats" \
        --argjson io_stats "$io_stats" \
        --argjson statements "$statements" \
        '{captured_at: $captured_at, db_stats: $db_stats, io_stats: $io_stats, statements: $statements}' \
        > "$output"

    echo "[pg-stats] Snapshot written to $output"
}

cmd_delta() {
    local before="$1"
    local after="$2"
    local output="$3"

    # Compute numeric deltas between before/after db_stats fields
    python3 - "$before" "$after" "$output" <<'PYEOF'
import json, sys

before_file, after_file, out_file = sys.argv[1], sys.argv[2], sys.argv[3]

with open(before_file) as f:
    before = json.load(f)
with open(after_file) as f:
    after = json.load(f)

def delta_obj(b, a):
    if not isinstance(b, dict) or not isinstance(a, dict):
        return a
    result = {}
    for k, av in a.items():
        bv = b.get(k)
        if isinstance(av, (int, float)) and isinstance(bv, (int, float)):
            result[k] = av - bv
        else:
            result[k] = av
    return result

db_delta = delta_obj(before.get("db_stats") or {}, after.get("db_stats") or {})

result = {
    "before_captured_at": before["captured_at"],
    "after_captured_at": after["captured_at"],
    "db_stats_delta": db_delta,
    "io_stats_after": after.get("io_stats"),
    "statements_after": after.get("statements"),
}
with open(out_file, "w") as f:
    json.dump(result, f, indent=2)
print(f"[pg-stats] Delta written to {out_file}")
PYEOF
}

cmd_reset() {
    local db_name="$1"

    # Reset per-database counters
    psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "postgres" -At -c \
        "SELECT pg_stat_reset_single_table_counters(oid) FROM pg_database WHERE datname='${db_name}';" \
        > /dev/null 2>&1 || true

    # pg_stat_reset() resets the stats for the *current* database connection
    psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$db_name" -At -c \
        "SELECT pg_stat_reset();" > /dev/null 2>&1 || true

    # pg_stat_statements_reset for this database
    psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$db_name" -At -c \
        "SELECT pg_stat_statements_reset();" > /dev/null 2>&1 || true

    echo "[pg-stats] Stats reset for database '${db_name}'"
}

cmd_enable_extension() {
    psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "postgres" -At -c \
        "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;" > /dev/null 2>&1 || true
    echo "[pg-stats] pg_stat_statements extension ensured on postgres DB"
}

# ── dispatch ────────────────────────────────────────────────────────────────

case "${1:-}" in
    snapshot)
        [[ $# -ge 3 ]] || { echo "Usage: pg-stats.sh snapshot <db_name> <output_file>"; exit 1; }
        cmd_snapshot "$2" "$3"
        ;;
    delta)
        [[ $# -ge 4 ]] || { echo "Usage: pg-stats.sh delta <before> <after> <output>"; exit 1; }
        cmd_delta "$2" "$3" "$4"
        ;;
    reset)
        [[ $# -ge 2 ]] || { echo "Usage: pg-stats.sh reset <db_name>"; exit 1; }
        cmd_reset "$2"
        ;;
    enable-extension)
        cmd_enable_extension
        ;;
    *)
        echo "Usage: $0 {snapshot|delta|reset|enable-extension} [args...]"
        exit 1
        ;;
esac

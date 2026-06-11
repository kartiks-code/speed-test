#!/usr/bin/env bash
# run.sh — Benchmark orchestrator for the Petstore multi-stack speed test.
#
# Usage:
#   ./run.sh all                         # run all stacks, both variants
#   ./run.sh all naive                   # all stacks, naive Dockerfile only
#   ./run.sh all optimized               # all stacks, Dockerfile.optimized only
#   ./run.sh go                          # single stack, both variants
#   ./run.sh go naive                    # single stack, single variant
#   ./run.sh go,springboot optimized     # comma-separated stacks
#
# Environment overrides:
#   VUS           number of k6 virtual users (default: 20)
#   DURATION      k6 duration, e.g. 60s, 2m (default: 60s)
#   APP_CPUS      CPU limit for app containers (default: 2)
#   APP_MEMORY    Memory limit for app containers (default: 512m)
#   PGHOST        Postgres host reachable from the *host* machine (default: localhost)
#   PGPORT        Postgres port on the host (default: 5434)
#   PGUSER        Postgres user (default: myuser)
#   PGPASSWORD    Postgres password (default: mypassword)
#   LARAVEL_APP_KEY   Required for Laravel; generate with: php artisan key:generate --show
#   RAILS_SECRET_KEY_BASE  Required for Rails optimized; generate with: ruby -rsecurerandom -e 'puts SecureRandom.hex(64)'
#   PHOENIX_SECRET_KEY_BASE  Optional for Phoenix
#   NO_BUILD      Set to 1 to skip docker build (use existing images)
#   KEEP_RESULTS  Set to 1 to keep results even on failure (default: clean up on error)
#
# Prerequisites:
#   - Docker with Unix socket at /var/run/docker.sock
#   - Python 3 (stdlib only) for sampler.py and report.py
#   - jq for pg-stats.sh JSON parsing
#   - psql client (postgresql-client) for pg-stats.sh
#   - The shared Postgres container must be running (cd database && docker compose up -d)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── defaults ────────────────────────────────────────────────────────────────
: "${VUS:=20}"
: "${DURATION:=60s}"
: "${APP_CPUS:=2}"
: "${APP_MEMORY:=512m}"
: "${PGHOST:=localhost}"
: "${PGPORT:=5434}"
: "${PGUSER:=myuser}"
: "${PGPASSWORD:=mypassword}"
: "${NO_BUILD:=0}"
: "${KEEP_RESULTS:=0}"
: "${READINESS_TIMEOUT:=90}"

export PGHOST PGPORT PGUSER PGPASSWORD

DOCKER_NETWORK="database_default"
POSTGRES_CONTAINER="speed-test-postgres"

PG_STATS="$SCRIPT_DIR/pg-stats.sh"
SAMPLER="$SCRIPT_DIR/sampler.py"
STACKS_JSON="$SCRIPT_DIR/stacks.json"
K6_SCRIPT="$SCRIPT_DIR/k6/crud.js"
RESULTS_BASE="$SCRIPT_DIR/results"

# ── helpers ─────────────────────────────────────────────────────────────────

log()  { echo "[run] $*"; }
warn() { echo "[run] WARN: $*" >&2; }
die()  { echo "[run] ERROR: $*" >&2; exit 1; }

require_cmd() {
    command -v "$1" &>/dev/null || die "Required command not found: $1. Install it and retry."
}

check_prereqs() {
    require_cmd docker
    require_cmd python3
    require_cmd jq
    require_cmd psql
    # Verify docker socket is accessible
    [[ -S /var/run/docker.sock ]] || die "/var/run/docker.sock not found. Is Docker running?"
    # Verify postgres container is up
    docker inspect --format '{{.State.Running}}' "$POSTGRES_CONTAINER" 2>/dev/null | grep -q "true" \
        || die "Postgres container '$POSTGRES_CONTAINER' is not running. Start it: cd database && docker compose up -d"
    # Verify the docker network exists
    docker network inspect "$DOCKER_NETWORK" &>/dev/null \
        || die "Docker network '$DOCKER_NETWORK' not found. Start the database compose stack first."
}

# Read a field from stacks.json for a given stack id
stack_field() {
    local id="$1" field="$2"
    jq -r --arg id "$id" --arg field "$field" '.[] | select(.id==$id) | .[$field] // empty' "$STACKS_JSON"
}

# Write a Docker --env-file for a given stack id into a temp file.
# Prints the temp file path; caller is responsible for deleting it.
write_env_file() {
    local id="$1"
    local tmp
    tmp=$(mktemp /tmp/speed-test-env-XXXXXX)

    # Inject secrets from caller's environment for stacks that need them
    case "$id" in
        laravel)
            local app_key="${LARAVEL_APP_KEY:-}"
            if [[ -z "$app_key" ]]; then
                warn "LARAVEL_APP_KEY not set — Laravel container may refuse to start. Generate with: php artisan key:generate --show"
            else
                echo "APP_KEY=${app_key}" >> "$tmp"
            fi
            ;;
        rails)
            local secret="${RAILS_SECRET_KEY_BASE:-}"
            if [[ -z "$secret" ]]; then
                warn "RAILS_SECRET_KEY_BASE not set — Rails production container may fail. Generate with: ruby -rsecurerandom -e 'puts SecureRandom.hex(64)'"
            else
                echo "SECRET_KEY_BASE=${secret}" >> "$tmp"
            fi
            ;;
        phoenix)
            local secret="${PHOENIX_SECRET_KEY_BASE:-}"
            [[ -n "$secret" ]] && echo "SECRET_KEY_BASE=${secret}" >> "$tmp"
            ;;
    esac

    # Pull env object from stacks.json; skip PLACEHOLDER values
    while IFS="=" read -r key val; do
        case "$val" in
            PLACEHOLDER_*) continue ;;
        esac
        echo "${key}=${val}" >> "$tmp"
    done < <(jq -r --arg id "$id" \
        '.[] | select(.id==$id) | .env | to_entries[] | .key + "=" + .value' \
        "$STACKS_JSON")

    echo "$tmp"
}

truncate_db_tables() {
    local db_name="$1"
    log "Truncating tables in database '$db_name'..."
    PGPASSWORD="$PGPASSWORD" psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$db_name" -At \
        -c "TRUNCATE TABLE pet, \"order\", \"user\" RESTART IDENTITY CASCADE;" 2>/dev/null \
        || warn "Could not truncate tables in '$db_name' (may not exist yet — continuing)"
}

wait_ready() {
    local container="$1"
    local readiness_url="$2"
    local timeout="${3:-60}"
    local deadline=$(( $(date +%s) + timeout ))

    log "Waiting for $container to be ready at $readiness_url (timeout: ${timeout}s)..."
    while [[ $(date +%s) -lt $deadline ]]; do
        if docker run --rm --network "$DOCKER_NETWORK" \
               --entrypoint curl \
               curlimages/curl:latest \
               -sf --max-time 2 "$readiness_url" &>/dev/null; then
            log "$container is ready"
            return 0
        fi
        sleep 2
    done
    warn "Container $container did not become ready within ${timeout}s"
    return 1
}

# ── single run ───────────────────────────────────────────────────────────────

run_one() {
    local stack_id="$1"
    local variant="$2"   # "naive" or "optimized"

    local label
    label=$(stack_field "$stack_id" "label")
    local build_context
    build_context=$(stack_field "$stack_id" "build_context")
    local db_name
    db_name=$(stack_field "$stack_id" "db_name")
    local base_path
    base_path=$(stack_field "$stack_id" "base_path")
    local readiness_path
    readiness_path=$(stack_field "$stack_id" "readiness_path")

    local dockerfile
    if [[ "$variant" == "optimized" ]]; then
        dockerfile="Dockerfile.optimized"
    else
        dockerfile="Dockerfile"
        variant="naive"
    fi

    local ts
    ts=$(date -u +"%Y%m%dT%H%M%SZ")
    local run_id="${stack_id}-${variant}-${ts}"
    local results_dir="$RESULTS_BASE/${run_id}"
    mkdir -p "$results_dir"

    local image_name="speed-test-${stack_id}-${variant}"
    local container_name="speed-test-app-${stack_id}-${variant}"

    log "═══════════════════════════════════════════════════════"
    log "Stack: $label  |  Variant: $variant  |  Run ID: $run_id"
    log "═══════════════════════════════════════════════════════"

    # ── 1. Build ──────────────────────────────────────────────────────────
    if [[ "$NO_BUILD" == "0" ]]; then
        log "Building $image_name from $build_context/$dockerfile ..."
        docker build \
            -f "$REPO_ROOT/$build_context/$dockerfile" \
            -t "$image_name" \
            "$REPO_ROOT/$build_context" \
            2>&1 | tee "$results_dir/build.log"
        log "Build complete"
    else
        log "Skipping build (NO_BUILD=1)"
    fi

    # Cleanup any stale container from previous run
    docker rm -f "$container_name" &>/dev/null || true

    # ── 2. Ensure pg_stat_statements extension ───────────────────────────
    bash "$PG_STATS" enable-extension

    # ── 3. Truncate tables + reset pg stats ──────────────────────────────
    truncate_db_tables "$db_name"
    bash "$PG_STATS" reset "$db_name"

    # ── 4. Take before snapshot ──────────────────────────────────────────
    bash "$PG_STATS" snapshot "$db_name" "$results_dir/pg-before.json"

    # ── 5. Start app container ───────────────────────────────────────────
    local env_file
    env_file=$(write_env_file "$stack_id")

    # Optional entrypoint + cmd overrides (e.g. Rails naive uses a broken db:migrate entrypoint)
    local entrypoint_override
    entrypoint_override=$(stack_field "$stack_id" "entrypoint_override")
    local entrypoint_flag=()
    if jq -e --arg id "$stack_id" '.[] | select(.id==$id) | has("entrypoint_override")' \
            "$STACKS_JSON" > /dev/null 2>&1; then
        entrypoint_flag+=("--entrypoint" "${entrypoint_override:-/bin/sh}")
    fi

    # Optional cmd override (array of strings as JSON)
    local cmd_override_args=()
    local cmd_json
    cmd_json=$(jq -r --arg id "$stack_id" \
        '.[] | select(.id==$id) | .cmd_override // empty | .[]' "$STACKS_JSON" 2>/dev/null || true)
    if [[ -n "$cmd_json" ]]; then
        while IFS= read -r arg; do
            cmd_override_args+=("$arg")
        done <<< "$cmd_json"
    fi

    log "Starting container $container_name ..."
    docker run -d \
        --name "$container_name" \
        --network "$DOCKER_NETWORK" \
        --cpus "$APP_CPUS" \
        --memory "$APP_MEMORY" \
        --env-file "$env_file" \
        "${entrypoint_flag[@]}" \
        "$image_name" \
        "${cmd_override_args[@]}"
    rm -f "$env_file"

    local app_url="http://${container_name}:8080"
    local readiness_url="${app_url}${readiness_path}"

    # ── 6. Wait for readiness ─────────────────────────────────────────────
    # Per-stack timeout from stacks.json, falling back to READINESS_TIMEOUT env var
    local stack_timeout
    stack_timeout=$(stack_field "$stack_id" "readiness_timeout")
    local effective_timeout="${stack_timeout:-$READINESS_TIMEOUT}"
    local ready=0
    wait_ready "$container_name" "$readiness_url" "$effective_timeout" && ready=1
    if [[ $ready -eq 0 ]]; then
        warn "Container $container_name not ready — saving logs and skipping run"
        docker logs "$container_name" > "$results_dir/container.log" 2>&1 || true
        docker rm -f "$container_name" &>/dev/null || true
        echo '{"error": "container_not_ready"}' > "$results_dir/k6-summary.json"
        return 1
    fi

    # ── 7. Start sampler ─────────────────────────────────────────────────
    log "Starting sampler..."
    python3 "$SAMPLER" \
        --containers "$container_name" "$POSTGRES_CONTAINER" \
        --output "$results_dir/docker-stats.csv" &
    SAMPLER_PID=$!
    log "Sampler PID: $SAMPLER_PID"

    # ── 8. Run k6 ─────────────────────────────────────────────────────────
    # Pre-create summary file so k6 (runs as non-root inside container) can write it
    touch "$results_dir/k6-summary.json"
    chmod 666 "$results_dir/k6-summary.json"

    # Optional per-stack auth header (e.g. Python needs a Bearer token)
    local auth_header
    auth_header=$(stack_field "$stack_id" "auth_header")

    log "Running k6 (VUS=$VUS DURATION=$DURATION) ..."
    # Allow non-zero exit (threshold violations) so a single stack's issues don't abort the suite
    docker run --rm \
        --network "$DOCKER_NETWORK" \
        -v "$K6_SCRIPT:/scripts/crud.js:ro" \
        -v "$results_dir:/results" \
        -e "BASE_URL=${app_url}" \
        -e "BASE_PATH=${base_path}" \
        -e "VUS=${VUS}" \
        -e "DURATION=${DURATION}" \
        ${auth_header:+-e "AUTH_HEADER=${auth_header}"} \
        grafana/k6:latest run \
            --summary-export /results/k6-summary.json \
            /scripts/crud.js \
        2>&1 | tee "$results_dir/k6.log" || true
    log "k6 complete"

    # ── 9. Stop sampler ──────────────────────────────────────────────────
    if kill -0 "$SAMPLER_PID" 2>/dev/null; then
        kill "$SAMPLER_PID"
        wait "$SAMPLER_PID" 2>/dev/null || true
    fi

    # ── 10. After snapshot + delta ────────────────────────────────────────
    bash "$PG_STATS" snapshot "$db_name" "$results_dir/pg-after.json"
    bash "$PG_STATS" delta \
        "$results_dir/pg-before.json" \
        "$results_dir/pg-after.json" \
        "$results_dir/pg-delta.json"

    # ── 11. Save container logs ───────────────────────────────────────────
    docker logs "$container_name" > "$results_dir/container.log" 2>&1 || true

    # ── 12. Teardown ──────────────────────────────────────────────────────
    docker rm -f "$container_name" &>/dev/null || true

    # ── 13. Write run metadata ────────────────────────────────────────────
    jq -n \
        --arg run_id "$run_id" \
        --arg stack_id "$stack_id" \
        --arg label "$label" \
        --arg variant "$variant" \
        --arg dockerfile "$dockerfile" \
        --arg db_name "$db_name" \
        --arg vus "$VUS" \
        --arg duration "$DURATION" \
        --arg app_cpus "$APP_CPUS" \
        --arg app_memory "$APP_MEMORY" \
        --arg ts "$ts" \
        '{run_id: $run_id, stack_id: $stack_id, label: $label, variant: $variant,
          dockerfile: $dockerfile, db_name: $db_name,
          vus: ($vus|tonumber), duration: $duration,
          app_cpus: ($app_cpus|tonumber), app_memory: $app_memory,
          timestamp: $ts}' \
        > "$results_dir/run-meta.json"

    log "Results saved to $results_dir"
    log ""
}

# ── argument parsing ─────────────────────────────────────────────────────────

parse_stack_list() {
    local arg="$1"
    if [[ "$arg" == "all" ]]; then
        jq -r '.[].id' "$STACKS_JSON"
    else
        tr ',' '\n' <<< "$arg"
    fi
}

parse_variants() {
    local arg="${1:-both}"
    case "$arg" in
        naive)      echo "naive" ;;
        optimized)  echo "optimized" ;;
        both|"")    echo -e "naive\noptimized" ;;
        *)          die "Unknown variant '$arg'. Use: naive, optimized, or omit for both." ;;
    esac
}

# ── entry point ──────────────────────────────────────────────────────────────

main() {
    [[ $# -ge 1 ]] || {
        echo "Usage: $0 <stack|all|stack1,stack2> [naive|optimized]"
        echo "       $0 all"
        echo "       $0 go naive"
        echo "       $0 go,springboot optimized"
        exit 1
    }

    local stacks_arg="$1"
    local variant_arg="${2:-both}"

    check_prereqs
    mkdir -p "$RESULTS_BASE"

    local failed=0
    while IFS= read -r stack_id; do
        while IFS= read -r variant; do
            run_one "$stack_id" "$variant" || (( failed++ )) || true
        done < <(parse_variants "$variant_arg")
    done < <(parse_stack_list "$stacks_arg")

    if [[ $failed -gt 0 ]]; then
        warn "$failed run(s) failed. Check logs in $RESULTS_BASE."
        exit 1
    fi

    log "All runs complete. Results in $RESULTS_BASE"
    log "Generate comparison report: python3 $SCRIPT_DIR/report.py"
}

main "$@"

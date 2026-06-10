#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# Usage: ./create-databases.sh
#
# Reads credentials from environment variables (or a .env file in the same
# directory).  Connects to the running PostgreSQL container and creates one
# database per generated server in COMMANDS.md.
#
# Environment variables:
#   POSTGRES_USER     – superuser name          (default: postgres)
#   POSTGRES_PASSWORD – superuser password      (required)
#   POSTGRES_HOST     – host                    (default: localhost)
#   POSTGRES_PORT     – port                    (default: 5432)
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Load .env if present
if [[ -f "$SCRIPT_DIR/.env" ]]; then
  # shellcheck disable=SC1091
  set -a
  source "$SCRIPT_DIR/.env"
  set +a
fi

PGUSER="${POSTGRES_USER:-postgres}"
PGPASSWORD="${POSTGRES_PASSWORD:?POSTGRES_PASSWORD is required. Set it in the environment or in database/.env}"
PGHOST="${POSTGRES_HOST:-localhost}"
PGPORT="${POSTGRES_PORT:-5432}"

export PGPASSWORD

DATABASES=(
  "go-gin-server"
  "java-springboot"
  "java-helidon"
  "java-quarkus"
  "nodejs-express"
  "rust-server"
  "python-fastapi"
)

echo "Connecting to PostgreSQL at ${PGHOST}:${PGPORT} as '${PGUSER}'"
echo ""

for db in "${DATABASES[@]}"; do
  if psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d postgres -tc \
      "SELECT 1 FROM pg_database WHERE datname = '$db'" \
      | grep -q 1; then
    echo "  [skip]   $db  (already exists)"
  else
    psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d postgres \
      -c "CREATE DATABASE \"$db\";"
    echo "  [created] $db"
  fi
done

echo ""
echo "Done."

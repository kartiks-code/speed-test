#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

set -a && source .env && set +a

export PGPASSWORD="$POSTGRES_PASSWORD"

DATABASES=(
  go-gin-server
  java-springboot
  java-helidon
  java-quarkus
  nodejs-express
  rust-server
  python-fastapi
  csharp-aspnetcore
  php-laravel
  ruby-rails
  kotlin-ktor
  elixir-phoenix
  rust-actix
  go-fiber
  nodejs-fastify
  bun-elysia
  cpp-drogon
)

for db in "${DATABASES[@]}"; do
  echo "==> $db"
  psql -h localhost -p "${POSTGRES_PORT:-5432}" \
       -U "$POSTGRES_USER" -d "$db" \
       -f postgresql_schema.sql
  psql -h localhost -p "${POSTGRES_PORT:-5432}" \
       -U "$POSTGRES_USER" -d "$db" \
       -f postgresql_schema_oauth2.sql
done

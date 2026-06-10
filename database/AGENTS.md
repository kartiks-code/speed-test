# Database Agent Guide

This directory contains the local PostgreSQL setup and generated schema files for the Speed Test services.

## Working Directory

- Run database commands from `database/` unless a command explicitly says otherwise.
- Keep `database/.env` local only. Use `database/.env.example` for documented defaults or new variables.
- The Compose service uses PostgreSQL 17 and requires `POSTGRES_PASSWORD` to be set.

## Common Commands

```bash
docker compose up -d
docker compose ps
./create-databases.sh
./apply-schemas.sh
docker compose stop
```

Use `docker compose down -v` only when a full local data reset is intended, because it removes the `postgres_data` volume.

## Database Set

Keep the database list in sync across scripts and docs:

- `go-gin-server`
- `java-springboot`
- `java-helidon`
- `java-quarkus`
- `nodejs-express`
- `rust-server`
- `python-fastapi`

The same list appears in `create-databases.sh`, `apply-schemas.sh`, and `DEVELOPER.md`.

## Schema Files

- `postgresql_schema.sql` contains the Petstore domain tables and enum types.
- `postgresql_schema_oauth2.sql` contains OAuth2-related tables.
- These schemas may be generated artifacts. Keep formatting and naming consistent with the existing SQL unless intentionally regenerating.
- Do not enable destructive `DROP` statements unless the task explicitly calls for a reset or migration rewrite.

## Shell Scripts

- Bash scripts should keep `set -euo pipefail`.
- Prefer reading credentials from environment variables or `database/.env`; do not hardcode secrets.
- Preserve idempotency where present, especially in `create-databases.sh`.

## Verification

After changing scripts or schema behavior, run the smallest relevant check:

```bash
bash -n create-databases.sh apply-schemas.sh
docker compose ps
```

When PostgreSQL is running and credentials are available, verify schema application with `./create-databases.sh` followed by `./apply-schemas.sh`.

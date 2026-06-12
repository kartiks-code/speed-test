# Database Agent Guide

This directory contains the local PostgreSQL setup and generated schema files for the Speed Test services.

## Working Directory

- Run database commands from `database/` unless a command explicitly says otherwise.
- Keep `database/.env` local only. Use `database/.env.example` for documented defaults or new variables.
- The Compose service uses PostgreSQL 17 and requires `POSTGRES_PASSWORD` to be set.
- `max_connections` is raised to `500` (via `-c max_connections=500` in `docker-compose.yml`) so the virtual-thread server stacks (Quarkus, Helidon, Spring Boot) can use large connection pools without exhausting Postgres. Changing the `command:` requires recreating the container (`docker compose up -d`), not just restarting it.

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
- `csharp-aspnetcore`
- `php-laravel`
- `ruby-rails`
- `kotlin-ktor`
- `elixir-phoenix`

The same list appears in `create-databases.sh`, `apply-schemas.sh`, and `DEVELOPER.md`.

## Schema Files

- `postgresql_schema.sql` contains the Petstore domain tables, enum types, and ID sequences.
- `postgresql_schema_oauth2.sql` contains OAuth2-related tables.
- These schemas may be generated artifacts. Keep formatting and naming consistent with the existing SQL unless intentionally regenerating.
- Do not enable destructive `DROP` statements unless the task explicitly calls for a reset or migration rewrite.

## ID Sequences

`postgresql_schema.sql` defines four Postgres sequences for server-assigned IDs:

| Sequence | Table |
|---|---|
| `pet_id_seq` | `pet` |
| `order_id_seq` | `"order"` |
| `user_id_seq` | `"user"` |
| `pet_photo_id_seq` | `pet_photo` |

Each sequence is:
- **OWNED BY** its table's `id` column — dropped automatically if the table is dropped.
- Set as the column's **DEFAULT** — so `TRUNCATE … RESTART IDENTITY` (used by the benchmark harness between runs) resets the sequence to 1.
- **Seeded** beyond existing rows on `apply-schemas.sh` re-runs via `setval(…, false)`, making the schema idempotent on populated databases.

Server implementations should call `SELECT nextval('<table>_id_seq')` instead of `SELECT COALESCE(MAX(id), 0) + 1 FROM <table>` to avoid ID collision races under concurrent load.

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

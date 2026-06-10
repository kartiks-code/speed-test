# Database — Developer Guide

This guide walks through starting the database, creating all per-server databases, and applying the shared schema migrations to each one.

---

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) (with the Compose plugin, v2+)
- `psql` client — install via your package manager:
  ```bash
  # macOS
  brew install libpq && brew link --force libpq

  # Ubuntu / Debian
  sudo apt install postgresql-client
  ```

---

## 1 — Configure credentials

Create a `.env` file in this directory (it is gitignored, never commit it):

```bash
# database/.env
POSTGRES_USER=postgres
POSTGRES_PASSWORD=mysecret
# Optional overrides:
# POSTGRES_PORT=5432
```

---

## 2 — Start PostgreSQL

From the **`database/`** directory:

```bash
docker compose up -d
```

Wait for the container to become healthy:

```bash
docker compose ps          # STATUS should show "(healthy)"
docker compose logs -f     # stream logs; Ctrl-C to exit
```

To stop the container without destroying data:

```bash
docker compose stop
```

To stop **and remove** the container and its volume (full reset):

```bash
docker compose down -v
```

---

## 3 — Create the databases

Run the creation script once the container is healthy:

```bash
./create-databases.sh
```

This creates one database for each generated server:

| Database | Server |
|---|---|
| `go-gin-server` | Go — Gin |
| `java-springboot` | Java — Spring Boot |
| `java-helidon` | Java — Helidon |
| `java-quarkus` | Java — Quarkus |
| `nodejs-express` | Node.js — Express |
| `rust-server` | Rust |
| `python-fastapi` | Python — FastAPI |

The script is idempotent — re-running it skips databases that already exist.

---

## 4 — Apply schemas to every database

Two SQL files are applied to each database:

| File | Contents |
|---|---|
| `postgresql_schema.sql` | Petstore domain tables and enum types |
| `postgresql_schema_oauth2.sql` | OAuth2 framework tables |

Run the block below from the **`database/`** directory (it reads `POSTGRES_*` from your `.env`):

```bash
set -a && source .env && set +a

DATABASES=(
  go-gin-server
  java-springboot
  java-helidon
  java-quarkus
  nodejs-express
  rust-server
  python-fastapi
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
```

### Applying to a single database

```bash
psql -h localhost -U postgres -d java-springboot -f postgresql_schema.sql
psql -h localhost -U postgres -d java-springboot -f postgresql_schema_oauth2.sql
```

---

## 5 — Connect interactively

```bash
# Connect to a specific database
psql -h localhost -U postgres -d java-springboot

# List all databases
psql -h localhost -U postgres -c '\l'

# List tables inside a database
psql -h localhost -U postgres -d java-springboot -c '\dt'
```

---

## Quick-start (all steps combined)

```bash
cd database
cp /dev/null .env               # create .env if it doesn't exist yet
echo "POSTGRES_USER=postgres" >> .env
echo "POSTGRES_PASSWORD=mysecret" >> .env

docker compose up -d
docker compose ps               # wait until healthy

./create-databases.sh

set -a && source .env && set +a
for db in go-gin-server java-springboot java-helidon java-quarkus nodejs-express rust-server python-fastapi; do
  psql -h localhost -p "${POSTGRES_PORT:-5432}" -U "$POSTGRES_USER" -d "$db" \
    -f postgresql_schema.sql -f postgresql_schema_oauth2.sql
done
```

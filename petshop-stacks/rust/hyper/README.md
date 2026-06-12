# Rust Petstore Server

OpenAPI Generator 7.23.0 (`rust-server` target) scaffolded server, extended with a full PostgreSQL-backed implementation of all 19 Petstore API operations using `sqlx` 0.8 and Tokio.

## Prerequisites

- Rust (stable toolchain via `rustup`)
- Docker (with Compose v2) for the database

## Database Setup

The server uses the `rust-server` database in the shared PostgreSQL instance.

```bash
cd ../../../database
docker compose up -d
./create-databases.sh          # creates rust-server (idempotent)
./apply-schemas.sh             # applies petstore + OAuth2 tables
cd ../../../petshop-stacks/rust/hyper
```

## Environment Variables

Copy `.env.example` to `.env` and adjust if needed (defaults match `database/.env`):

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — | Full DSN; overrides all variables below |
| `POSTGRES_HOST` / `PGHOST` | `localhost` | PostgreSQL host |
| `POSTGRES_PORT` / `PGPORT` | `5434` | Host port (matches `database/.env`) |
| `POSTGRES_USER` / `PGUSER` | `myuser` | PostgreSQL user |
| `POSTGRES_PASSWORD` / `PGPASSWORD` | `mypassword` | PostgreSQL password |
| `POSTGRES_DB` / `PGDATABASE` | `rust-server` | Database name |

```bash
cp .env.example .env
# Edit .env if your credentials differ
```

## Build and Run

```bash
# Build
cargo build --example petstore-server-server

# Run (with optional request logging)
RUST_LOG=info cargo run --example petstore-server-server
```

The server listens on `http://127.0.0.1:8080`. All routes are under `/api/v3`.

## Try It

```bash
# Add a pet
curl -s -X POST http://127.0.0.1:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}' | jq .

# Get inventory
curl -s http://127.0.0.1:8080/api/v3/store/inventory | jq .

# Create a user
curl -s -X POST http://127.0.0.1:8080/api/v3/user \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"secret","email":"alice@example.com"}' | jq .
```

## Project Layout

| Path | Purpose |
|---|---|
| `src/` | **Generated** — models, HTTP router, API trait definitions. Do not edit directly. |
| `src/server/mod.rs` | Generated HTTP dispatch layer. **Hand-patched** — see `AGENTS.md` for details. |
| `examples/server/server.rs` | **Hand-written** — implements the `Api` trait with all 19 CRUD operations (sqlx). |
| `examples/server/db.rs` | **Hand-written** — connection pool factory and DSN resolution from env. |
| `examples/server/main.rs` | Binary entry point; builds the pool and starts the server. |

## Tests

```bash
cargo test
```

Unit tests in `examples/server/server.rs` and `db.rs` run without a live database.

## Mutation Testing

```bash
# Install (one time)
cargo install cargo-mutants

# Run mutation testing (produces mutants.out/)
cargo mutants

# Faster: limit to hand-written files with 4 parallel jobs
cargo mutants --jobs 4 --file examples/server/server.rs --file examples/server/db.rs
```

See `AGENTS.md` for what is mutated vs. excluded.

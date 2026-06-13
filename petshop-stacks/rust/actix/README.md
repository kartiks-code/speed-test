# Petstore — Rust + Actix-web

A complete implementation of the [OpenAPI Petstore v3](../../spec/petstore-31.yaml) API using **Rust**, **Actix-web 4**, and **sqlx** for async PostgreSQL access.

All 19 Petstore operations are implemented and backed by the shared PostgreSQL 17 database container.

## Quick Start

### Prerequisites

- Rust (stable, edition 2021)
- PostgreSQL 17 running via the shared Docker Compose stack (see `database/`)

### 1. Start the database

```bash
cd ../../database
docker compose up -d
./create-databases.sh
./apply-schemas.sh
```

### 2. Build and run

```bash
# Development
cargo run

# Release
cargo build --release
./target/release/petstore-actix
```

The server listens on `http://0.0.0.0:8080` by default. All routes are prefixed with `/api/v3`.

### 3. Run tests

Unit tests are pure and require no database:

```bash
cargo test
```

## Configuration

| Variable | Default | Description |
|---|---|---|
| `BIND_ADDR` | `0.0.0.0:8080` | Listen address |
| `DATABASE_URL` | _(constructed)_ | Full PostgreSQL DSN — overrides all `POSTGRES_*` vars |
| `POSTGRES_HOST` | `localhost` | DB host |
| `POSTGRES_PORT` | `5434` | DB port |
| `POSTGRES_USER` | `myuser` | DB user |
| `POSTGRES_PASSWORD` | `mypassword` | DB password |
| `POSTGRES_DB` | `rust-actix` | DB name |
| `DB_POOL_MAX` | `10` | Max connection pool size |

## Docker

```bash
# Naive build
docker build -t petstore-actix .

# Optimized build (uses cargo-chef for layer caching)
docker build -f Dockerfile.optimized -t petstore-actix:optimized .

# Run
docker run -p 8080:8080 \
  -e POSTGRES_HOST=host.docker.internal \
  -e POSTGRES_PORT=5434 \
  petstore-actix
```

## API Overview

All endpoints are at `http://localhost:8080/api/v3`:

| Operation | Method | Path |
|---|---|---|
| Add pet | `POST` | `/pet` |
| Update pet | `PUT` | `/pet` |
| Find by status | `GET` | `/pet/findByStatus?status=available` |
| Find by tags | `GET` | `/pet/findByTags?tags=tag1&tags=tag2` |
| Get pet | `GET` | `/pet/{petId}` |
| Update with form | `POST` | `/pet/{petId}` |
| Delete pet | `DELETE` | `/pet/{petId}` |
| Upload image | `POST` | `/pet/{petId}/uploadImage` |
| Get inventory | `GET` | `/store/inventory` |
| Place order | `POST` | `/store/order` |
| Get order | `GET` | `/store/order/{orderId}` |
| Delete order | `DELETE` | `/store/order/{orderId}` |
| Create user | `POST` | `/user` |
| Create users list | `POST` | `/user/createWithList` |
| Login | `GET` | `/user/login?username=u&password=p` |
| Logout | `GET` | `/user/logout` |
| Get user | `GET` | `/user/{username}` |
| Update user | `PUT` | `/user/{username}` |
| Delete user | `DELETE` | `/user/{username}` |

## Mutation Testing

```bash
cargo install cargo-mutants
cargo mutants
```

Mutations are scoped to `src/helpers.rs` and `src/db_config.rs` (pure logic, no DB needed).

## Project Structure

```
src/
├── main.rs           — Actix-web app and routing
├── models.rs         — Pet, Order, User, enums
├── helpers.rs        — Row-to-model converters (+ unit tests)
├── db_config.rs      — DSN builder (+ unit tests)
└── handlers/
    ├── mod.rs        — next_id helper
    ├── pet.rs        — Pet handlers
    ├── store.rs      — Store handlers
    └── user.rs       — User handlers
```

# Speed Test — Multi-Language Petstore API

A benchmark/comparison workspace that implements the **same OpenAPI Petstore API in several languages and frameworks**, each backed by the **same shared PostgreSQL** instance. The goal is an apples-to-apples comparison of identical CRUD behavior across stacks.

Every server is scaffolded from one OpenAPI spec with [OpenAPI Generator](https://openapi-generator.tech) (v7.23.0), then the generated stubs are filled in with real CRUD logic and PostgreSQL persistence.

## How It Fits Together

```
spec/petstore-31.yaml  ──(openapi-generator-cli, see COMMANDS.md)──►  per-language server projects
                                                                              │
                                          database/  (shared PostgreSQL, one DB per server)
```

1. The contract lives in `spec/` (`petstore-31.yaml` is the active spec).
2. `COMMANDS.md` documents the exact generator command for each target.
3. Each project implements the endpoints and persists to its **own database** in the shared Postgres container.

## Projects

| Project | Language / Framework | Directory | Database | Default base path |
|---|---|---|---|---|
| **Database** | PostgreSQL 17 (Docker Compose) | `database/` | — (hosts all DBs) | — |
| **Go — Gin** | Go + Gin + `database/sql`/pgx | `go/go-gin-server/` | `go-gin-server` | `/api/v3` |
| **Java — Spring Boot** | Java 17 + Spring Boot 3.3 + `JdbcTemplate` | `java/springboot/` | `java-springboot` | `/api/v3` |
| **Java — Helidon** | Java 21 + Helidon MP 4 (JAX-RS/CDI) + JDBC | `java/helidon/` | `java-helidon` | `/api/v3` |
| **Node.js — Express** | Node.js + Express + `pg` | `nodejs/` | `nodejs-express` | `/` (e.g. `/pet`) |
| **Python — FastAPI** | Python + FastAPI + `asyncpg` | `python/` | `python-fastapi` | `/` (e.g. `/pet`) |
| **Rust** | Rust + hyper + `sqlx` | `rust/` | `rust-server` | `/api/v3` |

Every server listens on **port 8080**. Each project has its own `README.md` (setup/run) and `AGENTS.md` (implementation conventions).

> A `java-quarkus` database name is reserved in the database scripts, but the Quarkus server has not been generated yet.

## Supporting Folders

- `spec/` — OpenAPI specifications (`petstore-31.yaml` is active; `petstore-32.yaml` and `petstore-api-modern.yaml` are alternates).
- `COMMANDS.md` — the OpenAPI Generator command for every target.
- `database/` — the shared PostgreSQL stack, the generated schema, and the create/apply scripts. See `database/DEVELOPER.md`.
- `performance-tests/` — placeholder for cross-implementation load/benchmark tests.
- `openapitools.json` — pins the `openapi-generator-cli` version.

## Quick Start

### 1. Start the shared database

```bash
cd database
cp .env.example .env        # set POSTGRES_USER / POSTGRES_PASSWORD (defaults: myuser / mypassword)
docker compose up -d        # PostgreSQL 17 on host port 5434
./create-databases.sh       # one database per server (idempotent)
./apply-schemas.sh          # applies petstore + OAuth2 schema to each
```

The shared instance is reachable at `localhost:5434` with user `myuser` / password `mypassword` by default.

### 2. Run any server

Each server connects to its own database in that shared instance. Pick a project and follow its README:

```bash
# Go
cd go/go-gin-server && go run main.go

# Java Spring Boot
cd java/springboot && mvn spring-boot:run

# Java Helidon
cd java/helidon && mvn package && java -jar target/petstore-helidon.jar

# Node.js
cd nodejs && npm install && npm start

# Python
cd python && pip install -r requirements.txt && PYTHONPATH=src uvicorn petstore.main:app --port 8080

# Rust
cd rust && cargo run --example petstore-server-server
```

Run one server at a time (or change ports), since they all default to `:8080`.

## Common Behavior Across Implementations

All servers implement the same 19 Petstore operations and share these persistence conventions:

- `category`, `photo_urls`, and `tags` are JSON columns; `category` is stored as a JSON string.
- `pet.status` / `order.status` are PostgreSQL enum types (`pet_status`, `order_status`).
- Server-assigned IDs use `MAX(id) + 1` when omitted; writes upsert with `INSERT … ON CONFLICT`.
- Tables `pet`, `"order"`, and `"user"` are used (the latter two are quoted reserved words).
- `uploadFile` persists the uploaded image bytes to the `pet_photo` table (`BYTEA` content) keyed by `pet_id`; `logoutUser` is a no-op.

See each project's `AGENTS.md` for language-specific details.

## Mutation Testing

All projects have a mutation testing tool configured against the DB-free unit
tests. No database is required.

| Project | Tool | Command |
|---|---|---|
| Java Spring Boot | [PIT](https://pitest.org) | `mvn test-compile org.pitest:pitest-maven:mutationCoverage` |
| Java Helidon | [PIT](https://pitest.org) | `mvn test-compile org.pitest:pitest-maven:mutationCoverage` |
| Node.js | [Stryker](https://stryker-mutator.io) | `npm run mutate` |
| Python | [mutmut](https://mutmut.readthedocs.io) 2.x | `PYTHONPATH=src mutmut run` |
| Go | [gremlins](https://gremlins.dev) | `gremlins unleash ./go/...` |
| Rust | [cargo-mutants](https://mutants.rs) | `cargo mutants` |

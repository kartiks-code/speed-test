# Go Gin Server — Agent Guide

OpenAPI-generated Petstore server implemented with Gin and PostgreSQL.

## Working Directory

Run all commands from `petshop-stacks/go/gin/` unless stated otherwise. The server listens on `:8080` (`PORT` overrides).

## Database

This server uses the shared PostgreSQL stack in `../../../database/` and connects to the `go-gin-server` database.

```bash
cd ../../../database
docker compose up -d
./create-databases.sh          # creates go-gin-server (idempotent)
./apply-schemas.sh             # applies postgresql_schema.sql + postgresql_schema_oauth2.sql
cd ../petshop-stacks/go/gin
```

### Connection environment variables

`store.go` builds a DSN from env. A full DSN takes precedence; otherwise the parts are assembled.

| Variable | Default | Notes |
|---|---|---|
| `DATABASE_URL` / `POSTGRES_DSN` | — | Full DSN; overrides everything below |
| `POSTGRES_USER` | `postgres` | |
| `POSTGRES_PASSWORD` | — | **Required** unless a full DSN is set |
| `POSTGRES_HOST` | `localhost` | |
| `POSTGRES_PORT` | `5432` | the shared stack uses `5434` |
| `POSTGRES_DB` | `go-gin-server` | |
| `POSTGRES_SSLMODE` | `disable` | |

### Tuning environment variables (optional)

These are unset by default; unset preserves current behavior. The benchmark harness's `Dockerfile.optimized` sets them via `ENV` (the harness `env` block in `performance-tests/stacks.json` applies to both variants, so tuning must stay env-driven).

| Variable | Default | Effect |
|---|---|---|
| `GIN_DISABLE_REQUEST_LOGGING` | unset | `true` → engine built with `gin.New()` + `gin.Recovery()` (no per-request logger) instead of `gin.Default()` (`go/routers.go`) |
| `DB_MAX_OPEN_CONNS` | unset (unlimited) | `sql.DB SetMaxOpenConns` (`go/store.go`) |
| `DB_MAX_IDLE_CONNS` | unset (Go default 2) | `sql.DB SetMaxIdleConns` (`go/store.go`) |
| `DB_CONN_MAX_IDLE_TIME_SECONDS` | unset (no limit) | `sql.DB SetConnMaxIdleTime`, in seconds (`go/store.go`) |

`Dockerfile.optimized` sets `GIN_DISABLE_REQUEST_LOGGING=true GOMEMLIMIT=460MiB DB_MAX_OPEN_CONNS=200 DB_MAX_IDLE_CONNS=50`: a pool of 200 handles up to 500 k6 VUs via queueing while staying under Postgres `max_connections=500`, and `GOMEMLIMIT` aligns the GC with the harness's 512m memory limit.

To point at the shared Compose stack, export a full DSN or override the parts:

```bash
export DATABASE_URL='postgres://myuser:mypassword@localhost:5434/go-gin-server?sslmode=disable'
go run main.go
```

## Code Layout

```
petshop-stacks/go/gin/
├── main.go                 # entry point; builds PostgresStore from env, wires handlers, starts router
└── go/                     # package "petstore"
    ├── api_pet.go          # Gin handlers for /pet — edit here
    ├── api_store.go        # Gin handlers for /store — edit here
    ├── api_user.go         # Gin handlers for /user — edit here
    ├── api_helpers.go      # request binding, query parsing, error mapping helpers
    ├── store.go            # Store interface + PostgresStore (all SQL, scanners, DSN) — edit here
    ├── routers.go          # generated router wiring
    ├── model_*.go          # generated DTOs (Pet, Order, User, …)
    ├── api_handlers_test.go
    └── store_test.go
```

## Conventions

- Handlers are thin: they bind input, call a `Store` method, and map errors. All persistence lives behind the `Store` interface in `store.go`.
- `PostgresStore` uses `database/sql` with the `pgx` stdlib driver (`github.com/jackc/pgx/v5/stdlib`). No ORM.
- `ErrNotFound` maps to 404 and `ErrInvalidInput` maps to 400 via `handleStoreError` / `storeError`.
- `category`, `photo_urls`, and `tags` are JSON columns written with `encoding/json`. `category` is stored as a JSON string.
- Enum columns (`pet.status` = `pet_status`, `order.status` = `order_status`) are cast with `NULLIF($n, '')::pet_status` on write and selected as `status::text` on read.
- IDs are generated server-side with `SELECT nextval('<table>_id_seq')` (sequences: `pet_id_seq`, `order_id_seq`, `user_id_seq`, `pet_photo_id_seq`) when omitted; writes use `INSERT … ON CONFLICT … DO UPDATE` upserts.
- `CreateUsers` runs inside a transaction. Tables used: `pet`, `"order"`, `"user"` (quoted because they are reserved words).
- `uploadFile` verifies the pet exists, then persists the raw request body via `Store.SavePetPhoto` into the `pet_photo` table; the response reports the stored byte count. `logoutUser` is a stateless no-op.

## Verification

```bash
go build ./...
go vet ./...
go test ./...
```

PostgreSQL integration tests are skipped unless `TEST_DATABASE_DSN` is set:

```bash
export TEST_DATABASE_DSN='postgres://myuser:mypassword@localhost:5434/go-gin-server?sslmode=disable'
go test ./go -run TestPostgresStore
```

Smoke-test a running server:

```bash
go run main.go &
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'
curl -s http://localhost:8080/api/v3/store/inventory
```

## Mutation Testing

[gremlins](https://gremlins.dev) is configured in `.gremlins.yaml`.
It mutates the hand-written `petstore` package (`./go/`) and reruns `go test`
against each mutant. The DB-free unit tests (mock `Store` interface) run fast
enough for per-mutant reruns without a live database.

```bash
# Install (one time):
go install github.com/go-gremlins/gremlins/cmd/gremlins@latest

# Run from the project directory (petshop-stacks/go/gin/):
gremlins unleash ./go/...
```

gremlins prints a mutation score and lists surviving mutants per file. A
surviving mutant means no test distinguished the mutated code from the original —
either add a sharper assertion or confirm the mutation is equivalent.

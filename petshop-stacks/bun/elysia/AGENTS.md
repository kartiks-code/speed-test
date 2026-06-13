# Bun + Elysia Petstore — Agent Guide

## Stack

- **Runtime:** [Bun](https://bun.sh) (>= 1.1)
- **Framework:** [Elysia](https://elysiajs.com) 1.x
- **DB client:** [`postgres`](https://github.com/porsager/postgres) 3.x (porsager — better Bun compatibility than `pg`)
- **Language:** TypeScript throughout (all `.ts` files)
- **All code is hand-written** — no OpenAPI Generator target for this stack.

## File Layout

| Path | Role |
|---|---|
| `index.ts` | Entry point: starts the Elysia server |
| `app.ts` | Elysia app factory (`buildApp()`), applies `/api/v3` prefix and mounts route groups |
| `models.ts` | Shared TypeScript interfaces: `Pet`, `Order`, `User`, `ApiResponse` |
| `db/client.ts` | `postgres.js` connection pool (reads env vars, exports tagged-template `sql` default) |
| `db/petRepository.ts` | Pet CRUD + photo upload |
| `db/orderRepository.ts` | Order CRUD + inventory |
| `db/userRepository.ts` | User CRUD + login |
| `routes/pet.ts` | Elysia route group for `/pet` endpoints |
| `routes/store.ts` | Elysia route group for `/store` endpoints |
| `routes/user.ts` | Elysia route group for `/user` endpoints |
| `test/` | Bun test files (DB-free, module-mocked) |
| `stryker.config.json` | Stryker mutation config (command runner with `bun test`) |
| `Dockerfile` | Naive image |
| `Dockerfile.optimized` | Multi-stage Alpine image with production env |

## Commands

```bash
bun install          # install dependencies (generates bun.lockb)
bun test             # run unit tests (no DB required)
bun run index.ts     # start server on port 8080
npm run mutate       # Stryker mutation testing (best-effort; see below)
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `POSTGRES_HOST` | `localhost` | DB host |
| `POSTGRES_PORT` | `5434` | DB port (shared Postgres container is on 5434 locally, 5432 in Docker network) |
| `POSTGRES_USER` | `myuser` | DB user |
| `POSTGRES_PASSWORD` | `mypassword` | DB password |
| `POSTGRES_DB` | `bun-elysia` | Database name |
| `DATABASE_URL` | _(unset)_ | Full postgres:// DSN; overrides all individual vars above |
| `PG_POOL_MAX` | `10` | Max DB connections (100 in optimized container) |
| `PORT` | `8080` | HTTP listen port |

## Database

This stack uses the **`bun-elysia`** database in the shared PostgreSQL container.

```bash
cd database
docker compose up -d
./create-databases.sh  # creates bun-elysia if not present
./apply-schemas.sh     # applies schema and sequences
```

## API Base Path

All routes are prefixed with `/api/v3` (set via `new Elysia({ prefix: '/api/v3' })`).

Readiness probe: `GET /api/v3/pet/findByStatus?status=available`

## Elysia Notes

- Routes are chained on the `Elysia` instance: `.get('/path', handler).post('/path', handler)`.
- Route groups are composed via `.use(routeGroup)` on the top-level app in `app.ts`.
- The `prefix` option on the top-level app applies to all mounted routes.
- For `POST /pet/:petId/uploadImage`, the handler reads `request.arrayBuffer()` as a fallback to handle various content types. The raw bytes are stored in `pet_photo.content` (BYTEA).

## Testing

Tests use Bun's built-in test runner (`bun:test`). All tests are DB-free via `mock.module`:

```bash
bun test
```

Each repository test file:
1. Registers a `mock.module('../../db/client', ...)` before the module under test loads.
2. Dynamically imports the repository in `beforeAll` (so the mock is in place).
3. Enqueues return values via a shared `sqlResults` array.

## Mutation Testing

Stryker is configured with the `command` test runner (runs `bun test`). This is **best-effort** — Stryker's command runner does not integrate as tightly with Bun as with Node.js:

- Stryker mutates source files one at a time and re-runs `bun test` for each mutation.
- There is no Bun-native Stryker plugin; the HTML report is written to `reports/mutation/mutation.html`.
- If `stryker run` hangs or produces incorrect results, run `bun test` independently to verify baseline test health first.

```bash
npm run mutate   # or: npx stryker run
```

## Performance Harness

| Key | Value |
|---|---|
| `stacks.json` id | `elysia` |
| Naive host port | 8111 |
| Optimized host port | 8112 |
| `base_path` | `/api/v3` |
| `readiness_path` | `/api/v3/pet/findByStatus?status=available` |
| In-container DB host | `speed-test-postgres:5432` |

# Node.js Fastify Server — Agent Guide

## Working Directory

Run all commands from `petshop-stacks/nodejs/fastify/` unless stated otherwise.

## Stack

Node.js + [Fastify 4](https://fastify.dev/). All code is hand-written — no OpenAPI Generator target for Fastify.

## Quick Start

```bash
npm install
npm start          # listens on :8080
```

API base path: `/api/v3`

## Database

This server uses the shared PostgreSQL instance managed from `../../../database/`. It connects to the `nodejs-fastify` database.

### Set up the database (one time)

```bash
cd ../../../database
docker compose up -d
./create-databases.sh          # creates nodejs-fastify if it doesn't exist
./apply-schemas.sh             # applies postgresql_schema.sql
```

### Connection environment variables

| Variable | Default | Notes |
|---|---|---|
| `POSTGRES_HOST` | `localhost` | Use `speed-test-postgres` inside Docker harness |
| `POSTGRES_PORT` | `5434` | matches `database/.env` default; use `5432` inside Docker |
| `POSTGRES_DB` | `nodejs-fastify` | |
| `POSTGRES_USER` | `myuser` | |
| `POSTGRES_PASSWORD` | `mypassword` | |
| `PG_POOL_MAX` | unset (pg default 10) | Max pool connections per process |
| `PG_POOL_IDLE_TIMEOUT_MS` | unset (pg default 10000ms) | `idleTimeoutMillis` for the pool |
| `PORT` | `8080` | HTTP listen port |

## Code Structure

```
petshop-stacks/nodejs/fastify/
├── index.js              # entry point: starts the Fastify server
├── cluster.js            # optional multi-process entry (WEB_CONCURRENCY)
├── server.js             # app factory: buildApp() creates and configures Fastify
├── db/
│   ├── pool.js           # pg.Pool singleton + query() helper
│   ├── cache.js          # in-process TTL cache (inventory + findByStatus)
│   ├── petRepository.js  # pet table CRUD + inventory; COALESCE id + RETURNING
│   ├── orderRepository.js # order table CRUD; COALESCE id + RETURNING
│   └── userRepository.js  # user table CRUD + authentication
├── routes/
│   ├── pet.js            # Fastify plugin: /api/v3/pet/* routes
│   ├── store.js          # Fastify plugin: /api/v3/store/* routes
│   └── user.js           # Fastify plugin: /api/v3/user/* routes (createWithList uses Promise.all)
├── test/
│   ├── helpers.js        # expectRejection utility
│   └── db/
│       ├── petRepository.test.js
│       ├── orderRepository.test.js
│       └── userRepository.test.js
├── stryker.config.json
├── Dockerfile            # naive: node:20
├── Dockerfile.optimized  # multi-stage: node:22-alpine, WEB_CONCURRENCY=2, cluster.js
└── .dockerignore
```

## Tuning environment variables

| Variable | Default | Effect |
|---|---|---|
| `WEB_CONCURRENCY` | `1` | `cluster.js` forks this many workers and respawns dead ones; `1` runs in-process |
| `PG_POOL_MAX` | unset (pg default 10) | Max pool connections per process |
| `PG_POOL_IDLE_TIMEOUT_MS` | unset (pg default 10000ms) | `idleTimeoutMillis` for the pool |
| `PORT` | `8080` | HTTP listen port |

The `Dockerfile.optimized` sets `WEB_CONCURRENCY=2 PG_POOL_MAX=100 NODE_ENV=production` and starts via `node cluster.js`.

## Conventions

- All route registrations follow Fastify plugin pattern (async functions accepting `fastify` instance).
- Static routes (`/pet/findByStatus`, `/pet/findByTags`, `/user/login`, `/user/logout`) are registered **before** parameterized routes (`/pet/:petId`, `/user/:username`) within each plugin.
- `db/pool.js` exports both `pool` (the `pg.Pool` instance) and `query` (a wrapper) — tests stub `pool.query` via Sinon.
- `pet.id` and `order.id` are assigned using `COALESCE($1::bigint, nextval('..._id_seq'))` in the INSERT with `RETURNING "id"` — a single query regardless of whether the caller supplies an id.
- `addPhoto` uses a single `INSERT ... SELECT ... FROM pet WHERE "id" = $1` query — the insert only proceeds if the pet exists, eliminating the previous two-query check-then-insert pattern.
- `petRepository` maintains an in-process TTL cache (`db/cache.js`) for `getInventory` (5 s TTL) and `findByStatus` per-status (3 s TTL). All writes call `invalidatePetCache()`. Call `cache.clearAll()` in test `beforeEach` hooks.
- `pet.status` and `order.status` are PostgreSQL enum types; written with explicit casts (`cast($N as pet_status)`, `cast($N as order_status)`), read back as `::text`.
- `category`, `photo_urls`, `tags` are JSON columns; serialized to strings before write, `mapRow` handles both pre-parsed objects and raw JSON strings on read.
- `uploadFile` stores raw request bytes (`application/octet-stream`, parsed as Buffer) into `pet_photo.content` (BYTEA).
- `logoutUser` is a stateless no-op returning 200.
- `POST /user/createWithList` creates all users concurrently via `Promise.all`.

## Common Commands

```bash
npm install           # install dependencies
npm start             # start server on port 8080
npm test              # run Mocha test suite (DB-free, uses Sinon stubs)
npm run mutate        # run Stryker mutation analysis
```

## Mutation Testing

[Stryker](https://stryker-mutator.io) is configured in `stryker.config.json`. It mutates `db/` (repositories, excluding `pool.js`) and `routes/`, running the Mocha suite against each mutant. No live database is needed.

```bash
npm run mutate        # report at reports/mutation/mutation.html
```

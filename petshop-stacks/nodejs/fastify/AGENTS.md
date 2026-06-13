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
├── server.js             # app factory: buildApp() creates and configures Fastify
├── db/
│   ├── pool.js           # pg.Pool singleton + query() helper
│   ├── petRepository.js  # pet table CRUD + inventory; uses nextval() for IDs
│   ├── orderRepository.js # order table CRUD; uses nextval() for IDs
│   └── userRepository.js  # user table CRUD + authentication
├── routes/
│   ├── pet.js            # Fastify plugin: /api/v3/pet/* routes
│   ├── store.js          # Fastify plugin: /api/v3/store/* routes
│   └── user.js           # Fastify plugin: /api/v3/user/* routes
├── test/
│   ├── helpers.js        # expectRejection utility
│   └── db/
│       ├── petRepository.test.js
│       ├── orderRepository.test.js
│       └── userRepository.test.js
├── stryker.config.json
├── Dockerfile            # naive: node:20
├── Dockerfile.optimized  # multi-stage: node:22-alpine, non-root user
└── .dockerignore
```

## Conventions

- All route registrations follow Fastify plugin pattern (async functions accepting `fastify` instance).
- Static routes (`/pet/findByStatus`, `/pet/findByTags`, `/user/login`, `/user/logout`) are registered **before** parameterized routes (`/pet/:petId`, `/user/:username`) within each plugin.
- `db/pool.js` exports both `pool` (the `pg.Pool` instance) and `query` (a wrapper) — tests stub `pool.query` via Sinon.
- Repositories use PostgreSQL sequences for server-assigned IDs: `nextval('pet_id_seq')`, `nextval('order_id_seq')`, `nextval('pet_photo_id_seq')`.
- `pet.status` and `order.status` are PostgreSQL enum types; written with explicit casts (`cast($N as pet_status)`, `cast($N as order_status)`), read back as `::text`.
- `category`, `photo_urls`, `tags` are JSON columns; serialized to strings before write, `mapRow` handles both pre-parsed objects and raw JSON strings on read.
- `uploadFile` stores raw request bytes (`application/octet-stream`, parsed as Buffer) into `pet_photo.content` (BYTEA).
- `logoutUser` is a stateless no-op returning 200.

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

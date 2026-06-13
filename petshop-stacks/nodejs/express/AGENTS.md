# Node.js Express Server — Agent Guide

## Working Directory

Run all commands from `petshop-stacks/nodejs/express/` unless stated otherwise.

## Quick Start

```bash
npm install
npm start          # listens on :8080
```

API docs are served at `http://localhost:8080/api-docs/`.

## Database

This server uses the shared PostgreSQL instance managed from `../../../database/`. It connects to the `nodejs-express` database.

### Set up the database (one time)

```bash
cd ../../../database
docker compose up -d
./create-databases.sh          # creates nodejs-express if it doesn't exist
./apply-schemas.sh             # applies postgresql_schema.sql + postgresql_schema_oauth2.sql
```

### Connection environment variables

| Variable | Default | Notes |
|---|---|---|
| `POSTGRES_HOST` | `localhost` | |
| `POSTGRES_PORT` | `5434` | matches `database/.env` default |
| `POSTGRES_DB` | `nodejs-express` | |
| `POSTGRES_USER` | `myuser` | |
| `POSTGRES_PASSWORD` | `mypassword` | |

Set these in the environment before `npm start`, or export from `database/.env`:

```bash
set -a && source ../../../database/.env && set +a
npm start
```

### Tuning environment variables

All optional; defaults preserve naive behavior. `Dockerfile.optimized` sets the tuned values (`WEB_CONCURRENCY=2 PG_POOL_MAX=100 EXPRESS_LEAN=true`, `NODE_ENV=production`) and starts via `node cluster.js`. The naive `Dockerfile` sets none of them and starts `node index.js`.

| Variable | Default | Effect |
|---|---|---|
| `WEB_CONCURRENCY` | `1` | `cluster.js` forks this many workers (each running `index.js`) and respawns dead ones; `1` runs the app in-process |
| `PG_POOL_MAX` | unset (pg default `10`) | `max` for the `pg.Pool` in `db/pool.js`, per process |
| `PG_POOL_IDLE_TIMEOUT_MS` | unset (pg default `10000`) | `idleTimeoutMillis` for the pool |
| `EXPRESS_LEAN` | unset | `true` sets `app.set('etag', false)` and disables `x-powered-by` in `expressServer.js` |
| `NODE_ENV` | unset | `production` makes `logger.js` use a single error-level Console transport instead of console + file transports |

## Code Structure

```
petshop-stacks/nodejs/express/
├── api/openapi.yaml          # API contract (do not edit manually)
├── controllers/              # thin request/response adapters (generated)
├── services/                 # business logic — edit here
│   ├── PetService.js
│   ├── StoreService.js
│   └── UserService.js
├── db/                       # persistence layer
│   ├── pool.js               # pg.Pool singleton
│   ├── petRepository.js      # pet table CRUD + inventory
│   ├── orderRepository.js    # order table CRUD
│   └── userRepository.js     # user table CRUD + auth
├── expressServer.js          # Express + OpenAPI validator setup
├── config.js                 # port, paths
├── index.js                  # entry point
└── cluster.js                # optional multi-process entry (WEB_CONCURRENCY)
```

## Conventions

- All business logic lives in `services/` and `db/`. Controllers are generated and should not need changes.
- `services/index.js` and `controllers/index.js` export the service/controller classes — do not let codegen overwrite them once the DB wiring is in place.
- Repositories throw plain `Error` objects with a `.status` property (404, 400, etc.) which the services map to `Service.rejectResponse(msg, status)`.
- The `pg` node-postgres driver is used directly (no ORM). SQL mirrors the Helidon sibling server in `../../java/helidon/`.
- `photo_urls` and `tags` are stored as JSON columns. `category` on `pet` is also a JSON string. Enums (`pet_status`, `order_status`) are cast explicitly in SQL.
- `pet.id` and `order.id` are generated server-side from a monotonic counter seeded at startup if absent in the request.
- `uploadFile` persists the raw request body (parsed via the `application/octet-stream` body parser) to the `pet_photo` table through `petRepository.addPhoto`; `logoutUser` requires no DB interaction.

## Mutation Testing

[Stryker](https://stryker-mutator.io) is configured in `stryker.config.json`.
It mutates the hand-written `db/` (repositories) and `services/` code, running
the full Mocha test suite against each mutant. The tests mock `pg.Pool`, so no
live database is needed.

```bash
npm install                   # installs @stryker-mutator/core + mocha-runner
npm run mutate                # runs stryker; writes reports/ to reports/mutation/

# Alias without the npm script:
npx stryker run
```

Stryker produces an HTML report at `reports/mutation/mutation.html`.
A surviving mutant means no test distinguished the mutated code from the
original — either tighten an assertion or confirm the mutation is equivalent.

**Config note:** `stryker.config.json` uses `mochaOptions.spec` (an array) for
`@stryker-mutator/mocha-runner` v9. The older `mocha.spec` key (a string) is not
recognised by Stryker 9.x and will produce a "No tests were executed" error.

**Current mutation score:** ~92% overall (100% on all `db/` repositories;
~83% on `services/` — the remaining no-coverage mutants are unreachable
`catch` blocks in `logoutUser` and StringLiteral fallback-message mutations
in service error paths that require an error with an empty message to kill).

## Common Commands

```bash
npm install                   # install dependencies including pg
npm start                     # start server on port 8080
npm test                      # run Mocha test suite
npm run mutate                # run Stryker mutation analysis
```

# Node.js Express — Petstore Server

OpenAPI-generated Petstore server built on **Express** with the `express-openapi-validator`, persisting to PostgreSQL via the `pg` (node-postgres) driver.

## Prerequisites

- Node.js >= 18
- npm
- Docker (with Compose v2) for the database

## Database

This server persists to the shared PostgreSQL instance in `../../../database/` and uses the `nodejs-express` database.

```bash
cd ../../../database
docker compose up -d
./create-databases.sh        # creates nodejs-express (idempotent)
./apply-schemas.sh           # applies petstore + OAuth2 tables
cd ../../../petshop-stacks/nodejs/express
```

### Connection configuration

Defaults align with `database/.env`. Override via environment variables:

| Variable | Default |
|---|---|
| `POSTGRES_HOST` | `localhost` |
| `POSTGRES_PORT` | `5434` |
| `POSTGRES_DB` | `nodejs-express` |
| `POSTGRES_USER` | `myuser` |
| `POSTGRES_PASSWORD` | `mypassword` |

To export the shared values before starting:

```bash
set -a && source ../database/.env && set +a
export POSTGRES_DB=nodejs-express
```

### Tuning configuration

All optional; defaults preserve the original (naive) behavior. The optimized Docker image (`Dockerfile.optimized`) sets the tuned values.

| Variable | Default | Notes |
|---|---|---|
| `WEB_CONCURRENCY` | `1` | Number of `node:cluster` workers started by `cluster.js`; `1` runs the app directly |
| `PG_POOL_MAX` | unset (pg default `10`) | `max` connections per process in the `pg.Pool` |
| `PG_POOL_IDLE_TIMEOUT_MS` | unset (pg default `10000`) | Pool `idleTimeoutMillis` |
| `EXPRESS_LEAN` | unset | `true` disables ETag generation and the `X-Powered-By` header |
| `NODE_ENV` | unset | `production` drops winston file transports (`error.log`/`combined.log`) and logs only errors to the console |

## Install and Run

```bash
npm install
npm start          # listens on :8080
```

Interactive API docs are served at `http://localhost:8080/api-docs/`.

## Try It

```bash
# Add a pet
curl -s -X POST http://localhost:8080/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'

# Fetch it back
curl -s http://localhost:8080/pet/1

# Inventory by status
curl -s http://localhost:8080/store/inventory
```

## Project Layout

```
nodejs/
├── api/openapi.yaml      # API contract (generated; do not edit manually)
├── controllers/          # thin request/response adapters (generated)
├── services/             # business logic — edit here (PetService, StoreService, UserService)
├── db/                   # persistence layer — edit here
│   ├── pool.js           # pg.Pool singleton
│   ├── petRepository.js  # pet CRUD + inventory
│   ├── orderRepository.js
│   └── userRepository.js
├── utils/openapiRouter.js
├── expressServer.js      # Express + OpenAPI validator setup
├── config.js             # port, paths
├── index.js              # entry point
└── cluster.js            # optional multi-process entry (WEB_CONCURRENCY workers)
```

All business logic lives in `services/` and `db/`. Controllers are generated and should not need changes. See `AGENTS.md` for implementation conventions.

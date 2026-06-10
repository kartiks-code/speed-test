# Node.js Express Server — Agent Guide

## Working Directory

Run all commands from `nodejs/` unless stated otherwise.

## Quick Start

```bash
npm install
npm start          # listens on :8080
```

API docs are served at `http://localhost:8080/api-docs/`.

## Database

This server uses the shared PostgreSQL instance managed from `../database/`. It connects to the `nodejs-express` database.

### Set up the database (one time)

```bash
cd ../database
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
set -a && source ../database/.env && set +a
npm start
```

## Code Structure

```
nodejs/
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
└── index.js                  # entry point
```

## Conventions

- All business logic lives in `services/` and `db/`. Controllers are generated and should not need changes.
- `services/index.js` and `controllers/index.js` export the service/controller classes — do not let codegen overwrite them once the DB wiring is in place.
- Repositories throw plain `Error` objects with a `.status` property (404, 400, etc.) which the services map to `Service.rejectResponse(msg, status)`.
- The `pg` node-postgres driver is used directly (no ORM). SQL mirrors the Helidon sibling server in `../java/helidon/`.
- `photo_urls` and `tags` are stored as JSON columns. `category` on `pet` is also a JSON string. Enums (`pet_status`, `order_status`) are cast explicitly in SQL.
- `pet.id` and `order.id` are generated server-side from a monotonic counter seeded at startup if absent in the request.
- `uploadFile` and `logoutUser` require no DB interaction.

## Common Commands

```bash
npm install                   # install dependencies including pg
npm start                     # start server on port 8080
```

# Petstore — Bun + Elysia

A hand-written implementation of the [Petstore OpenAPI spec](../../spec/petstore-31.yaml) using [Bun](https://bun.sh) as the runtime and [Elysia](https://elysiajs.com) as the web framework.

## Stack

| Component | Version |
|---|---|
| Bun | >= 1.1 |
| Elysia | 1.x |
| postgres.js | 3.x |
| TypeScript | 5.x |

## Prerequisites

- [Bun](https://bun.sh/docs/installation) installed
- Shared PostgreSQL container running (see [database/](../../database/))

## Setup

```bash
# 1. Start the shared database
cd ../../database
docker compose up -d
./create-databases.sh   # creates the bun-elysia database
./apply-schemas.sh      # applies tables, enums, and sequences

# 2. Install dependencies
cd ../../petshop-stacks/bun/elysia
bun install
```

## Run

```bash
bun run index.ts
# or:
bun start
```

The server listens on `http://localhost:8080` with all routes under `/api/v3`.

## Test

```bash
bun test
```

Tests are DB-free and use Bun's built-in module mocking (`mock.module`).

## Mutation Testing

```bash
npm run mutate
```

Uses Stryker with the `command` runner (invokes `bun test` for each mutant). Results are written to `reports/mutation/mutation.html`. See `AGENTS.md` for known limitations.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `POSTGRES_HOST` | `localhost` | DB host |
| `POSTGRES_PORT` | `5434` | DB port |
| `POSTGRES_USER` | `myuser` | DB user |
| `POSTGRES_PASSWORD` | `mypassword` | DB password |
| `POSTGRES_DB` | `bun-elysia` | Database name |
| `DATABASE_URL` | _(unset)_ | Full DSN; overrides the above |
| `PG_POOL_MAX` | `10` | Connection pool size |
| `PORT` | `8080` | HTTP port |

## Docker

**Naive build:**
```bash
docker build -t petstore-elysia .
docker run -p 8080:8080 \
  -e POSTGRES_HOST=host.docker.internal \
  -e POSTGRES_PORT=5434 \
  petstore-elysia
```

**Optimized build:**
```bash
docker build -f Dockerfile.optimized -t petstore-elysia-opt .
docker run -p 8080:8080 \
  -e POSTGRES_HOST=host.docker.internal \
  -e POSTGRES_PORT=5434 \
  petstore-elysia-opt
```

## API

All endpoints follow the Petstore OpenAPI 3.1 spec with base path `/api/v3`.

Key endpoints:

| Method | Path | Description |
|---|---|---|
| POST | `/api/v3/pet` | Add a new pet |
| PUT | `/api/v3/pet` | Update an existing pet |
| GET | `/api/v3/pet/findByStatus` | Find pets by status |
| GET | `/api/v3/pet/findByTags` | Find pets by tags |
| GET | `/api/v3/pet/{petId}` | Get pet by ID |
| POST | `/api/v3/pet/{petId}` | Update pet with form data |
| DELETE | `/api/v3/pet/{petId}` | Delete a pet |
| POST | `/api/v3/pet/{petId}/uploadImage` | Upload a pet photo |
| GET | `/api/v3/store/inventory` | Get pet inventory by status |
| POST | `/api/v3/store/order` | Place an order |
| GET | `/api/v3/store/order/{orderId}` | Get order by ID |
| DELETE | `/api/v3/store/order/{orderId}` | Delete an order |
| POST | `/api/v3/user` | Create a user |
| POST | `/api/v3/user/createWithList` | Create multiple users |
| GET | `/api/v3/user/login` | Log in (returns token) |
| GET | `/api/v3/user/logout` | Log out (stateless no-op) |
| GET | `/api/v3/user/{username}` | Get user by username |
| PUT | `/api/v3/user/{username}` | Update a user |
| DELETE | `/api/v3/user/{username}` | Delete a user |

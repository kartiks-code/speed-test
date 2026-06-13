# Petstore — Node.js + Fastify

An implementation of the [OpenAPI Petstore API](../../spec/petstore-31.yaml) using [Node.js](https://nodejs.org/) and [Fastify 4](https://fastify.dev/), backed by a shared PostgreSQL 17 database.

## Requirements

- Node.js 20+
- PostgreSQL 17 (shared instance via `../../../database/`)

## Build & Run

```bash
npm install
npm start        # starts on http://localhost:8080
```

## Test

```bash
npm test         # Mocha suite — DB-free (Sinon stubs)
```

## Mutation Testing

```bash
npm run mutate   # Stryker — report at reports/mutation/mutation.html
```

## Database Setup

The server connects to the `nodejs-fastify` database in the shared PostgreSQL container.

```bash
cd ../../../database
docker compose up -d
./create-databases.sh    # creates nodejs-fastify DB
./apply-schemas.sh       # applies schema
```

### Environment Variables

| Variable | Default |
|---|---|
| `POSTGRES_HOST` | `localhost` |
| `POSTGRES_PORT` | `5434` |
| `POSTGRES_DB` | `nodejs-fastify` |
| `POSTGRES_USER` | `myuser` |
| `POSTGRES_PASSWORD` | `mypassword` |
| `PORT` | `8080` |
| `PG_POOL_MAX` | *(pg default 10)* |

## API Base Path

All endpoints are served under `/api/v3`.

| Route | Description |
|---|---|
| `POST /api/v3/pet` | Add a new pet |
| `PUT /api/v3/pet` | Update an existing pet |
| `GET /api/v3/pet/findByStatus` | Find pets by status |
| `GET /api/v3/pet/findByTags` | Find pets by tags |
| `GET /api/v3/pet/:petId` | Get pet by ID |
| `POST /api/v3/pet/:petId` | Update pet with form data |
| `DELETE /api/v3/pet/:petId` | Delete a pet |
| `POST /api/v3/pet/:petId/uploadImage` | Upload a pet image |
| `GET /api/v3/store/inventory` | Get pet inventory by status |
| `POST /api/v3/store/order` | Place an order |
| `GET /api/v3/store/order/:orderId` | Get order by ID |
| `DELETE /api/v3/store/order/:orderId` | Delete an order |
| `POST /api/v3/user` | Create a user |
| `POST /api/v3/user/createWithList` | Create users from a list |
| `GET /api/v3/user/login` | Log in a user |
| `GET /api/v3/user/logout` | Log out (no-op) |
| `GET /api/v3/user/:username` | Get user by username |
| `PUT /api/v3/user/:username` | Update a user |
| `DELETE /api/v3/user/:username` | Delete a user |

## Docker

```bash
# Naive build
docker build -t petstore-fastify .

# Optimized build (multi-stage, non-root, PG_POOL_MAX=100)
docker build -f Dockerfile.optimized -t petstore-fastify:optimized .

docker run -p 8080:8080 \
  -e POSTGRES_HOST=host.docker.internal \
  -e POSTGRES_PORT=5434 \
  petstore-fastify
```

# Go Fiber Petstore

A Go implementation of the OpenAPI Petstore API using the [Fiber](https://github.com/gofiber/fiber) v2 web framework. This stack mirrors the Gin implementation at `../gin/` but uses Fiber's fasthttp-based context API.

## What it implements

All 19 Petstore operations across three resource groups:

- **Pet** — `AddPet`, `UpdatePet`, `GetPetById`, `DeletePet`, `FindPetsByStatus`, `FindPetsByTags`, `UpdatePetWithForm`, `UploadFile`
- **Store** — `GetInventory`, `PlaceOrder`, `GetOrderById`, `DeleteOrder`
- **User** — `CreateUser`, `CreateUsersWithListInput`, `LoginUser`, `LogoutUser`, `GetUserByName`, `UpdateUser`, `DeleteUser`

All endpoints are available under the base path `/api/v3`.

## Prerequisites

- Go 1.22+
- PostgreSQL 17 (shared database at `localhost:5434` in the dev environment)

## Database setup

Create and initialize the `go-fiber` database (run once):

```bash
cd ../../database
docker compose up -d
./create-databases.sh
./apply-schemas.sh
```

## Build

```bash
go build ./...
```

## Test

```bash
# Unit tests (no database required)
go test ./...

# Integration tests (requires PostgreSQL)
TEST_DATABASE_DSN="postgres://myuser:mypassword@localhost:5434/go-fiber?sslmode=disable" go test ./...
```

## Run

```bash
# Requires POSTGRES_PASSWORD (or DATABASE_URL / POSTGRES_DSN)
POSTGRES_PASSWORD=mypassword POSTGRES_PORT=5434 POSTGRES_USER=myuser go run main.go
```

The server listens on port 8080 by default. Set `PORT` to change it.

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_HOST` | `localhost` | DB host |
| `POSTGRES_PORT` | `5432` | DB port (shared dev stack uses `5434`) |
| `POSTGRES_USER` | `postgres` | DB user |
| `POSTGRES_PASSWORD` | *(required)* | DB password |
| `POSTGRES_DB` | `go-fiber` | Database name |
| `DATABASE_URL` | — | Full connection string (overrides above) |
| `PORT` | `8080` | HTTP listen port |
| `FIBER_DISABLE_REQUEST_LOGGING` | — | Set `true` to suppress access logs |
| `GOMEMLIMIT` | — | Go memory limit (e.g. `460MiB`) |
| `DB_MAX_OPEN_CONNS` | — | Max open DB connections |
| `DB_MAX_IDLE_CONNS` | — | Max idle DB connections |

## Docker

```bash
# Standard build
docker build -t petstore-fiber .

# Optimized (scratch-based) build
docker build -f Dockerfile.optimized -t petstore-fiber:opt .

docker run -e POSTGRES_PASSWORD=mypassword -e POSTGRES_HOST=host.docker.internal \
  -e POSTGRES_PORT=5434 -e POSTGRES_USER=myuser -p 8080:8080 petstore-fiber
```

## Architecture

```
main.go           — connects to PostgreSQL, wires routes, starts Fiber
go/store.go       — Store interface + PostgresStore (all SQL)
go/routers.go     — NewFiberApp(): route registration
go/api_helpers.go — shared Fiber context utilities
go/api_pet.go     — PetAPI handlers
go/api_store.go   — StoreAPI handlers
go/api_user.go    — UserAPI handlers
go/model_*.go     — struct definitions (JSON tags)
```

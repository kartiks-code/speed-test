# Go Gin Petstore Server

This service implements the generated OpenAPI Petstore server in `api/openapi.yaml` using Gin and PostgreSQL.

## Database Setup

Run the local PostgreSQL container and apply the shared schema from the repository `database/` directory:

```bash
cd ../../database
docker compose up -d
./create-databases.sh
./apply-schemas.sh
```

The service uses the `go-gin-server` database. The shared schema defines the Petstore tables and adds unique indexes for the API identities used by CRUD operations:

- `pet.id`
- `"order".id`
- `"user".username`

## Configuration

The server accepts either a full PostgreSQL URL or individual connection settings.

```bash
export POSTGRES_USER=postgres
export POSTGRES_PASSWORD=mysecret
export POSTGRES_HOST=localhost
export POSTGRES_PORT=5432
export POSTGRES_DB=go-gin-server
```

Alternatively:

```bash
export DATABASE_URL='postgres://postgres:mysecret@localhost:5432/go-gin-server?sslmode=disable'
```

`PORT` controls the HTTP port and defaults to `8080`.

## Run

```bash
go run main.go
```

The generated routes are mounted under `/api/v3`, for example:

```bash
curl -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"name":"doggie","photoUrls":["https://example.test/dog.png"],"status":"available"}'

curl http://localhost:8080/api/v3/pet/1
curl http://localhost:8080/api/v3/store/inventory
```

## Tests

Run package tests:

```bash
go test ./...
```

PostgreSQL integration coverage is skipped unless `TEST_DATABASE_DSN` is set:

```bash
export TEST_DATABASE_DSN='postgres://postgres:mysecret@localhost:5432/go-gin-server?sslmode=disable'
go test ./go -run TestPostgresStore
```


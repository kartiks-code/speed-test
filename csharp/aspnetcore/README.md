# C# ASP.NET Core 8 Petstore Server

An implementation of the [Swagger Petstore OpenAPI 3.1](../../spec/petstore-31.yaml) API using **C# ASP.NET Core 8** with **Npgsql + Dapper** for PostgreSQL persistence.

## Prerequisites

- [.NET 8 SDK](https://dotnet.microsoft.com/download/dotnet/8.0)
- PostgreSQL 17 (shared container in `../../database/`)

## Database Setup

The server uses the `csharp-aspnetcore` database in the shared PostgreSQL instance.

```bash
cd ../../database
docker compose up -d
./create-databases.sh
./apply-schemas.sh
```

## Build

```bash
dotnet build
```

## Test (no database required)

```bash
dotnet test tests/Petstore.Tests/Petstore.Tests.csproj
```

## Run

```bash
dotnet run --project src/Petstore
```

The server starts on **http://localhost:8080**. Swagger UI is at `http://localhost:8080/openapi/index.html`.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | *(unset)* | Full Npgsql connection string; overrides all `POSTGRES_*` vars |
| `POSTGRES_HOST` | `localhost` | PostgreSQL host |
| `POSTGRES_PORT` | `5434` | PostgreSQL port |
| `POSTGRES_USER` | `myuser` | PostgreSQL user |
| `POSTGRES_PASSWORD` | `mypassword` | PostgreSQL password |
| `POSTGRES_DB` | `csharp-aspnetcore` | PostgreSQL database name |
| `USE_IN_MEMORY_DB` | `false` | Set to `true` to use in-memory store (no DB required) |

Copy `.env.example` and adjust as needed. To source it before running:

```bash
set -a && source .env.example && set +a
dotnet run --project src/Petstore
```

## Mutation Testing (Stryker.NET)

```bash
# Install Stryker.NET tool if not already installed
dotnet tool install -g dotnet-stryker

dotnet stryker --config-file stryker-config.json
```

Report is written to `StrykerOutput/reports/`.

## Smoke-test Curl Examples

All endpoints are under the base path `/api/v3`.

```bash
BASE=http://localhost:8080/api/v3

# Add a pet
curl -s -X POST "$BASE/pet" \
  -H 'Content-Type: application/json' \
  -d '{"name":"Buddy","photoUrls":["http://example.com/photo.jpg"],"status":"available"}' | jq .

# Get the pet by ID
curl -s "$BASE/pet/1" | jq .

# Find pets by status
curl -s "$BASE/pet/findByStatus?status=available" | jq .

# Update a pet
curl -s -X PUT "$BASE/pet" \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"name":"Buddy Updated","photoUrls":["http://example.com/photo2.jpg"],"status":"pending"}' | jq .

# Delete a pet
curl -s -X DELETE "$BASE/pet/1"

# Store inventory
curl -s "$BASE/store/inventory" | jq .

# Place an order
curl -s -X POST "$BASE/store/order" \
  -H 'Content-Type: application/json' \
  -d '{"petId":1,"quantity":1,"status":"placed","complete":false}' | jq .

# Create a user
curl -s -X POST "$BASE/user" \
  -H 'Content-Type: application/json' \
  -d '{"username":"johndoe","firstName":"John","lastName":"Doe","email":"john@example.com","password":"secret","phone":"555-1234","userStatus":1}' | jq .

# Get user by name
curl -s "$BASE/user/johndoe" | jq .

# Login
curl -s "$BASE/user/login?username=johndoe&password=secret"

# Logout
curl -s "$BASE/user/logout"
```

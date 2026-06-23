# C# ASP.NET Core 8 — Agent Guide

## Working Directory

Run all commands from `petshop-stacks/csharp/aspnetcore/` unless stated otherwise.

## Quick Start

```bash
export PATH="$HOME/.dotnet:$PATH"
dotnet build
dotnet run --project src/Petstore    # listens on :8080
```

## Database

This server uses the shared PostgreSQL instance managed from `../../../database/`. It connects to the `csharp-aspnetcore` database.

### Set up the database (one time)

```bash
cd ../../../database
docker compose up -d
./create-databases.sh
./apply-schemas.sh
```

### Connection environment variables

| Variable | Default | Notes |
|---|---|---|
| `DATABASE_URL` | *(unset)* | Full Npgsql connection string; takes precedence |
| `POSTGRES_HOST` | `localhost` | |
| `POSTGRES_PORT` | `5434` | matches `database/.env` default |
| `POSTGRES_DB` | `csharp-aspnetcore` | |
| `POSTGRES_USER` | `myuser` | |
| `POSTGRES_PASSWORD` | `mypassword` | |
| `USE_IN_MEMORY_DB` | `false` | Set `true` to bypass Postgres (tests / dev) |
| `PG_MAX_POOL_SIZE` | *(unset)* | When set, appends `Maximum Pool Size=<value>` to the connection string (including `DATABASE_URL`, unless it already specifies a pool size). Unset = Npgsql default (100). `Dockerfile.optimized` sets `200` for the benchmark harness. |

### Database access design

The app uses a **singleton `NpgsqlDataSource`** (one shared connection pool) rather than constructing `NpgsqlConnection` objects from a raw connection string per operation:

- `Startup.ConfigureServices` registers `NpgsqlDataSource.Create(PostgresPetstoreRepository.BuildConnectionString())` as a singleton **via a lazy factory**, and only in the Postgres branch — when `USE_IN_MEMORY_DB=true` no data source is registered or created. Multiplexing is **not** enabled.
- `PostgresPetstoreRepository` takes the `NpgsqlDataSource` via constructor injection and calls `dataSource.OpenConnectionAsync()` per operation using `await using` (connections are returned to the pool on dispose).
- All repository methods are **async** (`Task<T>`) throughout — `IPetstoreRepository`, `PostgresPetstoreRepository`, `InMemoryPetstoreRepository`, and all three controllers — so thread-pool threads are never blocked waiting for I/O.
- Connection-string construction lives in the public static `PostgresPetstoreRepository.BuildConnectionString()` / `ApplyPoolSize()` helpers (env-var handling above, including `PG_MAX_POOL_SIZE`), keeping it unit-testable without a database.

### Performance optimizations applied

- **Async I/O end-to-end**: every controller action and repository method is `async Task<T>`, using Dapper's `QueryAsync` / `ExecuteAsync` / `QuerySingleAsync` / `QueryFirstOrDefaultAsync` — no thread-pool blocking under load.
- **Single-round-trip upserts**: `AddPet`, `PlaceOrder`, and `CreateUser` previously did `SELECT nextval(...)` + `INSERT` (two trips). Now a single `INSERT … VALUES (COALESCE(NULLIF(@Id, 0), nextval('…_id_seq')), …) … RETURNING id` handles both the sequence assignment and the upsert in one query.
- **Single-round-trip updates**: `UpdatePet` previously did `SELECT id` then `UPDATE`. Now a single `UPDATE … WHERE id = @Id` returning the affected row count is used. `UpdatePetWithForm` previously did `SELECT id, name, status` then `UPDATE`; now a single `UPDATE … SET name = COALESCE(@Name, name), status = CASE WHEN @Status IS NULL THEN status ELSE @Status::pet_status END WHERE id = @Id` handles null-preservation in one trip.
- **SQL-side tag filtering**: `FindPetsByTags` previously fetched all pets and filtered in C#. It now pushes the filter to PostgreSQL using `EXISTS (SELECT 1 FROM json_array_elements(tags) elem WHERE elem->>'name' = ANY(@Tags))`.
- **Shared connection for batch user inserts**: `CreateUsersWithListInput` opens one connection and reuses it across the loop (helper `InsertUserAsync` accepts a `NpgsqlConnection`).
- **Single-round-trip file upload**: `UploadFile` previously made three separate DB calls (check pet exists, get next sequence id, insert). Now a single CTE does all three: `WITH check_pet AS (SELECT id FROM pet WHERE id = @PetId), new_id AS (SELECT nextval('pet_photo_id_seq') AS id FROM check_pet) INSERT INTO pet_photo … SELECT … FROM new_id RETURNING id` — returns null rows when the pet does not exist.
- **Leaner middleware pipeline**: `UseHttpsRedirection`, `UseDefaultFiles`, and `UseStaticFiles` removed — this is a pure API server; those middlewares added per-request overhead for no benefit.

### Benchmark tuning (`Dockerfile.optimized` only)

The optimized Docker image additionally sets `ASPNETCORE_ENVIRONMENT=Production`, `Logging__LogLevel__Default=Warning`, `Logging__LogLevel__Microsoft.AspNetCore=Warning`, and `PG_MAX_POOL_SIZE=200` (pool of 200 handles up to 500 k6 VUs via queueing under Postgres `max_connections=500`). The naive `Dockerfile` is unchanged, so defaults preserve current behavior.

## Project Layout

```
petshop-stacks/csharp/aspnetcore/
├── Petstore.sln                          # Solution file
├── stryker-config.json                   # Stryker.NET mutation testing config
├── .env.example                          # Example env vars
├── src/Petstore/
│   ├── Petstore.csproj                   # Main project (net8.0)
│   ├── Program.cs                        # Entry point (generated)
│   ├── Startup.cs                        # DI wiring + middleware (modified)
│   ├── Controllers/
│   │   ├── PetApi.cs                     # Pet endpoints (hand-written over scaffold)
│   │   ├── StoreApi.cs                   # Store endpoints (hand-written over scaffold)
│   │   └── UserApi.cs                    # User endpoints (hand-written over scaffold)
│   ├── Models/                           # Generated model classes (do not edit)
│   │   ├── Pet.cs, Order.cs, User.cs, ...
│   ├── Repositories/
│   │   ├── IPetstoreRepository.cs        # Interface for all 19 operations (hand-written)
│   │   ├── PostgresPetstoreRepository.cs # Npgsql + Dapper implementation (hand-written)
│   │   └── InMemoryPetstoreRepository.cs # In-memory fake for tests (hand-written)
│   ├── Attributes/, Authentication/, Converters/,
│   │   Filters/, Formatters/, OpenApi/   # Generated infrastructure (do not edit)
│   └── appsettings.json                  # Generated app settings
└── tests/Petstore.Tests/
    ├── Petstore.Tests.csproj             # xUnit test project
    ├── PetTests.cs                       # Pet operation tests
    ├── StoreTests.cs                     # Store operation tests
    └── UserTests.cs                      # User operation tests
```

### Generated vs hand-written

**Generated (do not overwrite without care):**
- `src/Petstore/Models/*.cs`
- `src/Petstore/Attributes/`, `Authentication/`, `Converters/`, `Filters/`, `Formatters/`, `OpenApi/`
- `src/Petstore/Program.cs` (entry point is unchanged)
- `src/Petstore/appsettings*.json`

**Hand-written (preserve when re-running generator):**
- `src/Petstore/Controllers/PetApi.cs`, `StoreApi.cs`, `UserApi.cs`
- `src/Petstore/Startup.cs` (modified for DI)
- `src/Petstore/Repositories/*.cs`
- `tests/Petstore.Tests/**`

## Common Commands

```bash
# Build
dotnet build

# Test (no database required)
dotnet test tests/Petstore.Tests/Petstore.Tests.csproj

# Run server on port 8080
dotnet run --project src/Petstore

# Run with in-memory store (no Postgres)
USE_IN_MEMORY_DB=true dotnet run --project src/Petstore

# Mutation testing (install once: dotnet tool install -g dotnet-stryker)
dotnet stryker --config-file stryker-config.json
```

## Persistence Conventions

All implementations must be consistent with the shared database schema:

- **Tables**: `pet`, `"order"` (quoted), `"user"` (quoted), `pet_photo`
- **JSON columns**: `pet.category` (serialized `Category` object as JSON string), `pet.photo_urls` (JSON array), `pet.tags` (JSON array of Tag objects)
- **Enum columns**: `pet.status` uses `pet_status` PostgreSQL enum (`available`, `pending`, `sold`). `order.status` uses `order_status` enum (`placed`, `approved`, `delivered`). Cast on write: `@Status::pet_status` / `@Status::order_status`. Read as `status::text`.
- **Server-assigned IDs**: `SELECT nextval('<table>_id_seq')` (sequences: `pet_id_seq`, `order_id_seq`, `user_id_seq`, `pet_photo_id_seq`) when request omits or sends `0`.
- **Upserts**: `INSERT ... ON CONFLICT (id) DO UPDATE SET ...`.
- **uploadFile**: verify pet exists → read body bytes → insert raw bytes into `pet_photo.content` (BYTEA keyed by `pet_id`) → return `ApiResponse` with byte count in the message.
- **logoutUser**: stateless no-op, return HTTP 200.

## Mutation Testing

[Stryker.NET](https://stryker-mutator.io/docs/stryker-net/introduction/) is configured in `stryker-config.json`.
It targets `InMemoryPetstoreRepository.cs` (the in-memory implementation), running the xUnit test suite (no DB required).
`PostgresPetstoreRepository.cs` is excluded from the mutate scope because it requires a live Postgres connection and its mutants would all be NoCoverage without one.

```bash
dotnet tool install -g dotnet-stryker   # one-time install
dotnet stryker --config-file stryker-config.json
```

HTML report is written to `StrykerOutput/reports/mutation-report.html`.

**Current mutation score: 100% (51/51 mutants killed)**  
Achieved by 52 xUnit tests in `tests/Petstore.Tests/` (up from 35).

Key mutations covered:
- ID auto-increment sequencing (AddPet, PlaceOrder, CreateUser) — equality boundary and arithmetic
- `FindPetsByTags` with empty list (`||` vs `&&`)
- `CreateUsersWithListInput` with null/empty input (`||` vs `&&`)
- `UploadFile` exact byte count (statement mutation on `CopyTo`)
- `UpdateUser` return value (`true` vs `false`)
- `StringToPetStatus("pending")` and `PetStatusToString(PendingEnum)` string mutations
- `GetInventory` with invalid enum value (`?? "unknown"` null-coalescing)

## Verification Curl Examples

```bash
BASE=http://localhost:8080/api/v3

curl -s -X POST "$BASE/pet" \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://x.com/img.jpg"],"status":"available"}' | jq .id

curl -s "$BASE/pet/1" | jq .name

curl -s "$BASE/store/inventory" | jq .

curl -s -X POST "$BASE/store/order" \
  -H 'Content-Type: application/json' \
  -d '{"petId":1,"quantity":1,"status":"placed","complete":false}' | jq .id

curl -s -X POST "$BASE/user" \
  -H 'Content-Type: application/json' \
  -d '{"username":"test","firstName":"Test","password":"pw","userStatus":1}' | jq .username
```

# Java Helidon — Petstore Server

OpenAPI-generated Petstore server built on **Helidon MP 4** (MicroProfile / JAX-RS + CDI) with PostgreSQL persistence via plain JDBC and a HikariCP connection pool.

## Prerequisites

- JDK 21+ (required by Helidon 4; **Java 25 recommended** — the Docker images build on Temurin 25)
- Maven 3.8+

Request handling runs on **virtual threads by default** (Helidon MP 4's WebServer is Loom-based), so no extra configuration is needed. The optimized Docker image uses **G1GC** (previously ZGC); under the perf harness's `512m` memory limit the heap is small (~384MB), where G1 is more memory-efficient than ZGC.
- Docker (with Compose v2) for the database

## Database

This server persists to the shared PostgreSQL instance in `../../database/` and uses the `java-helidon` database.

```bash
cd ../../database
docker compose up -d
./create-databases.sh          # creates java-helidon (idempotent)
./apply-schemas.sh             # applies petstore + OAuth2 tables
cd ../../java/helidon
```

### Connection configuration

Defaults live in `src/main/resources/META-INF/microprofile-config.properties` and can be overridden by environment variables:

| Variable | Default | Notes |
|---|---|---|
| `POSTGRES_HOST` | `localhost` | |
| `POSTGRES_PORT` | `5432` | the shared Compose stack uses `5434` |
| `POSTGRES_USER` | `postgres` | shared stack default is `myuser` |
| `POSTGRES_PASSWORD` | `mysecret` | shared stack default is `mypassword` |
| `POSTGRES_DB` | `java-helidon` | |

To run against the shared Compose stack, export the matching values from `database/.env`:

```bash
set -a && source ../../database/.env && set +a
export POSTGRES_DB=java-helidon
```

## Build and Run

```bash
mvn package
java -jar target/petstore-helidon.jar
```

The server listens on `http://localhost:8080` and serves the API under `/api/v3`.

## Try It

```bash
# Add a pet
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'

# Fetch it back
curl -s http://localhost:8080/api/v3/pet/1

# Inventory by status
curl -s http://localhost:8080/api/v3/store/inventory
```

Health and metrics are exposed by MicroProfile:

```bash
curl -s http://localhost:8080/health
curl -s http://localhost:8080/metrics
```

## Project Layout

```
src/main/java/org/openapitools/server/
├── api/        # JAX-RS resources — *ServiceImpl classes hold the endpoint logic
├── db/         # persistence layer (DataSourceProvider + *Repository) — edit here
└── model/      # generated DTOs (Pet, Order, User, …)
```

Endpoint logic lives in `api/*ServiceImpl.java`, which delegate all SQL to the repositories in `db/`. See `AGENTS.md` for implementation conventions.

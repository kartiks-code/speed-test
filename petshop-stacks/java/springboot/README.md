# Java Spring Boot — Petstore Server

OpenAPI-generated Petstore server built on **Spring Boot 3.5.15** (Java 25) with PostgreSQL persistence via Spring's `JdbcTemplate`. Virtual threads are enabled (`spring.threads.virtual.enabled=true`).

## Prerequisites

- JDK 25+
- Maven 3.8+
- Docker (with Compose v2) for the database

## Database

This server persists to the shared PostgreSQL instance in `../../database/` and uses the `java-springboot` database.

```bash
cd ../../../database
docker compose up -d
./create-databases.sh          # creates java-springboot (idempotent)
./apply-schemas.sh             # applies petstore + OAuth2 tables
cd ../../../petshop-stacks/java/springboot
```

### Connection configuration

Defaults are in `src/main/resources/application.properties` and align with `database/.env`. Override via environment variables (or set `DATABASE_URL` directly):

| Variable | Default |
|---|---|
| `POSTGRES_HOST` | `localhost` |
| `POSTGRES_PORT` | `5434` |
| `POSTGRES_DB` | `java-springboot` |
| `POSTGRES_USER` | `myuser` |
| `POSTGRES_PASSWORD` | `mypassword` |
| `DATABASE_URL` | (full JDBC URL; overrides host/port/db) |
| `SPRING_DATASOURCE_HIKARI_MAXIMUM_POOL_SIZE` | `200` (HikariCP max pool size; kept large because request handling runs on virtual threads) |

```bash
set -a && source ../../database/.env && set +a
export POSTGRES_DB=java-springboot
```

## Build and Run

```bash
mvn package
java -jar target/petstore-server-1.0.0.jar
# or, for iterative development:
mvn spring-boot:run
```

The server listens on `http://localhost:8080` and serves the API under `/api/v3`.

- Swagger UI: `http://localhost:8080/swagger-ui.html`
- OpenAPI JSON: `http://localhost:8080/v3/api-docs`

### Docker

`Dockerfile` builds a simple image; `Dockerfile.optimized` produces a layered Temurin 25 JRE image that runs with **G1GC** (`-XX:+UseG1GC`, `-XX:MaxRAMPercentage=75.0`), tuned for the `--cpus 2 --memory 512m` performance-test limits.

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

## Project Layout

```
src/main/java/org/openapitools/
├── api/            # *ApiController implementations (delegate to PetStore) + ApiExceptionHandler
├── persistence/    # PetStore (all SQL) + Not/InvalidInput exceptions — edit here
├── configuration/  # springdoc + home redirect config
└── model/          # generated DTOs (Pet, Order, User, …)
```

All business logic and SQL live in `persistence/PetStore.java`. See `AGENTS.md` for conventions.

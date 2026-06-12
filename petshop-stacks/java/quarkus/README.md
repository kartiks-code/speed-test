# Petstore Quarkus Server

Petstore API implementation using [Quarkus](https://quarkus.io/) 3.36.x (latest), built with Gradle 9.5.x (Groovy DSL) on Java 25.

The API is served under `/api/v3` on port 8080 and backed by the shared PostgreSQL 17 database in `../../database/`.

All endpoints run on **Java virtual threads** via `@io.smallrye.common.annotation.RunOnVirtualThread` (from `quarkus-rest`). Quarkus has no global virtual-thread switch, so the annotation is applied at the class level on each JAX-RS resource (`PetApiImpl`, `StoreApiImpl`, `UserApiImpl`), which is well-suited to the blocking JDBC persistence. Because of this, the Agroal connection pool (`quarkus.datasource.jdbc.max-size`) is sized large (200) — a pool capped like a platform-thread pool would bottleneck virtual-thread concurrency and can deadlock under load. The optimized Docker image (`Dockerfile.optimized`) runs on Temurin 25 with `-XX:+UseG1GC`.

## Quick Start

```bash
# 1. Start the shared database
cd ../../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh
cd -

# 2. Build and run
./gradlew build
java -jar build/*-runner.jar
```

The server is now available at `http://localhost:8080/api/v3`.

## Development

```bash
# Hot-reload dev mode (requires DB)
./gradlew quarkusDev

# Tests only (no DB required)
./gradlew test

# Mutation testing
./gradlew pitest
```

## Stack

| Component | Version |
|---|---|
| Quarkus | 3.36.1 |
| Java | 25 |
| Gradle | 9.5.1 (wrapper) |
| RESTEasy / Quarkus REST | via Quarkus BOM |
| Agroal (connection pool) | via Quarkus BOM |
| PostgreSQL driver | via Quarkus BOM |
| JUnit 5 + Mockito | 5.14.2 |
| PIT mutation testing | 1.19.0 plugin |

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `POSTGRES_HOST` | `localhost` | PostgreSQL host |
| `POSTGRES_PORT` | `5434` | PostgreSQL port |
| `POSTGRES_USER` | `myuser` | PostgreSQL user |
| `POSTGRES_PASSWORD` | `mypassword` | PostgreSQL password |
| `POSTGRES_DB` | `java-quarkus` | PostgreSQL database name |
| `QUARKUS_DATASOURCE_JDBC_MAX_SIZE` | `200` | Agroal max pool size; kept large because endpoints run on virtual threads (must be ≥ concurrent request count) |

## API Endpoints

All 19 Petstore operations are implemented:

| Tag | Operations |
|---|---|
| pet | `POST /pet`, `PUT /pet`, `GET /pet/findByStatus`, `GET /pet/findByTags`, `GET /pet/{petId}`, `POST /pet/{petId}`, `DELETE /pet/{petId}`, `POST /pet/{petId}/uploadImage` |
| store | `GET /store/inventory`, `POST /store/order`, `GET /store/order/{orderId}`, `DELETE /store/order/{orderId}` |
| user | `POST /user`, `POST /user/createWithList`, `GET /user/login`, `GET /user/logout`, `GET /user/{username}`, `PUT /user/{username}`, `DELETE /user/{username}` |

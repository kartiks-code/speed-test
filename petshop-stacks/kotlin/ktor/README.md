# Kotlin + Ktor Petstore Server

OpenAPI Petstore implementation using Kotlin 2.3, Ktor 3.4, and PostgreSQL 17. One of the language servers in the [speed-test](../..) repository.

## Prerequisites

- JDK 21+ (tested with Corretto 21)
- Gradle 8.11+ (wrapper included — no separate install needed)
- PostgreSQL 17 container started via `database/docker compose up -d`

## Quick Start

```bash
# 1. Start the shared database (from repo root)
cd ../../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh

# 2. Build and test (from this directory)
cd ../../../petshop-stacks/kotlin/ktor
./gradlew test        # DB-free unit tests
./gradlew run         # starts server on :8080
```

## Commands

| Goal | Command |
|---|---|
| Compile | `./gradlew build` |
| Run tests (no DB) | `./gradlew test` |
| Start server | `./gradlew run` |
| Mutation testing | `./gradlew pitest` |

## Environment Variables

Copy `.env.example` to `.env` and adjust as needed. The server reads these at startup:

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | _(not set)_ | Full JDBC URL; takes precedence over individual vars |
| `POSTGRES_HOST` | `localhost` | PostgreSQL host |
| `POSTGRES_PORT` | `5434` | PostgreSQL port |
| `POSTGRES_DB` | `kotlin-ktor` | Database name |
| `POSTGRES_USER` | `myuser` | Database user |
| `POSTGRES_PASSWORD` | `mypassword` | Database password |

## API

Base path: `/api/v3`. All 19 Petstore operations are implemented:

- **Pet**: `POST /pet`, `PUT /pet`, `GET /pet/{petId}`, `DELETE /pet/{petId}`, `GET /pet/findByStatus`, `GET /pet/findByTags`, `POST /pet/{petId}`, `POST /pet/{petId}/uploadImage`
- **Store**: `GET /store/inventory`, `POST /store/order`, `GET /store/order/{orderId}`, `DELETE /store/order/{orderId}`
- **User**: `POST /user`, `POST /user/createWithList`, `GET /user/login`, `GET /user/logout`, `GET /user/{username}`, `PUT /user/{username}`, `DELETE /user/{username}`

## Verification

```bash
./gradlew run &
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'
curl -s http://localhost:8080/api/v3/store/inventory
```

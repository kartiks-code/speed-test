# Java Spring Boot Server — Agent Guide

OpenAPI-generated Petstore server on Spring Boot 3.3 with PostgreSQL persistence via `JdbcTemplate`.

## Working Directory

Run all commands from `java/springboot/` unless stated otherwise. The server listens on `:8080` and serves the API under `/api/v3`.

## Build, Run, Verify

```bash
mvn package                              # compile + run tests + build the jar
java -jar target/petstore-server-1.0.0.jar
mvn spring-boot:run                      # run without packaging
mvn test                                 # tests only
```

Requires JDK 17+ and Maven 3.8+.

## Database

Uses the shared PostgreSQL stack in `../../database/`, database `java-springboot`.

```bash
cd ../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh
```

Connection defaults in `src/main/resources/application.properties` already match `database/.env` (host `localhost`, port `5434`, user `myuser`, password `mypassword`, db `java-springboot`). Override with `POSTGRES_*` env vars or a full `DATABASE_URL`.

## Code Structure

```
src/main/java/org/openapitools/
├── api/
│   ├── PetApi.java / PetApiController.java        # generated interface + impl controller
│   ├── StoreApi.java / StoreApiController.java
│   ├── UserApi.java / UserApiController.java
│   ├── ApiExceptionHandler.java                   # maps persistence exceptions → HTTP status
│   └── ApiUtil.java
├── persistence/
│   ├── PetStore.java                              # @Repository — all SQL + RowMappers
│   ├── NotFoundException.java                     # → 404
│   └── InvalidInputException.java                 # → 400
├── configuration/                                 # springdoc + home controller
└── model/                                         # generated DTOs (do not hand-edit)
```

## Conventions

- `*ApiController` classes implement the generated `*Api` interfaces and are thin: they delegate to `PetStore` and wrap results in `ResponseEntity`. Put orchestration here.
- `PetStore` is a `@Repository` using `JdbcTemplate` (no ORM). All SQL, `RowMapper`s, validation, and ID generation live here.
- Domain errors throw `NotFoundException` (404) or `InvalidInputException` (400); `ApiExceptionHandler` maps them to responses. Don't return raw 500s for expected cases.
- `category`, `photo_urls`, and `tags` are JSON columns written via a `PGobject` of type `json` (serialized with Jackson). `category` is stored as a JSON string.
- Enum columns (`pet.status` = `pet_status`, `order.status` = `order_status`) are written with a `PGobject` of the enum type and read as `status::text`, then mapped via the model enum `fromValue`.
- IDs default to `COALESCE(MAX(id), 0) + 1` when omitted; writes use `INSERT … ON CONFLICT … DO UPDATE` upserts. `createUsers` is `@Transactional`.
- `uploadFile` returns a `ModelApiResponse` describing the upload but does not persist binary data; `logoutUser` is a no-op.
- Tables used: `pet`, `"order"`, `"user"` (quoted reserved words).

## Generated vs. Hand-Written

- `model/`, the `*Api.java` interfaces, and configuration scaffolding are generated — avoid hand edits.
- `*ApiController.java` and everything under `persistence/` are the hand-written implementation. Preserve them if the project is regenerated.

## Verification

```bash
mvn package
java -jar target/petstore-server-1.0.0.jar &
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'
curl -s http://localhost:8080/api/v3/store/inventory
```

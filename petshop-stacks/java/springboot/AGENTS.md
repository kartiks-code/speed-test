# Java Spring Boot Server — Agent Guide

OpenAPI-generated Petstore server on Spring Boot 3.5.15 (Java 25) with PostgreSQL persistence via `JdbcTemplate`. Virtual threads are enabled via `spring.threads.virtual.enabled=true`.

## Working Directory

Run all commands from `petshop-stacks/java/springboot/` unless stated otherwise. The server listens on `:8080` and serves the API under `/api/v3`.

## Build, Run, Verify

```bash
mvn package                              # compile + run tests + build the jar
java -jar target/petstore-server-1.0.0.jar
mvn spring-boot:run                      # run without packaging
mvn test                                 # tests only
```

Requires JDK 25+ and Maven 3.8+.

The optimized Docker image (`Dockerfile.optimized`, Temurin 25 JRE) runs with **G1GC** (`-XX:+UseG1GC`), chosen for the perf harness's `--cpus 2 --memory 512m` container limits.

It also uses the **Project Leyden AOT cache** (JEP 483, JDK 24+). A dedicated `training` build stage runs the app with `-XX:AOTMode=record -XX:AOTCacheOutput=app.aot` and `spring.datasource.hikari.initialization-fail-timeout=-1` so HikariCP does not block on an unavailable DB; SIGTERM after 10 s triggers a graceful JVM shutdown whose shutdown hooks record the class-loading profile and assemble the cache. The runtime stage copies `app.aot` and passes `-XX:AOTCache=app.aot` on startup; if the cache file is invalid the JVM falls back gracefully.

## Database

Uses the shared PostgreSQL stack in `../../../database/`, database `java-springboot`.

```bash
cd ../../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh
```

Connection defaults in `src/main/resources/application.properties` already match `database/.env` (host `localhost`, port `5434`, user `myuser`, password `mypassword`, db `java-springboot`). Override with `POSTGRES_*` env vars or a full `DATABASE_URL`.

**HikariCP pool size:** because request handling runs on virtual threads (`spring.threads.virtual.enabled=true`), the pool is set large (`spring.datasource.hikari.maximum-pool-size=200`) rather than left at HikariCP's default of 10, which would bottleneck virtual-thread concurrency. Override with `SPRING_DATASOURCE_HIKARI_MAXIMUM_POOL_SIZE`; keep it ≥ the benchmark VU count, bounded by Postgres `max_connections` (raised to 500 in `database/docker-compose.yml`).

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
- IDs default to `SELECT nextval('<table>_id_seq')` (sequences: `pet_id_seq`, `order_id_seq`, `user_id_seq`, `pet_photo_id_seq`) when omitted; writes use `INSERT … ON CONFLICT … DO UPDATE` upserts. `createUsers` is `@Transactional`.
- `uploadFile` reads the request body into a `byte[]` and persists it via `PetStore.savePetPhoto` into the `pet_photo` table; the returned `ModelApiResponse` reports the stored byte count. `logoutUser` is a no-op.
- Tables used: `pet`, `"order"`, `"user"` (quoted reserved words).

## Generated vs. Hand-Written

- `model/`, the `*Api.java` interfaces, and configuration scaffolding are generated — avoid hand edits.
- `*ApiController.java` and everything under `persistence/` are the hand-written implementation. Preserve them if the project is regenerated.

## Mutation Testing

[PIT](https://pitest.org) (pitest-maven 1.25.4) is configured in `pom.xml`.
It targets the hand-written `*ApiController`, `ApiExceptionHandler`, and
`persistence.*` classes. Generated model/interface/config code is excluded.
No live database is required — the unit tests mock `JdbcTemplate`.

```bash
# Run mutation analysis (produces target/pit-reports/index.html):
mvn test-compile org.pitest:pitest-maven:mutationCoverage

# Incremental re-run — only re-tests classes changed since the last run:
mvn test-compile org.pitest:pitest-maven:mutationCoverage -DwithHistory
```

A surviving mutant means no test distinguishes the mutated bytecode from the
original. Fix survivors by adding a sharper assertion or confirm they are
equivalent mutations.

## Verification

```bash
mvn package
java -jar target/petstore-server-1.0.0.jar &
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'
curl -s http://localhost:8080/api/v3/store/inventory
```

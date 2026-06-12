# Java Quarkus Server — Agent Guide

OpenAPI Petstore server on Quarkus 3.36.x (latest) with JAX-RS resources, Arc CDI, Agroal connection pooling, and plain-JDBC persistence. Built with Gradle 9.5.x (Groovy DSL) on Java 25. All endpoints run on virtual threads via `@RunOnVirtualThread`, and the optimized Docker image (`Dockerfile.optimized`) runs on Temurin 25 with `-XX:+UseG1GC`, `-XX:InitialRAMPercentage=75.0`/`-XX:MaxRAMPercentage=75.0`, `-XX:+OptimizeStringConcat`, and `QUARKUS_LOG_LEVEL=WARN` (set via image `ENV`) to cut per-request logging overhead.

The optimized Docker image (`Dockerfile.optimized`) uses **AppCDS** (Application Class Data Sharing, `-XX:ArchiveClassesAtExit` / `-XX:SharedArchiveFile`) for faster startup. A dedicated `training` build stage runs the uber-jar with `quarkus.datasource.jdbc.initial-size=0` / `min-size=0` (so Agroal creates no connections at build time); SIGTERM after 10 s causes the JVM to exit and write the class archive. The runtime stage copies `app.jsa` and passes `-XX:SharedArchiveFile=app.jsa` on startup. The training and runtime stages must use the **same GC/heap flags** (G1GC, Initial/MaxRAMPercentage, OptimizeStringConcat) — AppCDS archives are flag-sensitive and a mismatch makes the JVM silently fall back to non-shared class loading.

> **Why AppCDS and not Leyden AOT (`-XX:AOTCache`)?** Leyden's `-XX:AOTCache` (JEP 483) was tested but caused a consistent virtual-thread deadlock under Docker CPU limits (`--cpus 2`): the AOT-compiled Agroal path pinned both carrier threads, starving all other virtual threads. AppCDS archives only class metadata (no compiled code), so it does not interfere with JEP 491 virtual-thread scheduling.

## Working Directory

Run all commands from `petshop-stacks/java/quarkus/` unless stated otherwise. The server listens on `:8080` and serves the API under `/api/v3`.

## Build, Run, Verify

```bash
./gradlew build                          # compile + run tests + build uber-jar
java -jar build/quarkus-app/quarkus-run.jar  # run the built jar (fast-jar layout)
# OR with uber-jar (configured via gradle.properties):
java -jar build/*-runner.jar

./gradlew test                           # tests only
./gradlew quarkusDev                     # hot-reload dev mode (requires DB to be up)
```

Requires JDK 25 (provisioned automatically via Foojay toolchain resolver if not installed) and Gradle 9.5.1 wrapper (auto-downloads from `gradle-wrapper.properties`).

## Database

Uses the shared PostgreSQL stack in `../../../database/`, database `java-quarkus`.

```bash
cd ../../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh
```

Connection defaults are in `src/main/resources/application.properties` (host `localhost`, port `5434`, user `myuser`, password `mypassword`, db `java-quarkus`, `jdbc.max-size=200`). Each key is overridable by the matching environment variable: `POSTGRES_HOST`, `POSTGRES_PORT`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`, `QUARKUS_DATASOURCE_JDBC_MAX_SIZE`. **Because endpoints run on virtual threads, the pool is intentionally large (200) rather than capped like a platform-thread pool** — a small pool both bottlenecks concurrency and *deadlocks* under `@RunOnVirtualThread` + Agroal (connection acquisition pins the limited carrier threads under `--cpus 2`, so VUs waiting for a connection starve the VUs holding them). Keep `max-size` ≥ the benchmark VU count; it is bounded by Postgres `max_connections` (raised to 500 in `database/docker-compose.yml`).

## Code Structure

```
src/main/java/org/openapitools/server/
├── api/
│   ├── PetApi.java / PetApiImpl.java        # JAX-RS resource interface + impl
│   ├── StoreApi.java / StoreApiImpl.java
│   ├── UserApi.java / UserApiImpl.java
│   └── PetstoreApplication.java             # JAX-RS Application root
├── db/
│   ├── PetRepository.java                   # pet CRUD + inventory
│   ├── OrderRepository.java
│   └── UserRepository.java
└── model/                                   # generated DTOs (do not hand-edit)
```

## Conventions

- `*ApiImpl` classes are thin JAX-RS resources annotated `@ApplicationScoped` and `@io.smallrye.common.annotation.RunOnVirtualThread`; they `@Inject` a repository and delegate. Put endpoint logic here.
- All endpoints run on **virtual threads**: the class-level `@RunOnVirtualThread` (provided transitively by `quarkus-rest`) is applied to all three `*ApiImpl` classes. Quarkus has no global virtual-thread switch, so it is enabled per JAX-RS resource/method. This suits the blocking JDBC (Agroal) persistence used here.
- Repositories are `@ApplicationScoped`, `@Inject` Agroal `javax.sql.DataSource`, and use plain JDBC (`Connection`/`PreparedStatement`). No ORM.
- Errors are signaled with `jakarta.ws.rs.WebApplicationException` carrying the right status (404 not found, 400 bad request); these propagate to the HTTP response.
- `category`, `photo_urls`, and `tags` are JSON columns written with Jackson `ObjectMapper` and `cast(? as json)`. `category` is stored as a JSON string.
- Enum columns (`pet.status` = `pet_status`, `order.status` = `order_status`) are written with `cast(? as pet_status)` and read as `status::text`, then mapped via the model enum `fromValue`.
- IDs use an `AtomicLong` counter (seeded at startup) when the request omits one.
- `uploadFile` reads the uploaded body into a `byte[]` and persists it via `PetRepository.savePhoto` into the `pet_photo` table; the returned `ModelApiResponse` reports the stored byte count. `logoutUser` is a no-op.
- Tables used: `pet`, `"order"`, `"user"` (quoted reserved words), and `pet_photo`.

## Generated vs. Hand-Written

- `model/` and `*Api.java` interfaces are generated/adapted artifacts — avoid hand edits; they may be overwritten on regeneration.
- `*ApiImpl.java` and everything under `db/` are the hand-written implementation. Preserve them if the project is regenerated.

## Mutation Testing

[PIT](https://pitest.org) (`info.solidsoft.pitest` 1.19.0, Gradle 9 compatible) is configured in `build.gradle`.
It targets the hand-written `*ApiImpl` and `db.*` classes.
Generated model/interface code is excluded. The unit tests use plain Mockito (no CDI container), so PIT instruments them without issues.

```bash
# Run mutation analysis (produces build/reports/pitest/index.html):
./gradlew pitest

# Incremental re-run — only re-tests classes changed since the last run:
./gradlew pitest --history
```

A surviving mutant means no test distinguishes the mutated bytecode from the original. Fix survivors by adding a sharper assertion or confirm they are equivalent mutations.

## Verification

```bash
./gradlew build
java -jar build/*-runner.jar &
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'
curl -s http://localhost:8080/api/v3/store/inventory
```

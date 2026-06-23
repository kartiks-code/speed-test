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

The optimized Docker image (`Dockerfile.optimized`) runs with **G1GC** (`-XX:+UseG1GC`) on Temurin 25 JRE, pinned heap (`-XX:InitialRAMPercentage=75.0 -XX:MaxRAMPercentage=75.0`), and a **Project Leyden AOT cache** (JEP 483). See the Optimized Docker Image section below.

The experimental Docker image (`Dockerfile.graalvm`) is the **GraalVM CE 25 native-image build**. It appears as the **"experimental"** variant in the frontend and CLI. See the GraalVM Native Image section below.

The CRaC Docker image (`Dockerfile.crac`) snapshots a fully-warmed JVM process using **CRaC / CRIU** and restores it on startup. It appears as the **"crac"** variant in the frontend and CLI. See the CRaC section below.

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

## Optimized Docker Image (`Dockerfile.optimized`)

Runs with **G1GC** (`-XX:+UseG1GC`), chosen for the perf harness's `--cpus 2 --memory 512m` container limits. The heap is pinned with `-XX:InitialRAMPercentage=75.0 -XX:MaxRAMPercentage=75.0` so it does not resize during the measured window, and benchmark-only system properties disable SpringDoc and request logging (`-Dspringdoc.swagger-ui.enabled=false -Dspringdoc.api-docs.enabled=false -Dlogging.level.root=WARN`); `application.properties` is untouched, so the naive image keeps SpringDoc enabled. These flags are passed identically in the AOT training stage and the runtime `ENTRYPOINT`.

It also uses the **Project Leyden AOT cache** (JEP 483, JDK 24+). A dedicated `training` build stage runs the app with `-XX:AOTMode=record -XX:AOTCacheOutput=app.aot` and `spring.datasource.hikari.initialization-fail-timeout=-1` so HikariCP does not block on an unavailable DB; SIGTERM after 10 s triggers a graceful JVM shutdown whose shutdown hooks record the class-loading profile and assemble the cache. The runtime stage copies `app.aot` and passes `-XX:AOTCache=app.aot` on startup; if the cache file is invalid the JVM falls back gracefully.

## GraalVM Native Image — Experimental (`Dockerfile.graalvm`)

Appears as the **"experimental"** variant in the frontend and CLI. Builds a self-contained native binary using GraalVM CE 25 (`ghcr.io/graalvm/native-image-community:25`).

**Reliability: HIGH.** Spring Boot 3.5 has mature native image support via Spring AOT processing. All components used here ship native image metadata:
- JdbcTemplate / HikariCP: handled by `spring-boot-autoconfigure` AOT hints
- Jackson: bundled native-image metadata since 2.15
- springdoc-openapi 2.3+: native hints included
- Virtual threads (`spring.threads.virtual.enabled`): SubstrateVM Loom support since GraalVM 21

**Build:** `mvn -Pnative native:compile -DskipTests` (uses the `native` profile added to `pom.xml`; the `spring-boot-starter-parent` 3.5 manages the `native-maven-plugin` version automatically).

**Reflection config:** `src/main/resources/META-INF/native-image/reflect-config.json` registers `org.openapitools.RFC3339DateFormat` for reflection. This is required because `application.properties` sets `spring.jackson.date-format=org.openapitools.RFC3339DateFormat` as a class name; in native image, `ClassUtils.forName()` fails for unregistered classes, causing Spring to fall back to `new SimpleDateFormat("org.openapitools.RFC3339DateFormat")`, which crashes on the letter `'o'`.

**Output:** `target/petstore-server` (Linux native binary, ~101 MB on disk, ~80–120 MB RSS).

**Startup:** < 100 ms JVM start; ~143 ms container-to-HTTP-ready.

The native profile in `pom.xml` activates `native-maven-plugin` with `--no-fallback` so build failures surface immediately instead of silently falling back to JVM mode.

Benchmark usage (via `performance-tests/run.sh`):
```bash
cd performance-tests
VUS=3 DURATION=15s ./run.sh springboot experimental
```

## CRaC — Checkpoint/Restore (`Dockerfile.crac`)

Appears as the **"crac"** variant in the frontend and CLI. Snapshots the fully-initialised JVM process using **CRaC** (Coordinated Restore at Checkpoint) built on **CRIU**, producing sub-100 ms restore startup and a warmed-up JVM state retained across restarts.

**Base image:** `azul/zulu-openjdk:25-jdk-crac` (Azul Zulu JDK 25 with CRaC/CRIU support, Ubuntu/glibc). BellSoft only publishes CRaC images up to JDK 21; Azul has JDK 25.

**Dependency:** `org.crac:crac` is added to `pom.xml` (version managed by `spring-boot-starter-parent`). HikariCP 5.x activates built-in CRaC hooks when this library is on the classpath.

**Checkpoint trigger:** `crac-checkpoint.sh` starts the JVM with `-XX:CRaCCheckpointTo=/checkpoint`, polls Tomcat at `/` (no DB traffic — lazy `DataSource` stays uninitialized), resolves the child JVM PID via `jps` (CRaC forks a wrapper process so `$!` is wrong), then calls `jcmd $PID JDK.checkpoint`. `-Dspring.context.checkpoint=onRefresh` is **not** used: it checkpoints before `HikariCheckpointRestoreLifecycle` can suspend the pool, and any open PostgreSQL socket fails the checkpoint. The script also excludes JPA auto-config (unused here) and passes `-Dspring.datasource.hikari.allow-pool-suspension=true`, `minimum-idle=0`, and `initialization-fail-timeout=-1`. After restore, the benchmark container's env vars supply the real Postgres URL and connections open on first request.

**Two-phase build performed by `run.sh`:**

1. **Phase A — `docker build`**: produces the checkpoint-ready image with `ENTRYPOINT ["/app/crac-checkpoint.sh"]`. The script starts the app with `-XX:CRaCCheckpointTo=/checkpoint`, waits for HTTP readiness, then calls `jcmd JDK.checkpoint`; CRIU writes process state to `/checkpoint` and the JVM exits.

2. **Phase B — `docker run` + `docker commit`**: `run.sh` runs the checkpoint-ready image with `--cap-add CHECKPOINT_RESTORE --cap-add SYS_PTRACE --security-opt seccomp=unconfined --network database_default` (and matching `--cpus`/`--memory` so heap sizing in the checkpoint matches the benchmark container). After the container exits, `docker commit --change 'ENTRYPOINT ["java", "-XX:CRaCRestoreFrom=/checkpoint"]'` bakes the checkpoint files into the final image. The benchmark container is then run **without** special caps — restore is unprivileged.

**Expected characteristics:**
- Startup: < 100 ms container-to-HTTP-ready (vs ~1 s for Leyden AOT, < 150 ms for GraalVM native)
- Steady-state throughput: comparable to `optimized` (still HotSpot + JIT, retains JIT warmth across restarts)
- Memory: similar to `optimized` (~200–300 MB RSS)
- Build time: slower than `optimized` (adds a checkpoint-run + commit phase)

Benchmark usage:
```bash
cd performance-tests
VUS=3 DURATION=15s ./run.sh springboot crac
```

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

**Current mutation score (June 2026):** 98% overall (182/185 killed), 100% test
strength, 93 unit tests in `PetStoreTest`. Three uncovered mutations remain and
are all equivalent or in generated Spring boilerplate (`getRequest()`, and a
`fromJsonList` early-return that returns `new ArrayList<>()` vs
`Collections.emptyList()` — the caller never modifies the list).

## Verification

```bash
mvn package
java -jar target/petstore-server-1.0.0.jar &
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'
curl -s http://localhost:8080/api/v3/store/inventory
```

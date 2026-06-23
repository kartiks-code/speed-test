# Java Helidon Server — Agent Guide

OpenAPI-generated Petstore server on Helidon MP 4 (MicroProfile / JAX-RS + CDI) with JDBC + HikariCP persistence.

## Working Directory

Run all commands from `petshop-stacks/java/helidon/` unless stated otherwise. The server listens on `:8080` and serves the API under `/api/v3`.

## Build, Run, Verify

```bash
mvn package                          # compile + run tests + build target/petstore-helidon.jar
java -jar target/petstore-helidon.jar
mvn test                             # tests only
```

Requires JDK 21+ (Helidon 4; **Java 25 recommended** and used by the Docker images) and Maven 3.8+. Compiles to release 25 (`maven.compiler.source=25` in `pom.xml`). Helidon MP 4's WebServer is Loom-based, so request handling runs on **virtual threads by default** — no code change needed.

## Database

Uses the shared PostgreSQL stack in `../../../database/`, database `java-helidon`.

```bash
cd ../../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh
```

Connection defaults are in `src/main/resources/META-INF/microprofile-config.properties` (`POSTGRES_HOST` `localhost`, `POSTGRES_PORT` `5432`, `POSTGRES_USER` `postgres`, `POSTGRES_PASSWORD` `mysecret`, `POSTGRES_DB` `java-helidon`). Each key is overridable by the matching environment variable. The shared Compose stack uses port `5434` / `myuser` / `mypassword`, so override those when targeting it.

**HikariCP pool size:** Helidon MP 4's WebServer is Loom-based (requests run on virtual threads), so the pool in `DataSourceProvider` is intentionally large (default `200`) rather than capped like a platform-thread pool — a small pool needlessly bottlenecks virtual-thread concurrency. Override with `HIKARI_MAXIMUM_POOL_SIZE`; keep it ≥ the benchmark VU count, bounded by Postgres `max_connections` (raised to 500 in `database/docker-compose.yml`).

**HikariCP minimum idle:** `HIKARI_MINIMUM_IDLE` (optional, no default) sets `minimumIdle`. When unset, HikariCP defaults `minimumIdle` to `maximumPoolSize`, so all 200 connections open at startup — that is the naive behavior. `Dockerfile.optimized` sets `HIKARI_MINIMUM_IDLE=25` so the optimized image starts light while the pool can still grow to `maximumPoolSize` under load.

## Code Structure

```
src/main/java/org/openapitools/server/
├── api/
│   ├── PetService.java / PetServiceImpl.java      # JAX-RS resource interface + impl
│   ├── StoreService.java / StoreServiceImpl.java
│   └── UserService.java / UserServiceImpl.java
├── db/
│   ├── DataSourceProvider.java                    # @ApplicationScoped HikariCP DataSource
│   ├── PetRepository.java                         # pet CRUD + inventory
│   ├── OrderRepository.java
│   └── UserRepository.java
└── model/                                         # generated DTOs (do not hand-edit)
```

## Conventions

- `*ServiceImpl` classes are thin JAX-RS resources annotated `@ApplicationScoped`; they `@Inject` a repository and delegate. Put endpoint logic here.
- Repositories are `@ApplicationScoped`, `@Inject` `DataSourceProvider`, and use plain JDBC (`Connection`/`PreparedStatement`). No ORM.
- Errors are signaled with `jakarta.ws.rs.WebApplicationException` carrying the right status (404 not found, 400 bad request); these propagate to the HTTP response.
- `category`, `photo_urls`, and `tags` are JSON columns written with Jackson `ObjectMapper` and `cast(? as json)`. `category` is stored as a JSON string.
- Enum columns (`pet.status` = `pet_status`, `order.status` = `order_status`) are written with `cast(? as pet_status)` and read as `status::text`, then mapped via the model enum `fromValue`.
- IDs default to an `AtomicLong` counter (seeded at startup) when omitted by the request.
- `uploadFile` reads the uploaded `File` body into a `byte[]` and persists it via `PetRepository.savePhoto` into the `pet_photo` table; the returned `ModelApiResponse` reports the stored byte count. `logoutUser` is a no-op.
- Tables used: `pet`, `"order"`, `"user"` (quoted reserved words).

## Generated vs. Hand-Written

- `model/` and the generator scaffolding are generated artifacts — avoid hand edits; they may be overwritten on regeneration.
- `api/*ServiceImpl.java` and everything under `db/` are the hand-written implementation. Preserve them if the project is regenerated.

## GraalVM Native Image

`Dockerfile.graalvm.broken` is kept for documentation — **the build fails and there is no current fix**.

**Reliability: FAILS.** Helidon MicroProfile 4 uses Weld CDI + Jersey JAX-RS. During GraalVM native-image analysis, Weld creates JDK dynamic proxy objects for CDI beans and annotation instances. GraalVM cannot persist these proxy objects in the image heap because they are typed as runtime-generated classes, but Weld's initialization triggers them at build time. This creates a cascade:

1. `com.fasterxml.jackson.core.Version` in `JacksonJsonProvider` → needs `--initialize-at-build-time`
2. Adding that reveals CDI proxies for `jakarta.validation.Valid` (and other annotations) → same issue
3. Each fix reveals the next layer of Weld/Jersey initialization conflicts

The fundamental problem: Weld CDI was not designed for GraalVM native compilation. Quarkus avoids this by using ArC (its own CDI implementation designed specifically for native image). Spring Boot avoids it via AOT-generated static bean factories. Helidon MP has no equivalent.

**Conclusion:** The `experimental` variant is not available for Helidon. Use `springboot experimental` or `quarkus experimental` instead — both produce sub-100 ms startup native binaries.

## CRaC — Checkpoint/Restore (`Dockerfile.crac`)

Appears as the **"crac"** variant in the frontend and CLI. Unlike Spring Boot (which has built-in `spring.context.checkpoint=onRefresh`), Helidon has no automatic checkpoint trigger, so `crac-checkpoint.sh` is used as the entrypoint: it starts the JVM, polls the health endpoint, then calls `jcmd JDK.checkpoint` once Helidon is ready.

**Base image:** `azul/zulu-openjdk:25-jdk-crac` (Azul Zulu JDK 25 with CRaC/CRIU support, Ubuntu/glibc). Full JDK required for `jcmd`. BellSoft only goes to JDK 21 for CRaC; Azul has JDK 25.

**Dependency:** `org.crac:crac` (`1.4.0`) is added to `pom.xml`. HikariCP 5.x has built-in CRaC support that activates when this library is on the classpath: `beforeCheckpoint` closes all pool connections, `afterRestore` re-creates them to the same DB. Helidon WebServer's socket lifecycle is also handled by the CRaC JDK runtime on restore.

**Two-phase build performed by `run.sh`:**

1. **Phase A — `docker build`**: builds the jar and produces the checkpoint-ready image with `ENTRYPOINT ["/app/crac-checkpoint.sh"]`. The script starts the JVM with `-XX:CRaCCheckpointTo=/checkpoint`, polls `http://localhost:8080/api/v3/pet/findByStatus?status=available` (60 s timeout), then calls `jcmd $JVM_PID JDK.checkpoint`. CRIU writes `/checkpoint` and the JVM exits. HikariCP connects to Postgres on the Docker network (default `initializationFailTimeout=1 ms`), so the DB **must be reachable** — `run.sh` adds `--network database_default` to the checkpoint container run.

2. **Phase B — `docker run` + `docker commit`**: `run.sh` runs with `--cap-add CHECKPOINT_RESTORE --cap-add SYS_PTRACE --security-opt seccomp=unconfined --network database_default`, waits for exit, commits with `ENTRYPOINT ["java", "-XX:CRaCRestoreFrom=/checkpoint"]`. Restore is unprivileged.

> **Compared to GraalVM:** The `experimental` variant is unavailable for Helidon (Weld CDI + Jersey proxy cascade breaks native-image). CRaC is Helidon's path to fast startup without a GraalVM build.

Benchmark usage:
```bash
cd performance-tests
VUS=3 DURATION=15s ./run.sh helidon crac
```

## Mutation Testing

[PIT](https://pitest.org) (pitest-maven 1.25.4) is configured in `pom.xml`.
It targets the hand-written `*ServiceImpl` and `db.*` classes.
Generated model/interface/scaffolding code is excluded. The unit tests use
plain Mockito (no CDI container), so PIT instruments them without issues.

```bash
# Run mutation analysis (produces target/pit-reports/index.html):
mvn test-compile org.pitest:pitest-maven:mutationCoverage

# Incremental re-run — only re-tests classes changed since the last run:
mvn test-compile org.pitest:pitest-maven:mutationCoverage -DwithHistory
```

A surviving mutant means no test distinguishes the mutated bytecode from the
original. Fix survivors by adding a sharper assertion or confirm they are
equivalent mutations.

Current mutation score: **91%** (166/183 killed, 17 no-coverage, 0 survivors, 100% test strength).
The 17 no-coverage mutations are in JDBC paths reachable only via integration tests
(e.g. `savePhoto` JDBC parameter binding, `getInventory` null-status branch).

## Docker

- `Dockerfile` (naive) and `Dockerfile.optimized` build on Temurin 25.
- The optimized image uses **G1GC** (`-XX:+UseG1GC`), not ZGC: with `--cpus 2 --memory 512m` the heap is only ~384MB at `MaxRAMPercentage=75`, where G1 is more memory-efficient and lower-overhead than ZGC. (`-XX:+ZGenerational` was also dropped — it is obsolete on Java 25, where generational ZGC is the default.) It also passes `-XX:InitialRAMPercentage=75.0` (heap pre-sized to the cap, avoiding resize pauses) and `-XX:+OptimizeStringConcat`, on both the training and runtime invocations.
- The optimized runtime passes `-Dmetrics.rest-request.enabled=false` to disable per-request MicroProfile metrics instrumentation during benchmarks; the naive image keeps it enabled via `microprofile-config.properties`.
- The optimized runtime sets `ENV HIKARI_MINIMUM_IDLE=25` (see HikariCP minimum idle above); the naive image leaves it unset.
- It also uses the **Project Leyden AOT cache** (JEP 483, JDK 24+). A dedicated `training` build stage runs the jar with `-XX:AOTMode=record -XX:AOTCacheOutput=app.aot`; HikariCP fails fast (default `initializationFailTimeout=1 ms`) due to no database in the build environment, causing a clean JVM exit whose shutdown hooks record the class-loading profile and assemble the cache. SIGTERM after 10 s handles any case where the app starts successfully. The runtime stage copies `app.aot` and passes `-XX:AOTCache=app.aot` on startup; if the cache file is invalid the JVM falls back gracefully.

## Verification

```bash
mvn package
java -jar target/petstore-helidon.jar &
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}'
curl -s http://localhost:8080/api/v3/store/inventory
```

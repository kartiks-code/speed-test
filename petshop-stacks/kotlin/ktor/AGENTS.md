# Kotlin Ktor Server — Agent Guide

OpenAPI-scaffolded Petstore server on Ktor 3.4 with PostgreSQL persistence via raw JDBC + HikariCP.

## Working Directory

Run all commands from `petshop-stacks/kotlin/ktor/` unless stated otherwise. The server listens on `:8080` and serves the API under `/api/v3`.

## Build, Run, Verify

```bash
./gradlew build          # compile + run tests
./gradlew test           # tests only (no DB needed)
./gradlew run            # run server on :8080
```

Requires JDK 21 and uses the bundled Gradle wrapper (`./gradlew`). No separate Gradle install needed.

## Database

Uses the shared PostgreSQL stack in `../../../database/`, database `kotlin-ktor`.

```bash
cd ../../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh
```

Connection defaults in `AppMain.kt` match `database/.env` (host `localhost`, port `5434`, user `myuser`, password `mypassword`, db `kotlin-ktor`). Override with `POSTGRES_*` env vars or a full `DATABASE_URL` (JDBC URL).

### Tuning env vars

HikariCP pool sizing in `PostgresPetstoreRepository.createDataSource()` is env-driven; defaults preserve baseline behavior:

| Env var | Default | Effect |
|---|---|---|
| `HIKARI_MAXIMUM_POOL_SIZE` | `10` | Hikari `maximumPoolSize` |
| `HIKARI_MINIMUM_IDLE` | unset | Hikari `minimumIdle`; only applied when set (unset keeps Hikari's default = max pool size) |

`Dockerfile.optimized` sets `HIKARI_MAXIMUM_POOL_SIZE=200 HIKARI_MINIMUM_IDLE=25` for the benchmark harness (200 handles up to 500 k6 VUs via queueing, under Postgres `max_connections=500`).

### Logging

`src/main/resources/logback.xml` root level is `WARN` (it was previously misconfigured at `trace`, which flooded stdout with millions of log lines under load and skewed benchmarks).

## Code Structure

```
src/main/kotlin/com/example/petstore/
├── AppMain.kt                          # Ktor entry point + module wiring (hand-written)
├── Configuration.kt                    # Placeholder (stub — compression/auth removed)
├── AllApis.kt                          # Aggregates all route functions (hand-written)
├── Paths.kt                            # Typed route resource classes (generated)
├── apis/
│   ├── PetApi.kt                       # Pet routes — hand-written (scaffold replaced)
│   ├── StoreApi.kt                     # Store routes — hand-written
│   └── UserApi.kt                      # User routes — hand-written
├── models/                             # Generated data classes (@Serializable) — do not hand-edit
│   ├── Pet.kt, Order.kt, User.kt, ...
└── repository/
    ├── PetstoreRepository.kt           # Interface — hand-written
    ├── PostgresPetstoreRepository.kt   # JDBC implementation — hand-written
    └── InMemoryPetstoreRepository.kt   # In-memory fake for tests — hand-written
src/test/kotlin/com/example/petstore/
├── PetApiTest.kt
├── StoreApiTest.kt
└── UserApiTest.kt
```

## Conventions

- **Blocking JDBC runs on `Dispatchers.IO`, not Netty event-loop threads.** All `PetstoreRepository` methods are `suspend` functions. `PostgresPetstoreRepository` wraps each method's JDBC body in `withContext(Dispatchers.IO)`, so every call site gets off-event-loop execution automatically and route handlers stay plain `repo.method()` calls. Composite operations (e.g. `addPet` upsert + re-read) use private blocking helpers (`fetchPetById`, `fetchOrderById`, `fetchUserByName`) so each method's JDBC work stays inside its single `withContext` block on one dispatcher thread. `Dispatchers.IO` defaults to 64 threads — intentionally untuned; with `HIKARI_MAXIMUM_POOL_SIZE=200`, excess demand queues at the dispatcher/pool. `InMemoryPetstoreRepository` implements the same suspend interface without `withContext` (no blocking I/O).
- Routes call `PetstoreRepository` methods; `PetApi`, `StoreApi`, `UserApi` are `Route` extension functions that accept the repository as a parameter.
- `configureModule(repo)` in `AppMain.kt` is the testable module setup; `main()` calls it with `PostgresPetstoreRepository`.
- `InMemoryPetstoreRepository` lives in main sources so PIT can mutate it; tests inject it via `testApplication { application { configureModule(InMemoryPetstoreRepository()) } }`.
- `NotFoundException` → 404, `InvalidInputException` → 400. These are in `PostgresPetstoreRepository.kt` (same file as the helper exceptions).
- `category`, `photo_urls`, `tags` are JSON columns; `category` is stored as a JSON text string serialized with `kotlinx.serialization`.
- Enum columns (`pet.status = pet_status`, `order.status = order_status`) are cast with `?::pet_status` / `?::order_status` on write, read as `status::text`.
- IDs default to `SELECT nextval('<table>_id_seq')` (sequences: `pet_id_seq`, `order_id_seq`, `user_id_seq`, `pet_photo_id_seq`); writes use `INSERT … ON CONFLICT … DO UPDATE` upserts.
- `uploadFile` reads the raw request body bytes and inserts into `pet_photo (content BYTEA)`.
- `logoutUser` is a stateless no-op returning 200.
- Routing uses Ktor Resources plugin (`@Resource`-annotated classes in `Paths.kt`). Import only `io.ktor.server.resources.{get,post,put,delete}` in route files — **do not** `import io.ktor.server.routing.*` there, as Ktor 3 adds a typed `post<T>` to routing that creates an ambiguity with Resources.

## Generated vs. Hand-Written

| File / directory | Status |
|---|---|
| `models/` | Generated — do not hand-edit |
| `Paths.kt` | Generated — do not hand-edit |
| `.openapi-generator*` | Generator artifacts |
| `apis/PetApi.kt`, `StoreApi.kt`, `UserApi.kt` | Hand-written (scaffold replaced) |
| `AppMain.kt`, `AllApis.kt` | Hand-written |
| `repository/` | Hand-written |
| `Configuration.kt`, `infrastructure/ApiKeyAuth.kt` | Replaced with stubs (auth/compression removed) |

## Mutation Testing

[PIT](https://pitest.org) via `gradle-pitest-plugin` 1.15.0.

Targets `com.example.petstore.apis.*` and `com.example.petstore.repository.InMemoryPetstoreRepository`. Generated models and infrastructure stubs are excluded. No live database required.

```bash
# Run mutation analysis (produces build/reports/pitest/index.html):
./gradlew pitest
```

**Current score: 85% test strength** (50/59 mutations killed, 9 survived — all equivalent).

### Exclusions

The pitest config deliberately excludes three categories of Kotlin noise mutations that no functional test can kill:

| Exclusion | Mechanism | Reason |
|---|---|---|
| `invokeSuspend` methods | `excludedMethods` | Kotlin coroutine state-machine generated code — 233 equivalent mutations |
| `PetApi`, `UserApi`, `StoreApi` methods | `excludedMethods` | Ktor route-registration wrappers — 38 equivalent mutations |
| `kotlin.jvm.internal.Intrinsics`, `kotlin.ResultKt` calls | `avoidCallsTo` | Defensive Kotlin null/exception checks — semantically unreachable in strongly-typed code |

### Remaining 9 equivalent mutations (will not be killed)

| Method | Why equivalent |
|---|---|
| `deletePet`, `deleteOrder`, `deleteUser`, `logoutUser`, `updateUser` — `replaced return value with null` | These return `Unit`; returning `null` is indistinguishable to all callers |
| `createUsersWithList` line 109 — `removed conditional` | Coroutine state-machine label check that is never exercised in synchronous `runBlocking` tests (inner `createUser` never actually suspends) |
| `findPetsByTags` line 137 ×2 — `removed conditional` | Coroutine state-machine header check in the inlined `filter`/`any` body |
| `getInventory` line 77 — `removed conditional` | A redundant second null check for `Pet.Status.value` which is always a non-null `String`; the `?: continue` branch is unreachable |

### Test files

| File | Tests | Coverage |
|---|---|---|
| `src/test/kotlin/com/example/petstore/InMemoryRepositoryTest.kt` | 48 direct repo unit tests (`runBlocking`) | All 19 operations including edge cases, null inputs, not-found throws, ID sequencing |
| `src/test/kotlin/com/example/petstore/PetApiTest.kt` | 25 Ktor integration tests | All Pet endpoints, validation, status/tag filtering |
| `src/test/kotlin/com/example/petstore/StoreApiTest.kt` | 12 Ktor integration tests | Inventory counting, order lifecycle |
| `src/test/kotlin/com/example/petstore/UserApiTest.kt` | 17 Ktor integration tests | User CRUD, login/logout, bulk create |

A surviving mutant means no test distinguishes the mutated bytecode from the original. Fix by adding sharper assertions.

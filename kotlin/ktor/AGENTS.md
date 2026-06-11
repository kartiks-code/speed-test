# Kotlin Ktor Server — Agent Guide

OpenAPI-scaffolded Petstore server on Ktor 3.4 with PostgreSQL persistence via raw JDBC + HikariCP.

## Working Directory

Run all commands from `kotlin/ktor/` unless stated otherwise. The server listens on `:8080` and serves the API under `/api/v3`.

## Build, Run, Verify

```bash
./gradlew build          # compile + run tests
./gradlew test           # tests only (no DB needed)
./gradlew run            # run server on :8080
```

Requires JDK 21 and uses the bundled Gradle wrapper (`./gradlew`). No separate Gradle install needed.

## Database

Uses the shared PostgreSQL stack in `../../database/`, database `kotlin-ktor`.

```bash
cd ../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh
```

Connection defaults in `AppMain.kt` match `database/.env` (host `localhost`, port `5434`, user `myuser`, password `mypassword`, db `kotlin-ktor`). Override with `POSTGRES_*` env vars or a full `DATABASE_URL` (JDBC URL).

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

- Routes call `PetstoreRepository` methods; `PetApi`, `StoreApi`, `UserApi` are `Route` extension functions that accept the repository as a parameter.
- `configureModule(repo)` in `AppMain.kt` is the testable module setup; `main()` calls it with `PostgresPetstoreRepository`.
- `InMemoryPetstoreRepository` lives in main sources so PIT can mutate it; tests inject it via `testApplication { application { configureModule(InMemoryPetstoreRepository()) } }`.
- `NotFoundException` → 404, `InvalidInputException` → 400. These are in `PostgresPetstoreRepository.kt` (same file as the helper exceptions).
- `category`, `photo_urls`, `tags` are JSON columns; `category` is stored as a JSON text string serialized with `kotlinx.serialization`.
- Enum columns (`pet.status = pet_status`, `order.status = order_status`) are cast with `?::pet_status` / `?::order_status` on write, read as `status::text`.
- IDs default to `COALESCE(MAX(id), 0) + 1`; writes use `INSERT … ON CONFLICT … DO UPDATE` upserts.
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

A surviving mutant means no test distinguishes the mutated bytecode from the original. Fix by adding sharper assertions.

# Speed Test — Root Agent Guide

This repository implements the **same OpenAPI Petstore API in multiple languages**, each backed by the **same shared PostgreSQL** instance. Use this file as the map; each project and the database have their own `AGENTS.md` with deeper, stack-specific rules — read the relevant one before editing that project.

## Repository Map

| Path | What it is | Agent guide |
|---|---|---|
| `spec/` | OpenAPI specs; `petstore-31.yaml` is the active contract | — |
| `COMMANDS.md` | OpenAPI Generator command for each target | — |
| `database/` | Shared PostgreSQL (Docker Compose), schema, create/apply scripts | `database/AGENTS.md` |
| `petshop-stacks/go/gin/` | Go + Gin server | `petshop-stacks/go/gin/AGENTS.md` |
| `petshop-stacks/java/springboot/` | Java 25 + Spring Boot 3.5 server | `petshop-stacks/java/springboot/AGENTS.md` |
| `petshop-stacks/java/helidon/` | Java 21 + Helidon MP 4 server | `petshop-stacks/java/helidon/AGENTS.md` |
| `petshop-stacks/java/quarkus/` | Java 25 + Quarkus 3.36.x server (Gradle) | `petshop-stacks/java/quarkus/AGENTS.md` |
| `petshop-stacks/nodejs/express/` | Node.js + Express server | `petshop-stacks/nodejs/express/AGENTS.md` |
| `petshop-stacks/python/fastapi/` | Python + FastAPI server | `petshop-stacks/python/fastapi/AGENTS.md` |
| `petshop-stacks/rust/hyper/` | Rust + hyper server | `petshop-stacks/rust/hyper/AGENTS.md` |
| `petshop-stacks/csharp/aspnetcore/` | C# + ASP.NET Core 8 server | `petshop-stacks/csharp/aspnetcore/AGENTS.md` |
| `petshop-stacks/php/laravel/` | PHP + Laravel server | `petshop-stacks/php/laravel/AGENTS.md` |
| `petshop-stacks/ruby/rails/` | Ruby + Rails server | `petshop-stacks/ruby/rails/AGENTS.md` |
| `petshop-stacks/kotlin/ktor/` | Kotlin + Ktor server | `petshop-stacks/kotlin/ktor/AGENTS.md` |
| `petshop-stacks/elixir/phoenix/` | Elixir + Phoenix server (hand-written) | `petshop-stacks/elixir/phoenix/AGENTS.md` |
| `performance-tests/` | Language-agnostic Docker benchmark harness; runs all 12 server stacks (`naive` + `optimized` variants), collects RPS/latency/CPU/RAM/Postgres stats; includes a results viewer + control server | `performance-tests/AGENTS.md` |

## The Core Workflow

1. The API contract is `spec/petstore-31.yaml`. Changing behavior usually starts there.
2. Servers are scaffolded with `openapi-generator-cli` (v7.23.0, pinned in `openapitools.json`) per `COMMANDS.md`.
3. Generated code is then filled in with real CRUD logic and PostgreSQL persistence.
4. Each server targets its **own database** inside the shared Postgres container.

**Regeneration caution:** re-running a generator can overwrite hand-written implementation code. Each project's `AGENTS.md` lists which files are generated vs. hand-written. Preserve the hand-written implementation (persistence layers, service/handler logic) when regenerating.

## Per-Project Commands

Always run commands from the project's own directory. The database must be up first (see below).

| Project | Build | Test | Run (port 8080) |
|---|---|---|---|
| `petshop-stacks/go/gin` | `go build ./...` | `go test ./...` | `go run main.go` |
| `petshop-stacks/java/springboot` | `mvn package` | `mvn test` | `mvn spring-boot:run` |
| `petshop-stacks/java/helidon` | `mvn package` | `mvn test` | `java -jar target/petstore-helidon.jar` |
| `petshop-stacks/java/quarkus` | `./gradlew build` | `./gradlew test` | `java -jar build/*-runner.jar` |
| `petshop-stacks/nodejs/express` | `npm install` | `npm test` | `npm start` |
| `petshop-stacks/python/fastapi` | `pip install -r requirements.txt` | `PYTHONPATH=src pytest tests` | `PYTHONPATH=src uvicorn petstore.main:app --port 8080` |
| `petshop-stacks/rust/hyper` | `cargo build` | `cargo test` | `cargo run --example petstore-server-server` |
| `petshop-stacks/csharp/aspnetcore` | `dotnet build` | `dotnet test` | `dotnet run --project src/Petstore` |
| `petshop-stacks/php/laravel` | `composer install` | `php artisan test` | `php artisan serve --port=8080` |
| `petshop-stacks/ruby/rails` | `bundle install` | `bundle exec rspec` | `bundle exec rails server -p 8080` |
| `petshop-stacks/kotlin/ktor` | `./gradlew build` | `./gradlew test` | `./gradlew run` |
| `petshop-stacks/elixir/phoenix` | `mix deps.get && mix compile` | `mix test` | `mix phx.server` |

## Mutation Testing

Every project (except Rust which has limited test coverage) has a mutation testing tool configured. All mutation tools use the same DB-free unit tests, so no PostgreSQL is required.

| Project | Tool | Command |
|---|---|---|
| `petshop-stacks/java/springboot` | [PIT](https://pitest.org) | `mvn test-compile org.pitest:pitest-maven:mutationCoverage` |
| `petshop-stacks/java/helidon` | [PIT](https://pitest.org) | `mvn test-compile org.pitest:pitest-maven:mutationCoverage` |
| `petshop-stacks/java/quarkus` | [PIT](https://pitest.org) | `./gradlew pitest` |
| `petshop-stacks/nodejs/express` | [Stryker](https://stryker-mutator.io) | `npm run mutate` (after `npm install`) |
| `petshop-stacks/python/fastapi` | [mutmut](https://mutmut.readthedocs.io) 2.x | `PYTHONPATH=src mutmut run` |
| `petshop-stacks/go/gin` | [gremlins](https://gremlins.dev) | `gremlins unleash ./go/...` |
| `petshop-stacks/rust/hyper` | [cargo-mutants](https://mutants.rs) | `cargo mutants` |
| `petshop-stacks/csharp/aspnetcore` | [Stryker.NET](https://stryker-mutator.io/docs/stryker-net/introduction/) | `dotnet stryker` |
| `petshop-stacks/php/laravel` | [Infection](https://infection.github.io) | `./vendor/bin/infection` |
| `petshop-stacks/ruby/rails` | [mutant](https://github.com/mbj/mutant) | `bundle exec mutant run` |
| `petshop-stacks/kotlin/ktor` | [PIT](https://pitest.org) | `./gradlew pitest` |
| `petshop-stacks/elixir/phoenix` | [muzak](https://github.com/devonestes/muzak) | `mix muzak` (best-effort; see AGENTS.md) |

Each project's `AGENTS.md` has the install step and a description of what is mutated vs. excluded.

### Notes for cross-project tasks (e.g. "build test cases for all projects")

- **Existing test coverage is uneven.** Go (`petshop-stacks/go/gin/go/*_test.go`), Helidon (`src/test/...`), Python (`petshop-stacks/python/fastapi/tests/`), and Spring Boot have some tests; **Node.js has no test runner wired** (only `start`/`prestart` scripts), and Rust tests are minimal. Treat adding tests as net-new work where missing.
- **Follow each language's idioms** rather than forcing one pattern: Go `testing`, JUnit 5 (Maven Surefire), `pytest`, Rust `#[test]`/`cargo test`, and Mocha+Chai for Node (a `test` script must be added to `petshop-stacks/nodejs/express/package.json`).
- **DB-dependent tests need a running database.** Prefer the per-project pattern: e.g. Go gates integration tests behind `TEST_DATABASE_DSN`; mirror that opt-in approach so unit tests stay runnable without Postgres.
- **Keep behavior identical across stacks** — the same request should produce the same result everywhere. When writing test cases, the shared conventions below are the contract to assert against.

## Shared Database

One PostgreSQL 17 container (in `database/`) hosts one database per server. Start it before running or testing any server:

```bash
cd database
docker compose up -d        # host port 5434
./create-databases.sh       # idempotent
./apply-schemas.sh
```

Defaults (from `database/.env`): host `localhost`, port `5434`, user `myuser`, password `mypassword`.

| Server | Database |
|---|---|
| Go | `go-gin-server` |
| Spring Boot | `java-springboot` |
| Helidon | `java-helidon` |
| Quarkus | `java-quarkus` |
| Node.js | `nodejs-express` |
| Python | `python-fastapi` |
| Rust | `rust-server` |
| C# ASP.NET Core | `csharp-aspnetcore` |
| PHP Laravel | `php-laravel` |
| Ruby Rails | `ruby-rails` |
| Kotlin Ktor | `kotlin-ktor` |
| Elixir Phoenix | `elixir-phoenix` |

> Connection **defaults differ per project**: Node.js, Python, Spring Boot, and Rust default to the shared `5434` / `myuser` / `mypassword`. Go and Helidon default to port `5432` and other credentials, so override their `POSTGRES_*` env vars (or pass a full DSN) when targeting the shared stack. See each project's `AGENTS.md`.

## Shared Persistence Conventions

All implementations must behave identically. When changing or testing any server, hold these invariants:

- All 19 Petstore operations are implemented in every server.
- `category`, `photo_urls`, `tags` are JSON columns; `category` is stored as a JSON string.
- `pet.status` (`pet_status`) and `order.status` (`order_status`) are PostgreSQL enum types; cast on write, read as `::text`.
- Server-assigned IDs use `SELECT nextval('<table>_id_seq')` (e.g. `pet_id_seq`, `order_id_seq`, `user_id_seq`, `pet_photo_id_seq`) when the request omits one; writes upsert via `INSERT … ON CONFLICT`. The sequences are defined in `database/postgresql_schema.sql`, are `OWNED BY` their column so `TRUNCATE … RESTART IDENTITY` resets them, and are seeded beyond existing rows on schema re-apply.
- Tables used: `pet`, `"order"`, `"user"` (the last two quoted because they are reserved words), and `pet_photo` (binary image storage).
- `uploadFile` verifies the pet exists, then persists the raw request body to the `pet_photo` table (`content` is a `BYTEA` column) keyed by `pet_id`; the response message reports the number of bytes stored. `logoutUser` is a stateless no-op.

## Conventions for Agents

- After making changes, update the relevant `AGENTS.md` and `README` for the project or subproject the change was made for, so docs stay in sync with the code.
- When asked to do something across all projects, parallelize the work across projects to the best extent possible (e.g. dispatch parallel agents per project) rather than working through them sequentially.
- Don't update git config, and don't commit unless explicitly asked.
- Keep `database/.env` and any credentials out of commits.
- Make the smallest change that satisfies the request; when touching a single project, prefer its local `AGENTS.md` rules over generic assumptions.
- After edits, run the relevant project's build/test from the table above to verify.

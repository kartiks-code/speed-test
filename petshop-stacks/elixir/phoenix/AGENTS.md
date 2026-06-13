# Elixir + Phoenix Petstore — Agent Guide

## Overview

This is a **hand-written** Elixir + Phoenix implementation of the Petstore API. There is no OpenAPI Generator target for Elixir/Phoenix; all files are authored manually.

## Project Layout

```
petshop-stacks/elixir/phoenix/
├── mix.exs                          # Project config + dependencies
├── config/
│   ├── config.exs                   # Base config (imports env-specific)
│   ├── dev.exs                      # Dev overrides
│   ├── test.exs                     # Test config (uses InMemoryRepository)
│   └── runtime.exs                  # Runtime env var overrides
├── lib/
│   ├── petstore/
│   │   ├── application.ex           # OTP Application supervisor
│   │   ├── repository.ex            # @behaviour with all 19 callbacks
│   │   ├── postgres_repository.ex   # Postgrex raw-SQL implementation
│   │   └── in_memory_repository.ex  # Agent-based test fake
│   └── petstore_web/
│       ├── endpoint.ex              # Phoenix Endpoint
│       ├── router.ex                # All 19 routes under /api/v3
│       ├── repo_helper.ex           # reads :repository config key
│       └── controllers/
│           ├── pet_controller.ex    # 8 pet operations
│           ├── store_controller.ex  # 4 store operations
│           ├── user_controller.ex   # 7 user operations
│           └── error_json.ex        # Error rendering
├── test/
│   ├── test_helper.exs
│   ├── support/
│   │   └── conn_case.ex                 # ConnCase helper for controller tests
│   └── petstore/
│       ├── pet_test.exs                 # Pet CRUD + edge cases + atom-key / snake-case variants
│       ├── store_test.exs               # Order CRUD + inventory + snake-case / atom-key variants
│       └── user_test.exs                # User CRUD + login/logout + snake-case / atom-key variants
│   └── petstore_web/
│       ├── error_json_test.exs          # ErrorJSON render clauses
│       └── controllers/
│           ├── pet_controller_test.exs  # HTTP tests for all 8 pet endpoints
│           ├── store_controller_test.exs # HTTP tests for all 4 store endpoints
│           └── user_controller_test.exs # HTTP tests for all 7 user endpoints
└── .env.example
```

## All Files Are Hand-Written

There is no generated code in this project. All files may be freely edited. When re-scaffolding or regenerating other projects, this directory is not affected.

## Key Conventions

### Repository Behaviour Pattern
- `Petstore.Repository` defines all 19 operation callbacks
- `Petstore.PostgresRepository` — production implementation using `Postgrex` (raw SQL, no Ecto)
- `Petstore.InMemoryRepository` — test fake using `Agent`; started via `start_supervised/1` in tests
- Active repository is configured via `config :petstore, :repository, <module>` (defaults to `PostgresRepository`; test config sets `InMemoryRepository`)

### Schema Invariants
- `category`, `photo_urls`, `tags` are JSON columns; `category` is stored as a JSON string
- `pet.status` / `order.status` are PostgreSQL enum types; cast with `$1::pet_status` / `$1::order_status` on write, read as `status::text`
- Server-assigned IDs use `SELECT nextval('<table>_id_seq')` (sequences: `pet_id_seq`, `order_id_seq`, `user_id_seq`, `pet_photo_id_seq`)
- Writes use `INSERT ... ON CONFLICT (id) DO UPDATE` (upsert)
- Tables: `pet`, `"order"` (quoted), `"user"` (quoted), `pet_photo` (BYTEA)
- `uploadFile`: verify pet exists → insert raw bytes into `pet_photo.content` (BYTEA) → return byte count
- `logoutUser`: stateless no-op, returns 200

### Database Connection
- Defaults: host `localhost`, port `5434`, db `elixir-phoenix`, user `myuser`, password `mypassword`
- All overrideable via env vars: `POSTGRES_HOST`, `POSTGRES_PORT`, `POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`
- Full URL override: `DATABASE_URL=postgres://...`
- `POOL_SIZE` sets the Postgrex pool size (default `10`, read in `config/runtime.exs`)

### Benchmark (optimized variant) Tuning

The optimized benchmark variant (`Dockerfile.optimized`, MIX_ENV=prod OTP release) bakes in:

- `ENV POOL_SIZE=200` — queues up to 500 k6 VUs on 200 DB connections (Postgres `max_connections=500`)
- `ENV ERL_FLAGS="+S 2:2 +sbwt none +sbwtdcpu none +sbwtdio none"` — `+S 2:2` matches the harness `--cpus 2` quota (the BEAM doesn't read cgroup CPU quotas); the `+sbwt*` flags disable scheduler busy-waiting, which wastes CPU in CPU-capped containers. The release boot script picks `ERL_FLAGS` up automatically.
- `config/prod.exs` sets `config :logger, level: :warning` to skip per-request info logging. This is prod-only, so the naive variant (dev config) is unaffected.

### Port
- Runs on port **8080** (set in `config/config.exs`)

## Commands

```bash
# Install dependencies
mix deps.get

# Compile
mix compile

# Run tests (no DB needed)
mix test

# Start server (port 8080)
PHX_SERVER=true mix phx.server

# Mutation testing (best-effort)
mix mutate
```

## Mutation Testing Notes

Mutation testing uses [muzak](https://github.com/devonestes/muzak) (`mix muzak`), aliased as `mix mutate`.

**Status: tool fails.** muzak 1.1.1 crashes with `MatchError` in `Muzak.Mutations.mutate_file/4` when attempting to tokenise files containing Phoenix-specific macros (`use PetstoreWeb, :router`, `use Application`, `++` operator in `application.ex`). The crash occurs before any mutations are generated, so no mutation score is available.

Confirmed failure log excerpt:
```
(MatchError) no match of right hand side value: {:ok, [...tokenized AST...]}
  (muzak 1.1.1) lib/muzak/mutations.ex:37: Muzak.Mutations.mutate_file/4
```

Because muzak is unavailable, test coverage via `mix test --cover` was maximised instead.

### Test Coverage (as of latest run)

**112 tests, all passing.** Run with `mix test --cover`.

| Module                      | Coverage |
|-----------------------------|----------|
| `Petstore.InMemoryRepository` | 99.09% |
| `Petstore.Repository` (behaviour) | 100% |
| `PetstoreWeb.PetController`  | 81.13% |
| `PetstoreWeb.StoreController` | 77.27% |
| `PetstoreWeb.UserController` | 75.76% |
| `PetstoreWeb.ErrorJSON`      | 100% |
| `PetstoreWeb.Router`         | 100% |
| `PetstoreWeb.RepoHelper`     | 100% |
| `PetstoreWeb.Endpoint`       | 100% |
| `Petstore.PostgresRepository` | 0% (DB-only, no Postgres in test) |
| `PetstoreWeb` (macro module) | 0% (compile-time only) |
| `Petstore.Application`       | 29.41% (DB branch untestable) |
| **Total**                    | **55.53%** |

The overall 55.53% is dragged down by `PostgresRepository` (raw SQL, requires a live DB) and `PetstoreWeb` (compile-time macro boilerplate). Excluding those, the testable business logic sits at ~90%+.

**Known coverage gaps in controllers (dead code):** Each controller has generic `{:error, reason} -> send_error(conn, 400, reason)` catch-all branches and a non-binary `send_error/3` overload that can only be triggered by errors InMemoryRepository never returns (it only yields `:not_found`). These branches are defensive programming against future repository implementations.

### Test setup fix

The original tests called `start_supervised(Repo)` in `setup`, which failed because `Petstore.Application` already starts `InMemoryRepository` as a named process before tests run. This was fixed by adding `InMemoryRepository.reset/0` (which resets Agent state to `initial_state/0`) and calling it in `setup` instead.

## Adding New Operations

1. Add callback to `lib/petstore/repository.ex`
2. Implement in `lib/petstore/postgres_repository.ex` (raw SQL) and `lib/petstore/in_memory_repository.ex`
3. Add route to `lib/petstore_web/router.ex`
4. Add action to the relevant controller
5. Add tests in `test/petstore/`

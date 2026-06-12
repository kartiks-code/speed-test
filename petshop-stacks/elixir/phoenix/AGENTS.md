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
│   └── petstore/
│       ├── pet_test.exs             # Pet CRUD + edge cases
│       ├── store_test.exs           # Order CRUD + inventory
│       └── user_test.exs            # User CRUD + login/logout
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

**Status: best-effort.** Elixir mutation testing tooling is less mature than the tools used by other projects in this repo (PIT for JVM, Stryker for JS/C#, mutmut for Python). `muzak` may not be available on all versions or may have limited operator support. If `mix mutate` fails to install or run:

1. Check if the package is available: `mix hex.info muzak`
2. If unavailable, document the failure in this file and skip mutation testing
3. The unit tests in `test/petstore/` still provide the primary quality signal

This mirrors the situation with `cargo-mutants` for the Rust server, which also has limited coverage documented in `petshop-stacks/rust/hyper/AGENTS.md`.

## Adding New Operations

1. Add callback to `lib/petstore/repository.ex`
2. Implement in `lib/petstore/postgres_repository.ex` (raw SQL) and `lib/petstore/in_memory_repository.ex`
3. Add route to `lib/petstore_web/router.ex`
4. Add action to the relevant controller
5. Add tests in `test/petstore/`

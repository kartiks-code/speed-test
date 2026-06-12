# Ruby on Rails Petstore — Agent Guide

## Working Directory

Run all commands from `petshop-stacks/ruby/rails/` unless stated otherwise.

## Quick Start

```bash
bundle install
bundle exec rails server -p 8080
```

API is served at `http://localhost:8080/api/v3`.

## Database

Shared PostgreSQL at `localhost:5434`. Database: `ruby-rails`.

### Connection environment variables

| Variable            | Default      |
|---------------------|--------------|
| `POSTGRES_HOST`     | `localhost`  |
| `POSTGRES_PORT`     | `5434`       |
| `POSTGRES_DB`       | `ruby-rails` |
| `POSTGRES_USER`     | `myuser`     |
| `POSTGRES_PASSWORD` | `mypassword` |

Set these or pass `DATABASE_URL` to override.

### Tuning environment variables

Defaults preserve single-mode, info-logging behavior; `Dockerfile.optimized` sets
the tuned values for benchmarks.

| Variable             | Default | `Dockerfile.optimized` | Effect |
|----------------------|---------|------------------------|--------|
| `WEB_CONCURRENCY`    | `0`     | `2`                    | Puma worker processes (`0` = single mode). `preload_app!` is disabled because the app runs in the development env where preload conflicts with the reloader. |
| `RAILS_MAX_THREADS`  | `5`     | `10`                   | Puma threads per worker. Threads hold thread-local PG connections, so max DB connections ≈ workers × threads. |
| `RAILS_LOG_LEVEL`    | `info`  | `warn`                 | Rails log level (set in `config/application.rb`, also honored by `config/environments/production.rb`). |
| `RUBY_YJIT_ENABLE`   | unset   | `1`                    | Enables YJIT. |
| `LD_PRELOAD`         | unset   | `/usr/lib/libjemalloc.so.2` | jemalloc allocator (installed via `apk add jemalloc` in the optimized image). |

## Build / Test / Run

| Command                           | What it does                       |
|-----------------------------------|------------------------------------|
| `bundle install`                  | Install dependencies               |
| `bundle exec rspec`               | Run DB-free unit tests             |
| `bundle exec mutant run`          | Mutation testing (InMemory repo)   |
| `bundle exec rails server -p 8080`| Start server on port 8080          |

## Code Layout

```
app/controllers/
  application_controller.rb   — repo accessor + parsed_body helper
  pet_controller.rb           — 8 pet operations (thin, delegates to repo)
  store_controller.rb         — 4 store operations
  user_controller.rb          — 7 user operations

config/
  routes.rb                   — All 19 /api/v3 routes; static paths before dynamic
  initializers/repository.rb  — Instantiates PETSTORE_REPOSITORY constant

lib/
  petstore_errors.rb               — NotFoundError, InvalidInputError (top-level)
  postgres_petstore_repository.rb  — HAND-WRITTEN: pg gem, raw SQL, all 19 ops
  in_memory_petstore_repository.rb — HAND-WRITTEN: hash-based, used in tests

spec/
  spec_helper.rb
  lib/in_memory_petstore_repository_spec.rb  — All repo operations, DB-free
```

## Generated vs. Hand-Written

| Path                                         | Status        |
|----------------------------------------------|---------------|
| `config/routes.rb`                           | Hand-written  |
| `app/controllers/*_controller.rb`            | Hand-written  |
| `lib/postgres_petstore_repository.rb`        | Hand-written  |
| `lib/in_memory_petstore_repository.rb`       | Hand-written  |
| `lib/petstore_errors.rb`                     | Hand-written  |
| `spec/**/*`                                  | Hand-written  |
| `app/models/*.rb` (generated stubs)          | Generated (unused; safe to ignore) |
| `config/initializers/{backtrace,cors,...}.rb`| Generated (kept) |

## Schema / Persistence Conventions

- No ActiveRecord. All DB access via `PostgresPetstoreRepository` using `pg` gem `exec_params`.
- `category` column is `TEXT` (stored as JSON string, e.g. `'{"id":1,"name":"cats"}'`).
- `photo_urls` / `tags` are `JSON` columns; read back with `JSON.parse`.
- `pet.status` / `order.status` cast on write: `cast($1 as pet_status)` / `cast($1 as order_status)`.
  Read with `status::text`.
- Server-assigned IDs: `SELECT nextval('<table>_id_seq')` (sequences: `pet_id_seq`, `order_id_seq`, `user_id_seq`, `pet_photo_id_seq`).
- Writes upsert: `INSERT ... ON CONFLICT (id) DO UPDATE SET ...`.
- `user` primary key is `username`, not `id`.
- `uploadFile`: verifies pet exists, inserts raw body bytes into `pet_photo.content` (BYTEA).
- `logoutUser`: stateless no-op, returns 200.

## Running Tests Without a Database

```bash
bundle exec rspec
# No POSTGRES_* variables needed — uses InMemoryPetstoreRepository
```

## Mutation Testing

```bash
bundle exec mutant run
```

Targets `InMemoryPetstoreRepository`. `.mutant.yml` configures subjects and integration.
Requires `gem 'mutant-rspec'` (in Gemfile dev/test group).

## Adding New Operations

1. Add route to `config/routes.rb`.
2. Add action to appropriate controller.
3. Add method to both `PostgresPetstoreRepository` and `InMemoryPetstoreRepository`.
4. Add spec to `spec/lib/in_memory_petstore_repository_spec.rb`.
5. Update this file and `README.md`.

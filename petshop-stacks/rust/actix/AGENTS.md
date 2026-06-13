# Rust Actix-web Petstore — Agent Guide

Hand-written Rust server implementing all 19 Petstore API operations using [Actix-web 4](https://actix.rs/) and [sqlx](https://github.com/launchbadge/sqlx) for async PostgreSQL access.

## Stack

| Item | Value |
|---|---|
| Language | Rust (edition 2021) |
| Framework | Actix-web 4 |
| DB driver | sqlx 0.8 (runtime-tokio-rustls, postgres, chrono) |
| Database | `rust-actix` (shared PostgreSQL container) |
| Base path | `/api/v3` |
| Listen port | `8080` (configurable via `BIND_ADDR`) |

## Generated vs Hand-Written

**All files are hand-written.** There is no OpenAPI Generator target for Actix-web; this server was written from scratch mirroring the SQL patterns from the sibling hyper stack.

| File | Status |
|---|---|
| `src/main.rs` | Hand-written — Actix-web app config and routing |
| `src/models.rs` | Hand-written — Pet, Order, User, Category, Tag, ApiResponse, enums |
| `src/helpers.rs` | Hand-written — pure row-to-model converters + unit tests |
| `src/db_config.rs` | Hand-written — DSN builder + pool config + unit tests |
| `src/handlers/mod.rs` | Hand-written — `next_id` helper |
| `src/handlers/pet.rs` | Hand-written — all 8 pet operations |
| `src/handlers/store.rs` | Hand-written — all 4 store operations |
| `src/handlers/user.rs` | Hand-written — all 7 user operations |
| `Dockerfile` | Hand-written — naive build |
| `Dockerfile.optimized` | Hand-written — cargo-chef layer-cached build |

## Commands

All commands run from `petshop-stacks/rust/actix/`.

| Action | Command |
|---|---|
| Build | `cargo build` |
| Build (release) | `cargo build --release` |
| Test (unit, no DB) | `cargo test` |
| Run | `cargo run` |
| Run (release) | `cargo run --release` |

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `BIND_ADDR` | `0.0.0.0:8080` | Listen address and port |
| `DATABASE_URL` | _(see below)_ | Full DSN — overrides all POSTGRES_* vars |
| `POSTGRES_HOST` / `PGHOST` | `localhost` | DB host |
| `POSTGRES_PORT` / `PGPORT` | `5434` | DB port (shared stack default) |
| `POSTGRES_USER` / `PGUSER` | `myuser` | DB user |
| `POSTGRES_PASSWORD` / `PGPASSWORD` | `mypassword` | DB password |
| `POSTGRES_DB` / `PGDATABASE` | `rust-actix` | DB name |
| `DB_POOL_MAX` | `10` (container default: `200`) | Max pool connections |

## Database

Uses the shared PostgreSQL container (`database/`). The database `rust-actix` must exist:

```bash
cd database
docker compose up -d
./create-databases.sh   # idempotent
./apply-schemas.sh
```

## API Routes

All routes are served under `/api/v3`:

| Method | Path | Handler |
|---|---|---|
| `POST` | `/pet` | `add_pet` |
| `PUT` | `/pet` | `update_pet` |
| `GET` | `/pet/findByStatus` | `find_pets_by_status` |
| `GET` | `/pet/findByTags` | `find_pets_by_tags` |
| `GET` | `/pet/{petId}` | `get_pet_by_id` |
| `POST` | `/pet/{petId}` | `update_pet_with_form` |
| `DELETE` | `/pet/{petId}` | `delete_pet` |
| `POST` | `/pet/{petId}/uploadImage` | `upload_file` |
| `GET` | `/store/inventory` | `get_inventory` |
| `POST` | `/store/order` | `place_order` |
| `GET` | `/store/order/{orderId}` | `get_order_by_id` |
| `DELETE` | `/store/order/{orderId}` | `delete_order` |
| `POST` | `/user` | `create_user` |
| `POST` | `/user/createWithList` | `create_users_with_list_input` |
| `GET` | `/user/login` | `login_user` |
| `GET` | `/user/logout` | `logout_user` |
| `GET` | `/user/{username}` | `get_user_by_name` |
| `PUT` | `/user/{username}` | `update_user` |
| `DELETE` | `/user/{username}` | `delete_user` |

## Mutation Testing

Mutation testing is scoped to the pure helper modules (no database required):

```bash
cargo install cargo-mutants   # one-time install
cargo mutants
```

The `.cargo/mutants.toml` examines `src/helpers.rs` and `src/db_config.rs`.

## Notable Implementation Details

- **Route ordering**: static paths (`/pet/findByStatus`, `/user/login`, etc.) are registered before parameterised paths (`/pet/{petId}`, `/user/{username}`) to prevent ambiguous matches.
- **`findByTags`**: query param `tags` is a repeated key (e.g. `?tags=a&tags=b`); actix-web's `web::Query` with `Vec<String>` handles this via `serde_urlencoded`.
- **`updatePetWithForm`**: accepts `application/x-www-form-urlencoded` body via `web::Form<UpdatePetForm>`.
- **`uploadFile`**: accepts raw bytes via `web::Bytes`; `additionalMetadata` is a query param.
- **`logoutUser`**: stateless no-op; always returns 200.
- **`deletePet`**: idempotent; always returns 200 regardless of rows affected.
- **`deleteOrder` / `deleteUser`**: check `rows_affected()` and return 404 if the row did not exist.

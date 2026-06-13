# C++ + Drogon Petstore — Agent Guide

## Stack overview

- **Language / Framework:** C++17 + [Drogon](https://github.com/drogonframework/drogon)
- **Database:** PostgreSQL 17, database name `cpp-drogon`
- **All code is hand-written** — there is no OpenAPI Generator target for this stack.

## Directory structure

```
src/
  main.cc             entry point; configures Drogon app + DB pool
  config.h / .cc      build_connection_string, pool_size (no Drogon dep)
  helpers.h / .cc     row_to_pet/order/user, JSON helpers (no Drogon dep)
  models.h            Pet, Order, User structs + nlohmann/json serialization
  db.h / .cc          getDb() — returns Drogon's shared DbClientPtr
  controllers/
    PetController.h / .cc
    StoreController.h / .cc
    UserController.h / .cc
test/
  CMakeLists.txt      doctest unit tests (no Drogon, no DB)
  test_helpers.cc     tests for helpers.cc
  test_config.cc      tests for config.cc
```

## Build

```bash
cd petshop-stacks/cpp/drogon
cmake -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

Requires Drogon and its dependencies to be installed (`drogonframework/drogon` Docker image
provides all of them). On a plain Ubuntu system you can install Drogon from source:

```bash
apt install -y cmake ninja-build g++ libssl-dev zlib1g-dev libjsoncpp-dev \
               libuuid-dev libpq-dev nlohmann-json3-dev
# Then build Drogon from https://github.com/drogonframework/drogon
```

## Test

```bash
cd build && ctest --output-on-failure
```

Tests are pure unit tests: no PostgreSQL required, no Drogon runtime needed.

## Run

```bash
./build/petstore-drogon         # port 8080
PORT=9090 ./build/petstore-drogon
```

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | _(unset)_ | Full libpq connection string; overrides all `POSTGRES_*` vars |
| `POSTGRES_HOST` | `localhost` | DB host |
| `POSTGRES_PORT` | `5434` | DB port (shared dev container runs on 5434) |
| `POSTGRES_USER` | `myuser` | DB user |
| `POSTGRES_PASSWORD` | `mypassword` | DB password |
| `POSTGRES_DB` | `cpp-drogon` | Database name |
| `DB_POOL_MAX` | `10` | Drogon async connection pool size |
| `PORT` | `8080` | HTTP listen port |

When running inside Docker (performance harness), the DB host becomes `speed-test-postgres` and
port `5432` — set via `ENV` in both Dockerfiles.

## Docker

```bash
# Naive build
docker build -t petstore-drogon .

# Optimized build (O2, non-root, DB_POOL_MAX=200)
docker build -f Dockerfile.optimized -t petstore-drogon-opt .
```

## Mutation testing (best-effort)

Mull (LLVM-based mutation testing) requires an LLVM bitcode build. It is not integrated into
the standard CMake build. To attempt it:

```bash
# Install mull-runner from https://github.com/mull-project/mull
cmake -B build-mull \
  -DCMAKE_CXX_COMPILER=clang++ \
  -DCMAKE_CXX_FLAGS="-flegacy-pass-manager -Xclang -load -Xclang LLVMMullPass.so" \
  -DCMAKE_BUILD_TYPE=RelWithDebInfo
cmake --build build-mull
mull-runner build-mull/petstore-tests
```

This may require additional LLVM/Mull setup beyond what the Dockerfile provides.
Mutation coverage of `helpers.cc` and `config.cc` is the primary target (no DB dep).

## Files preserved during regeneration

All files in this stack are hand-written. There is no generator to run; do **not** delete or
overwrite any source file with scaffolding.

## SQL invariants

- Enum casts: `NULLIF($n,'')::pet_status`, `NULLIF($n,'')::order_status`; read as `status::text`
- JSON columns cast on write: `$n::json`
- Sentinel-based NULLs for optional numerics: `NULLIF($n, 0)` for pet_id / quantity in orders
- IDs: `SELECT nextval('pet_id_seq')` / `order_id_seq` / `user_id_seq` / `pet_photo_id_seq`
- Upserts: `INSERT ... ON CONFLICT (id) DO UPDATE SET ...`
- Tables: `pet`, `"order"`, `"user"`, `pet_photo`

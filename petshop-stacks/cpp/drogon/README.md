# petstore-drogon — C++ + Drogon

An implementation of the [Petstore OpenAPI v3.1](../../spec/petstore-31.yaml) API in **C++17**
using the [Drogon](https://github.com/drogonframework/drogon) async HTTP framework and
PostgreSQL via Drogon's built-in ORM/DB client.

## Prerequisites

- CMake ≥ 3.16
- Ninja (or Make)
- C++17-capable compiler (GCC 10+ / Clang 12+)
- Drogon + its dependencies installed (see below)
- PostgreSQL 17 running (shared dev container on port 5434)

### Installing Drogon on Ubuntu

```bash
apt install -y cmake ninja-build g++ libssl-dev zlib1g-dev libjsoncpp-dev \
               libuuid-dev libpq-dev nlohmann-json3-dev git

git clone https://github.com/drogonframework/drogon
cd drogon && git submodule update --init
cmake -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build --parallel
sudo cmake --install build
```

Alternatively, use the `drogonframework/drogon` Docker image for building (the Dockerfiles
in this directory do exactly that).

## Database setup

```bash
cd ../../database
docker compose up -d
./create-databases.sh   # creates cpp-drogon among others
./apply-schemas.sh
```

## Build

```bash
cmake -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

## Test

Unit tests use [doctest](https://github.com/doctest/doctest) and require no DB:

```bash
cd build && ctest --output-on-failure
# or
./build/test/petstore-tests
```

## Run

```bash
./build/petstore-drogon     # listens on :8080
```

Override defaults with environment variables:

```
POSTGRES_HOST=localhost
POSTGRES_PORT=5434
POSTGRES_DB=cpp-drogon
POSTGRES_USER=myuser
POSTGRES_PASSWORD=mypassword
DB_POOL_MAX=10
PORT=8080
```

Or supply a full connection string:

```
DATABASE_URL="host=localhost port=5434 user=myuser password=mypassword dbname=cpp-drogon sslmode=disable"
```

## Docker

```bash
# Standard build
docker build -t petstore-drogon .

# Optimised (O2, non-root user, DB_POOL_MAX=200)
docker build -f Dockerfile.optimized -t petstore-drogon-opt .

# Run against shared DB container
docker run --network host \
  -e POSTGRES_HOST=localhost \
  -e POSTGRES_PORT=5434 \
  -p 8080:8080 \
  petstore-drogon
```

## API

All 19 Petstore operations are implemented at base path `/api/v3`:

| Method | Path | Operation |
|---|---|---|
| POST | `/api/v3/pet` | addPet |
| PUT | `/api/v3/pet` | updatePet |
| GET | `/api/v3/pet/findByStatus` | findPetsByStatus |
| GET | `/api/v3/pet/findByTags` | findPetsByTags |
| GET | `/api/v3/pet/{id}` | getPetById |
| POST | `/api/v3/pet/{id}` | updatePetWithForm |
| DELETE | `/api/v3/pet/{id}` | deletePet |
| POST | `/api/v3/pet/{id}/uploadImage` | uploadFile |
| GET | `/api/v3/store/inventory` | getInventory |
| POST | `/api/v3/store/order` | placeOrder |
| GET | `/api/v3/store/order/{id}` | getOrderById |
| DELETE | `/api/v3/store/order/{id}` | deleteOrder |
| POST | `/api/v3/user` | createUser |
| POST | `/api/v3/user/createWithList` | createUsersWithList |
| GET | `/api/v3/user/login` | loginUser |
| GET | `/api/v3/user/logout` | logoutUser (no-op) |
| GET | `/api/v3/user/{username}` | getUserByName |
| PUT | `/api/v3/user/{username}` | updateUser |
| DELETE | `/api/v3/user/{username}` | deleteUser |

## Architecture notes

- All I/O is async using Drogon's `execSqlAsync` callback pattern — no blocking on the event
  loop.
- `helpers.cc` and `config.cc` have **no Drogon dependency** so they can be unit-tested without
  linking the full framework.
- JSON serialization uses [nlohmann/json](https://github.com/nlohmann/json) with hand-written
  `to_json`/`from_json` for camelCase field mapping.
- Enum columns (`pet_status`, `order_status`) are written with `NULLIF($n,'')::pet_status` and
  read back as `status::text`.

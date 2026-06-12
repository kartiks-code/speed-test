# PHP + Laravel Petstore Server

Implements all 19 Petstore API operations using **PHP 8.3+ / Laravel 13** with a raw-SQL PDO/PostgreSQL persistence layer. Part of the [speed-test](../..) multi-language benchmark suite.

## Prerequisites

| Tool | Minimum version |
|------|----------------|
| PHP  | 8.3            |
| Composer | 2.x       |
| PostgreSQL | 17 (shared container at `localhost:5434`) |

> **Note:** All commands below can also be run via the Docker-based approach described at the bottom of this file if PHP is not installed locally.

## Quick Start

```bash
# 1. Start the shared database (from repo root)
cd ../../../database && docker compose up -d && ./create-databases.sh && ./apply-schemas.sh && cd ../../../petshop-stacks/php/laravel

# 2. Install dependencies
composer install

# 3. Configure environment
cp .env.example .env
php artisan key:generate

# 4. Run the server (port 8080)
php artisan serve --port=8080
```

## Environment Variables

| Variable       | Default        | Description              |
|----------------|----------------|--------------------------|
| `DB_HOST`      | `localhost`    | PostgreSQL host          |
| `DB_PORT`      | `5434`         | PostgreSQL port          |
| `DB_DATABASE`  | `php-laravel`  | Database name            |
| `DB_USERNAME`  | `myuser`       | Database user            |
| `DB_PASSWORD`  | `mypassword`   | Database password        |
| `APP_KEY`      | *(generated)*  | Laravel encryption key   |

## Commands

| Action             | Command                                              |
|--------------------|------------------------------------------------------|
| Install            | `composer install`                                   |
| Run tests          | `php artisan test` or `composer test`                |
| Run (port 8080)    | `php artisan serve --port=8080`                      |
| Mutation testing   | `./vendor/bin/infection` or `composer mutate`        |

## Running Tests (no database needed)

Tests use the `InMemoryPetstoreRepository` — no PostgreSQL required:

```bash
php artisan test
# or directly:
vendor/bin/phpunit --testsuite=Unit
```

## Mutation Testing

```bash
./vendor/bin/infection --threads=4
```

Targets `app/Repositories/InMemoryPetstoreRepository.php`. Results written to `infection.log`.

## Smoke Tests

With the server running at `http://localhost:8080`:

```bash
# Add a pet
curl -s -X POST http://localhost:8080/api/v3/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/photo.jpg"],"status":"available"}' | jq .

# Get inventory
curl -s http://localhost:8080/api/v3/store/inventory | jq .

# Create user
curl -s -X POST http://localhost:8080/api/v3/user \
  -H 'Content-Type: application/json' \
  -d '{"username":"johndoe","firstName":"John","lastName":"Doe","email":"john@example.com","password":"secret","phone":"555-1234","userStatus":1}' | jq .

# Login
curl -s "http://localhost:8080/api/v3/user/login?username=johndoe&password=secret"
```

## Using Docker (no local PHP)

```bash
# Run tests
docker run --rm -v "$(pwd):/app" -w /app php:8.4-cli php vendor/bin/phpunit --testsuite=Unit

# Run server (requires network access to postgres)
docker run --rm -p 8080:8080 --network host \
  -v "$(pwd):/app" -w /app \
  -e DB_HOST=localhost -e DB_PORT=5434 \
  php:8.4-cli php artisan serve --host=0.0.0.0 --port=8080
```

## API Base Path

All endpoints are under `/api/v3`. Example: `GET /api/v3/pet/{petId}`

## Architecture

```
app/
  Http/Controllers/
    PetController.php        # Pet API handlers
    StoreController.php      # Store/order API handlers
    UserController.php       # User API handlers
  Repositories/
    PetstoreRepositoryInterface.php    # All 19-operation contract
    InMemoryPetstoreRepository.php     # Array-backed fake (tests)
    PostgresPetstoreRepository.php     # PDO pgsql production impl
  Providers/
    AppServiceProvider.php             # Binds interface → Postgres impl
routes/
  api.php                             # All 19 routes under /api/v3
tests/
  Unit/
    PetRepositoryTest.php
    StoreRepositoryTest.php
    UserRepositoryTest.php
```

# Ruby on Rails Petstore Server

A Rails 8 API-only server implementing the [Swagger Petstore API](../spec/petstore-31.yaml) (19 operations).
Uses **raw SQL via the `pg` gem** — no ActiveRecord/ORM.

## Prerequisites

- Ruby ≥ 3.2 (4.0+ supported)
- Bundler (`gem install bundler`)
- PostgreSQL 17 shared container running (see `../database/`)

## Quick Start

```bash
# 1 — Start the database (one-time or after a restart)
cd ../../../database
docker compose up -d
./apply-schemas.sh

# 2 — Install dependencies
cd ../../../petshop-stacks/ruby/rails
bundle install

# 3 — Run the server (port 8080)
bundle exec rails server -p 8080
```

Base URL: `http://localhost:8080/api/v3`

## Environment Variables

| Variable            | Default       | Notes                              |
|---------------------|---------------|------------------------------------|
| `POSTGRES_HOST`     | `localhost`   |                                    |
| `POSTGRES_PORT`     | `5434`        | Matches shared container port      |
| `POSTGRES_DB`       | `ruby-rails`  |                                    |
| `POSTGRES_USER`     | `myuser`      |                                    |
| `POSTGRES_PASSWORD` | `mypassword`  |                                    |
| `PORT`              | `8080`        | Puma listen port                   |
| `RAILS_ENV`         | `development` |                                    |
| `DATABASE_URL`      | (unset)       | Overrides all POSTGRES_* if set    |

Copy `.env.example` to `.env` and adjust as needed.

## Running Tests

Tests use `InMemoryPetstoreRepository` — **no database required**.

```bash
bundle exec rspec
```

## Mutation Testing

```bash
bundle exec mutant run
```

Mutant targets `InMemoryPetstoreRepository`. See `.mutant.yml`.

## Project Structure

```
petshop-stacks/ruby/rails/
├── app/controllers/
│   ├── application_controller.rb   # repo accessor, parsed_body helper
│   ├── pet_controller.rb           # 8 pet operations
│   ├── store_controller.rb         # 4 store operations
│   └── user_controller.rb          # 7 user operations
├── config/
│   ├── routes.rb                   # All 19 API routes under /api/v3
│   └── initializers/repository.rb  # Instantiates PostgresPetstoreRepository
├── lib/
│   ├── petstore_errors.rb          # NotFoundError, InvalidInputError
│   ├── postgres_petstore_repository.rb  # pg gem + raw SQL
│   └── in_memory_petstore_repository.rb # In-memory (tests)
└── spec/
    └── lib/
        └── in_memory_petstore_repository_spec.rb
```

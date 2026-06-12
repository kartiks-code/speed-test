# Petstore — Elixir + Phoenix

Hand-written implementation of the OpenAPI Petstore API using Elixir 1.16+ and Phoenix 1.7+. No OpenAPI Generator is used; all code is authored by hand.

## Prerequisites

- Elixir 1.16+ / OTP 26+
- Mix (bundled with Elixir)
- PostgreSQL 17 (shared container in `../../database/`)

## Database Setup

```bash
cd ../../../database
docker compose up -d
./create-databases.sh
./apply-schemas.sh
```

This creates the `elixir-phoenix` database with the petstore schema.

## Configuration

Copy `.env.example` to `.env` and adjust as needed:

```bash
cp .env.example .env
```

Environment variables (all have defaults):

| Variable | Default | Description |
|---|---|---|
| `POSTGRES_HOST` | `localhost` | DB host |
| `POSTGRES_PORT` | `5434` | DB port |
| `POSTGRES_DB` | `elixir-phoenix` | Database name |
| `POSTGRES_USER` | `myuser` | DB user |
| `POSTGRES_PASSWORD` | `mypassword` | DB password |
| `DATABASE_URL` | _(unset)_ | Full Postgres URL (overrides individual vars) |
| `SECRET_KEY_BASE` | dev default | Phoenix secret key (change in production) |
| `PHX_SERVER` | _(unset)_ | Set to `true` to start server |

## Build

```bash
mix deps.get && mix compile
```

## Run Tests (no database required)

Tests use the in-memory repository:

```bash
mix test
```

## Run Server (port 8080)

```bash
PHX_SERVER=true mix phx.server
```

## API Endpoints

All routes are under `/api/v3`:

### Pet
| Method | Path | Operation |
|---|---|---|
| POST | `/pet` | Add a new pet |
| PUT | `/pet` | Update an existing pet |
| GET | `/pet/findByStatus?status=` | Find pets by status |
| GET | `/pet/findByTags?tags=` | Find pets by tags |
| GET | `/pet/:petId` | Get pet by ID |
| POST | `/pet/:petId` | Update pet with form data |
| DELETE | `/pet/:petId` | Delete a pet |
| POST | `/pet/:petId/uploadImage` | Upload pet image |

### Store
| Method | Path | Operation |
|---|---|---|
| GET | `/store/inventory` | Get pet inventory by status |
| POST | `/store/order` | Place an order |
| GET | `/store/order/:orderId` | Get order by ID |
| DELETE | `/store/order/:orderId` | Delete an order |

### User
| Method | Path | Operation |
|---|---|---|
| POST | `/user` | Create a user |
| POST | `/user/createWithList` | Create users from list |
| GET | `/user/login?username=&password=` | Log user in |
| GET | `/user/logout` | Log user out (no-op) |
| GET | `/user/:username` | Get user by username |
| PUT | `/user/:username` | Update user |
| DELETE | `/user/:username` | Delete user |

## Mutation Testing (best-effort)

```bash
mix mutate
```

> **Note:** Elixir mutation testing tooling (`muzak`) is still maturing. See `AGENTS.md` for details.

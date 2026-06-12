# Python FastAPI — Agent Guide

This directory contains the OpenAPI-generated FastAPI Petstore server with PostgreSQL persistence via `asyncpg`.

## Working Directory

Run all commands from `petshop-stacks/python/fastapi/` unless a command explicitly says otherwise.

## Project Layout

```
petshop-stacks/python/fastapi/
  src/petstore/
    apis/            # Generated route handlers + base classes (do not edit)
    models/          # Generated Pydantic models (do not edit)
    db.py            # Lazy asyncpg connection pool + JSON codec setup
    main.py          # FastAPI app factory (do not edit)
    petstore/impl/   # Implementation classes — edit these
      pet_api_impl.py
      store_api_impl.py
      user_api_impl.py
  requirements.txt
  .env.example
```

## Database

The server connects to the shared PostgreSQL instance from `../../../database/`. It targets the `python-fastapi` database.

Configure connection via environment variables (see `.env.example`):

```bash
cp .env.example .env
# edit .env with your actual credentials
```

Or export `DATABASE_URL` as a full DSN:

```bash
export DATABASE_URL=postgresql://myuser:mypassword@localhost:5434/python-fastapi
```

Default values when no env vars are set:
- host: `localhost`
- port: `5434`
- user: `myuser`
- password: `mypassword`
- database: `python-fastapi`

Ensure the database and schema are applied before starting the server:

```bash
cd ../../../database
docker compose up -d
./create-databases.sh
./apply-schemas.sh
```

## Common Commands

```bash
# Install dependencies
pip install -r requirements.txt

# Run the server
PYTHONPATH=src uvicorn petstore.main:app --host 0.0.0.0 --port 8080 --reload

# Run with .env loaded
set -a && source .env && set +a
PYTHONPATH=src uvicorn petstore.main:app --host 0.0.0.0 --port 8080 --reload

# Run via Docker (builds and starts the service)
docker compose up --build

# Run tests
PYTHONPATH=src pytest tests
```

Browse the interactive API docs at `http://localhost:8080/docs` after starting.

## Mutation Testing

[mutmut](https://mutmut.readthedocs.io) (2.x) mutates the hand-written
implementation in `src/petstore/petstore/impl/` and reruns the pytest suite
against each mutant. Configuration lives in the `[mutmut]` section of
`setup.cfg`. The unit tests use an in-memory fake database (`tests/conftest.py`),
so no PostgreSQL is needed.

```bash
pip install -r requirements.txt -r requirements-dev.txt
PYTHONPATH=src mutmut run        # mutate impl/, rerun pytest per mutant
mutmut results                   # list surviving mutants
mutmut show <id>                 # inspect a specific surviving mutant
mutmut html                      # write an HTML report to html/
```

`PYTHONPATH=src` is required so the spawned pytest runner can import the
`petstore` package from the `src/` layout. A surviving mutant means no test
distinguishes the mutated code from the original — either kill it with a
sharper assertion or confirm it is equivalent.

## Implementation Pattern

The generated routers in `apis/` dispatch to subclasses of `BasePetApi`, `BaseStoreApi`, and `BaseUserApi`. The `pkgutil` loader in each router file auto-discovers all modules under `petstore.petstore.impl` and registers any subclass. To add a new implementation or replace the existing one, create a class that inherits from the appropriate base class in that package.

The `db.py` module provides:
- `get_pool()` — returns a shared `asyncpg.Pool`, creating it lazily on first call
- JSON/JSONB codecs are registered so JSON-type columns round-trip as Python objects

## Database Schema Notes

- `pet.category` is a `TEXT` column storing a JSON object; read with `json.loads`.
- `pet.photo_urls` and `pet.tags` are `JSON` columns; decoded automatically by the asyncpg codec.
- `pet.status` and `order.status` are PostgreSQL enum types (`pet_status`, `order_status`); use `$n::pet_status` / `$n::order_status` when writing.
- `user.username` is the primary key for the `user` table.

## Verification

After changing implementation files, restart the server and exercise key endpoints:

```bash
# Add a pet
curl -s -X POST http://localhost:8080/pet \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fido","photoUrls":["http://example.com/fido.jpg"],"status":"available"}' | jq .

# Fetch it back
curl -s http://localhost:8080/pet/1 | jq .

# Inventory
curl -s http://localhost:8080/store/inventory | jq .
```

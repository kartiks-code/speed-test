# Go Fiber Petstore — Agent Guide

## Stack

- Language: Go (1.22+)
- Framework: [Fiber v2](https://github.com/gofiber/fiber) (`github.com/gofiber/fiber/v2`)
- Database driver: `github.com/jackc/pgx/v5/stdlib` (via `database/sql`)
- Database: PostgreSQL 17, database name `go-fiber`

All code in this stack is **hand-written** — there is no OpenAPI Generator target for Fiber. The implementation mirrors the Gin stack at `../gin/` but uses Fiber's context API.

## Directory layout

```
petshop-stacks/go/fiber/
├── go.mod / go.sum
├── main.go                    — entry point (hand-written)
├── go/                        — package petstore
│   ├── store.go               — Store interface + PostgresStore (all SQL)
│   ├── routers.go             — Fiber app factory, route registration
│   ├── api_helpers.go         — shared context helpers (bindJSON, queryList, …)
│   ├── api_pet.go             — PetAPI handlers
│   ├── api_store.go           — StoreAPI handlers
│   ├── api_user.go            — UserAPI handlers
│   ├── model_*.go             — model structs (9 files, copied from gin)
│   └── *_test.go              — unit tests (no DB required)
├── Dockerfile                 — standard build
├── Dockerfile.optimized       — scratch-based optimized build
├── .dockerignore
├── .gremlins.yaml             — mutation test thresholds
├── AGENTS.md                  — this file
└── README.md
```

## Commands

Run from the project root (`petshop-stacks/go/fiber/`):

| Action | Command |
|--------|---------|
| Build | `go build ./...` |
| Test (unit, no DB) | `go test ./...` |
| Run (port 8080) | `go run main.go` |
| Run (custom port) | `PORT=9090 go run main.go` |

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_HOST` | `localhost` | PostgreSQL host |
| `POSTGRES_PORT` | `5432` | PostgreSQL port |
| `POSTGRES_USER` | `postgres` | PostgreSQL user |
| `POSTGRES_PASSWORD` | *(required)* | PostgreSQL password |
| `POSTGRES_DB` | `go-fiber` | PostgreSQL database name |
| `POSTGRES_SSLMODE` | `disable` | SSL mode |
| `DATABASE_URL` | — | Full DSN (overrides individual vars) |
| `POSTGRES_DSN` | — | Full DSN (overrides individual vars, lower priority than `DATABASE_URL`) |
| `PORT` | `8080` | HTTP listen port |
| `FIBER_DISABLE_REQUEST_LOGGING` | `false` | Set `true` to suppress request logs |
| `GOMEMLIMIT` | — | Go runtime memory limit (e.g. `460MiB`) |
| `DB_MAX_OPEN_CONNS` | — | `database/sql` max open connections |
| `DB_MAX_IDLE_CONNS` | — | `database/sql` max idle connections |
| `DB_CONN_MAX_IDLE_TIME_SECONDS` | — | Max idle connection lifetime in seconds |

## PostgreSQL integration tests

Set `TEST_DATABASE_DSN` to run integration tests against a real database:

```bash
TEST_DATABASE_DSN="postgres://myuser:mypassword@localhost:5434/go-fiber?sslmode=disable" go test ./...
```

Unit tests (no DB required) always run. Integration tests are skipped when `TEST_DATABASE_DSN` is unset.

## Mutation testing

```bash
gremlins unleash --coverpkg github.com/GIT_USER_ID/GIT_REPO_ID/go --integration --timeout-coefficient 100 ./go
```

## Key implementation notes

### Fiber vs Gin context differences

- **Body parsing**: `c.BodyParser(&x)` (vs Gin's `c.ShouldBindJSON`)
- **Path params**: `c.Params("petId")` (vs Gin's `c.Param("petId")`)
- **Query params**: `c.Query("name")` for single; `queryList(c, "name")` for multi-value
- **JSON response**: `c.JSON(x)` (sets status 200 automatically)
- **Status + body**: `c.Status(404).JSON(x)`
- **String response**: `c.Status(200).SendString(s)`
- **Status only**: `c.SendStatus(200)`
- **Headers**: `c.Set("X-Header", "value")`
- **Raw body**: `c.Body()` returns `[]byte`
- **Request context**: `c.Context()` returns a `*fasthttp.RequestCtx`; use `c.UserContext()` for a `context.Context` compatible with `database/sql` — actually `c.Context()` has `Done()` etc. so handlers pass `c.UserContext()` (not available directly; they pass the adapter). In this implementation handlers pass `c.Context()` which wraps fasthttp context. For DB calls the standard `context.Context` is used via `fiber.Ctx`'s context adapter.

### Route ordering

Fiber matches routes in registration order. Static routes are registered before parameterized ones:
- `GET /pet/findByStatus` before `GET /pet/:petId`
- `GET /user/login`, `GET /user/logout` before `GET /user/:username`
- `POST /user/createWithList` before `POST /user`

### Generated vs hand-written

Everything is **hand-written**. There is no `.openapi-generator` directory and no generation command. Do not run OpenAPI Generator against this stack.

### model_*.go files

The 9 model files (`model_address.go`, `model_api_response.go`, `model_category.go`, `model_customer.go`, `model_error.go`, `model_order.go`, `model_pet.go`, `model_tag.go`, `model_user.go`) are identical to the Gin stack — they contain only struct definitions with JSON tags and have no framework dependency.

### store.go

`go/store.go` is framework-agnostic (pure `database/sql`). The only difference from the Gin stack is:
- Default `POSTGRES_DB` is `"go-fiber"` (not `"go-gin-server"`)

When updating SQL logic, keep `store.go` in sync between the Gin and Fiber stacks.

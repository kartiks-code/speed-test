# Go Gin PostgreSQL CRUD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the generated Go Gin Petstore stubs with PostgreSQL-backed CRUD behavior for the `go-gin-server` database.

**Architecture:** Add a focused database/repository layer in package `petstore`, inject it into the generated API structs, and keep route paths/models compatible with `api/openapi.yaml`. Use the existing generated PostgreSQL schema, hardened only where CRUD semantics require identity constraints.

**Tech Stack:** Go 1.19, Gin, `database/sql`, PostgreSQL via `github.com/jackc/pgx/v5/stdlib`, JSON columns for nested Pet fields.

---

### Task 1: PostgreSQL Store Contract

**Files:**
- Create: `go/go-gin-server/go/store.go`
- Test: `go/go-gin-server/go/store_test.go`
- Modify: `go/go-gin-server/go.mod`

- [ ] **Step 1: Write failing repository tests**

Create tests that open a PostgreSQL-backed store from a DSN, clean `pet`, `"order"`, and `"user"`, then verify create/get/update/delete for a pet, order, and user.

- [ ] **Step 2: Verify tests fail**

Run: `go test ./go -run 'TestPostgresStore'`
Expected: FAIL because `NewPostgresStore` and store methods do not exist.

- [ ] **Step 3: Implement the store**

Use `database/sql`, context-aware methods, JSON marshal/unmarshal for `Pet.Category`, `Pet.PhotoUrls`, and `Pet.Tags`, and SQL `INSERT ... ON CONFLICT` upserts for pet/order/user.

- [ ] **Step 4: Verify tests pass**

Run: `go test ./go -run 'TestPostgresStore'`
Expected: PASS when PostgreSQL is running and schema has been applied.

### Task 2: Handler Wiring

**Files:**
- Modify: `go/go-gin-server/main.go`
- Modify: `go/go-gin-server/go/api_pet.go`
- Modify: `go/go-gin-server/go/api_store.go`
- Modify: `go/go-gin-server/go/api_user.go`
- Test: `go/go-gin-server/go/api_handlers_test.go`

- [ ] **Step 1: Write failing handler tests**

Use Gin test recorders with a test store to verify representative endpoints: add/get/update/delete pet, place/get/delete order, create/get/update/delete user, inventory, login, logout, and upload response.

- [ ] **Step 2: Verify tests fail**

Run: `go test ./go -run 'Test.*API'`
Expected: FAIL because handlers still return placeholder `{"status":"OK"}`.

- [ ] **Step 3: Implement handlers**

Parse request bodies/path/query values, call the store methods, return OpenAPI-aligned JSON responses and status codes, and map missing rows to `404`.

- [ ] **Step 4: Verify tests pass**

Run: `go test ./go`
Expected: PASS.

### Task 3: Schema Constraints

**Files:**
- Modify: `database/postgresql_schema.sql`

- [ ] **Step 1: Add identity constraints safely**

Add idempotent unique indexes for `pet(id)`, `"order"(id)`, and `"user"(username)` so upserts and CRUD identity lookups are reliable without destructive drops.

- [ ] **Step 2: Verify SQL parses**

Run: `bash -n database/create-databases.sh database/apply-schemas.sh`
Expected: PASS.

### Task 4: Documentation and Verification

**Files:**
- Create or modify: `go/go-gin-server/README.md`
- Modify if useful: `go/go-gin-server/go/README.md`

- [ ] **Step 1: Document setup**

Document database startup from `database/`, schema application, server environment variables, and `go run main.go`.

- [ ] **Step 2: Run final checks**

Run: `go test ./...`
Run: `go test ./go` with PostgreSQL available if possible.
Run: `ReadLints` on edited Go files.


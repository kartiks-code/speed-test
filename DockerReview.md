# Docker Review

Review of all per-project Dockerfiles, June 10, 2026.

Every project ships two images:

- **`Dockerfile` (basic)** — a deliberately naive baseline: single stage, simple copy order, no layer-caching tricks, no runtime tuning. It must still build and run correctly.
- **`Dockerfile.optimized`** — multi-stage build, dependency-layer caching, minimal runtime image, non-root user, and security hygiene.

The review found three classes of problems, all fixed: files that were **broken** (would not build or run), basic files that were **contaminated with optimizations** (invalidating any basic-vs-optimized comparison), and bugs/rough edges in the optimized files. A per-project `.dockerignore` was also added everywhere (none existed before), keeping `.env*` files, build artifacts, and the Dockerfiles themselves out of every image.

## Broken files fixed

| Project | Problem | Fix |
|---|---|---|
| Go (both files) | `go.mod` requires Go 1.25 but images used `golang:1.19` / `golang:1.23-alpine` — could not compile | Bumped to `golang:1.25` (basic) and `golang:1.25-alpine` (optimized) |
| C# basic | Nonexistent base images (`mcr.microsoft.com/dotnet/core/{aspnet,sdk}:8.0-buster*` — the `dotnet/core` repos ended at 3.1) | Rewritten on `mcr.microsoft.com/dotnet/sdk:8.0` |
| C# (stray) | `src/Petstore/Dockerfile` — generator leftover, same nonexistent images, duplicated the root Dockerfile | Deleted |
| C# optimized | `DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=false` on Alpine, which ships without ICU → crash at startup | Added `apk add --no-cache icu-libs icu-data-full` |
| PHP optimized | Runtime stage ran `docker-php-ext-install` with no compiler or `postgresql-dev` → build failure | Runtime now copies the compiled extensions and `conf.d` from the deps stage |
| Ruby basic | `ruby:2.5-alpine` (EOL 2021) cannot run Rails 8.1; installed `sqlite-dev` while the app uses `pg` | Rewritten on `ruby:3.4-alpine` with `postgresql-dev` |
| Kotlin basic | `openjdk:8-jre-alpine` (deprecated, Java 8) cannot run the Java 21 bytecode produced by `jvmToolchain(21)`; only copied a host-built jar | Rewritten on `eclipse-temurin:21-jdk`, builds the fat jar inside Docker |
| Python basic | Final stage had no `CMD`/`EXPOSE` and never copied the app — container started a bare REPL; also ran pytest during the build | Rewritten as a runnable single stage (`python:3.12`, `pip install -r requirements.txt`, `PYTHONPATH=/app/src`, uvicorn CMD) |

## Optimizations removed from basic files

Baselines must be honest for the benchmark comparison to mean anything.

- **Go** — was multi-stage with a `scratch` runtime and stripped static binary. Now a single `golang:1.25` stage that builds and runs in place (`GIN_MODE=release` kept for behavior parity).
- **Quarkus** — had a Gradle dependency-cache warm-up layer before copying `src`. Now `COPY . .` then build, no caching tricks.
- **Kotlin** — shipped GC tuning (`-Xms4g -Xmx4g -XX:+UseG1GC -XX:MaxGCPauseMillis=100 -XX:+UseStringDeduplication`). Now a plain `java -jar`.
- **Python** — was 3 stages with a venv. Now one stage.
- **C#** — was 4 stages with cached `dotnet restore`. Now one SDK stage that publishes and runs.
- **Rust** — built a **debug** binary; "no Docker optimizations" should not mean "no compiler optimizations" for a benchmarked server. Now builds `--release` (still single-stage and naive), and `rust:latest` was pinned to `rust:1`.
- **Ruby** — Gemfile-first layer caching removed; the baseline copies everything then runs `bundle install`.

Already-clean baselines left untouched: Spring Boot, Helidon, Node.js, PHP, Elixir.

## Fixes to optimized files

- **Go** — removed hardcoded `GOOS=linux GOARCH=amd64` (broke arm64 hosts); added `USER 65534:65534` to the `scratch` runtime, which previously ran as root.
- **Quarkus** — runtime image changed from full `eclipse-temurin:25-jdk` to `eclipse-temurin:25-jre-alpine` (user creation switched to Alpine-style `adduser`); dropped no-op `-XX:+TieredCompilation`.
- **Spring Boot / Helidon / Ktor / Quarkus** — dropped `-XX:+UseContainerSupport` (default since JDK 10).
- **Node.js** — dropped `--max-old-space-size=256 --optimize-for-size`; they trade throughput for memory, which is backwards in a performance benchmark.
- **Python** — fixed layer-caching order: dependencies now install from `requirements.txt` *before* `src` is copied, so source edits no longer re-resolve all dependencies (`pip install --no-deps .` afterwards).
- **C#** — dropped `DOTNET_GCConserveMemory=9` (same throughput-vs-memory reasoning as Node).
- **PHP** — added `opcache.enable_cli=1`; the app runs via `php artisan serve` (a CLI process), so the previously configured OPcache was silently off. The opcache.ini is written after the `conf.d` copy so it isn't overwritten.
- **Ruby** — `ruby:4.0-alpine` replaced with `ruby:3.4-alpine` (safer, verified pin for Rails 8.1).
- **Rust** — pinned the third-party builder image to `lukemathwalker/cargo-chef:0.1.77-rust-1.95.0-bookworm` instead of a floating `latest-rust-1` tag (supply-chain hygiene; the tag also matches the `bookworm-slim` runtime, avoiding a glibc mismatch now that `latest-rust-1` defaults to trixie).
- **Python (builder gcc kept deliberately)** — `requirements.txt` pins `MarkupSafe==2.0.1` and `websockets==10.0`, which ship no Python 3.12 wheels and compile from source, so the builder genuinely needs a compiler. `python/docker-compose.yaml` also lost its `target: service` line, which pointed at a build stage that no longer exists.
- **Ruby (`docker-entrypoint.sh`)** — the entrypoint script was stale: it only matched the `s` server alias and removed a pid file at an obsolete `/myapp` path. It now matches `server` or `s` and cleans `tmp/pids/server.pid` relative to the workdir before running `db:migrate`.
- **Elixir** — replaced the inert `locales` package with `ENV LANG=C.UTF-8 LC_ALL=C.UTF-8` (Erlang needs a UTF-8 locale; C.UTF-8 is built into bookworm). Also changed `COPY mix.exs mix.lock ./` to `COPY mix.exs mix.lock* ./` — no `mix.lock` is checked in, so the strict COPY would have failed at build time (committing a lockfile would be the better long-term fix).

## Cross-cutting: `.dockerignore` added to all 12 projects

No project had one, so every `COPY . .` pulled in `.git`, `.env*` files, host build artifacts (`node_modules/`, `target/`, `_build/`, `build/`, `vendor/`, `bin/`/`obj/`, `__pycache__/`), markdown docs, and the Dockerfiles themselves. Each project now has a stack-appropriate `.dockerignore`; build inputs (sources, lockfiles, gradle wrapper, etc.) are deliberately not ignored.

## Security posture after review

- All optimized images run as a non-root user (including Go on `scratch`, via numeric `USER 65534:65534`).
- Runtime images contain no compilers or build toolchains (multi-stage everywhere).
- `.env*` files are excluded from build contexts in every project.
- Base images are pinned to version tags; the one third-party image (cargo-chef) is pinned to an exact version.

## Known remaining limitations (deliberate non-changes)

- **PHP** still serves through `php artisan serve` (single-threaded dev server). php-fpm + nginx, or Laravel Octane, would be the next real optimization, but changes the runtime architecture — out of scope for this pass. Laravel's `config:cache`/`route:cache` were also skipped since they bake env values at build time.
- **No `HEALTHCHECK`s** — cheap to add to the optimized set if/when orchestration needs them.
- Images are pinned to version tags, not digests.

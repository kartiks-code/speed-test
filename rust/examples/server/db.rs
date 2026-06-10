//! Database connection pool and helper utilities for the petstore example server.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

/// Build a DSN from environment variables, falling back to sibling-server defaults.
fn build_dsn() -> String {
    if let Ok(url) = env::var("DATABASE_URL") {
        return url;
    }

    let host = env::var("POSTGRES_HOST")
        .or_else(|_| env::var("PGHOST"))
        .unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("POSTGRES_PORT")
        .or_else(|_| env::var("PGPORT"))
        .unwrap_or_else(|_| "5434".to_string());
    let user = env::var("POSTGRES_USER")
        .or_else(|_| env::var("PGUSER"))
        .unwrap_or_else(|_| "myuser".to_string());
    let password = env::var("POSTGRES_PASSWORD")
        .or_else(|_| env::var("PGPASSWORD"))
        .unwrap_or_else(|_| "mypassword".to_string());
    let db = env::var("POSTGRES_DB")
        .or_else(|_| env::var("PGDATABASE"))
        .unwrap_or_else(|_| "rust-server".to_string());

    format!("postgresql://{user}:{password}@{host}:{port}/{db}")
}

/// Create and return a Postgres connection pool.
pub async fn create_pool() -> PgPool {
    // Load a .env file in the rust/ directory if present (optional, non-fatal).
    let _ = dotenvy::from_filename(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env"),
    );

    let dsn = build_dsn();
    PgPoolOptions::new()
        .min_connections(1)
        .max_connections(10)
        .connect(&dsn)
        .await
        .unwrap_or_else(|err| panic!("Failed to connect to PostgreSQL ({}): {}", dsn, err))
}

#[cfg(test)]
mod tests {
    use super::build_dsn;
    use std::env;
    use std::sync::Mutex;

    /// All environment variables `build_dsn` consults. Tests clear these before
    /// running so the host machine's environment cannot influence the result.
    const DSN_VARS: &[&str] = &[
        "DATABASE_URL",
        "POSTGRES_HOST",
        "PGHOST",
        "POSTGRES_PORT",
        "PGPORT",
        "POSTGRES_USER",
        "PGUSER",
        "POSTGRES_PASSWORD",
        "PGPASSWORD",
        "POSTGRES_DB",
        "PGDATABASE",
    ];

    /// `build_dsn` reads process-global environment variables, so the tests that
    /// exercise it must not run concurrently. This lock serializes them.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Run `f` with a clean environment in which only `set` variables are
    /// present, then restore the previous environment regardless of outcome.
    fn with_env<F: FnOnce()>(set: &[(&str, &str)], f: F) {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        let saved: Vec<(&str, Option<String>)> =
            DSN_VARS.iter().map(|&k| (k, env::var(k).ok())).collect();

        for &k in DSN_VARS {
            env::remove_var(k);
        }
        for (k, v) in set {
            env::set_var(k, v);
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

        for (k, v) in saved {
            match v {
                Some(val) => env::set_var(k, val),
                None => env::remove_var(k),
            }
        }

        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    #[test]
    fn defaults_match_shared_stack() {
        with_env(&[], || {
            assert_eq!(
                build_dsn(),
                "postgresql://myuser:mypassword@localhost:5434/rust-server"
            );
        });
    }

    #[test]
    fn database_url_overrides_everything() {
        with_env(
            &[
                ("DATABASE_URL", "postgresql://u:p@db.example.com:6000/custom"),
                ("POSTGRES_HOST", "ignored-host"),
                ("POSTGRES_DB", "ignored-db"),
            ],
            || {
                assert_eq!(build_dsn(), "postgresql://u:p@db.example.com:6000/custom");
            },
        );
    }

    #[test]
    fn assembles_dsn_from_postgres_vars() {
        with_env(
            &[
                ("POSTGRES_HOST", "pghost"),
                ("POSTGRES_PORT", "1234"),
                ("POSTGRES_USER", "alice"),
                ("POSTGRES_PASSWORD", "secret"),
                ("POSTGRES_DB", "petsdb"),
            ],
            || {
                assert_eq!(build_dsn(), "postgresql://alice:secret@pghost:1234/petsdb");
            },
        );
    }

    #[test]
    fn postgres_vars_take_precedence_over_pg_vars() {
        with_env(
            &[
                ("POSTGRES_HOST", "primary"),
                ("PGHOST", "fallback"),
                ("POSTGRES_USER", "primary-user"),
                ("PGUSER", "fallback-user"),
            ],
            || {
                let dsn = build_dsn();
                assert!(dsn.contains("@primary:"), "expected primary host in {}", dsn);
                assert!(
                    dsn.starts_with("postgresql://primary-user:"),
                    "expected primary user in {}",
                    dsn
                );
            },
        );
    }

    #[test]
    fn pg_vars_used_when_postgres_vars_absent() {
        with_env(
            &[
                ("PGHOST", "pg-only-host"),
                ("PGPORT", "7777"),
                ("PGUSER", "pg-user"),
                ("PGPASSWORD", "pg-pass"),
                ("PGDATABASE", "pg-db"),
            ],
            || {
                assert_eq!(
                    build_dsn(),
                    "postgresql://pg-user:pg-pass@pg-only-host:7777/pg-db"
                );
            },
        );
    }

    #[test]
    fn partial_overrides_fall_back_to_defaults() {
        with_env(&[("POSTGRES_DB", "only-db")], || {
            assert_eq!(
                build_dsn(),
                "postgresql://myuser:mypassword@localhost:5434/only-db"
            );
        });
    }
}

//! Database connection configuration helpers.
//!
//! Pure functions that derive a PostgreSQL DSN and pool parameters from
//! environment variables.  They are database-free and fully unit-testable.

use std::env;

/// Build a PostgreSQL DSN from environment variables.
///
/// Priority:
/// 1. `DATABASE_URL` — used verbatim if set.
/// 2. `POSTGRES_*` variables (canonical names).
/// 3. `PG*` variables (short-form aliases).
/// 4. Hard-coded defaults matching `database/.env`.
pub fn build_dsn() -> String {
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

/// Maximum connection-pool size from `DB_POOL_MAX`.
///
/// Returns 10 when the variable is unset, empty, non-numeric, or zero.
pub fn pool_max_connections() -> u32 {
    env::var("DB_POOL_MAX")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(10)
}

// ---------------------------------------------------------------------------
// Unit tests — no database required
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{build_dsn, pool_max_connections};
    use std::env;
    use std::sync::Mutex;

    const DSN_VARS: &[&str] = &[
        "DB_POOL_MAX",
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

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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

    #[test]
    fn partial_host_override() {
        with_env(&[("POSTGRES_HOST", "custom-host")], || {
            let dsn = build_dsn();
            assert!(dsn.contains("@custom-host:5434/"), "got: {}", dsn);
        });
    }

    #[test]
    fn partial_port_override() {
        with_env(&[("POSTGRES_PORT", "9999")], || {
            let dsn = build_dsn();
            assert!(dsn.contains(":9999/"), "got: {}", dsn);
        });
    }

    #[test]
    fn partial_user_override() {
        with_env(&[("POSTGRES_USER", "custom-user")], || {
            let dsn = build_dsn();
            assert!(dsn.starts_with("postgresql://custom-user:"), "got: {}", dsn);
        });
    }

    #[test]
    fn partial_password_override() {
        with_env(&[("POSTGRES_PASSWORD", "s3cr3t")], || {
            let dsn = build_dsn();
            assert!(dsn.contains("://myuser:s3cr3t@"), "got: {}", dsn);
        });
    }

    #[test]
    fn pghost_fallback_used_when_postgres_host_absent() {
        with_env(&[("PGHOST", "pg-fallback-host")], || {
            let dsn = build_dsn();
            assert!(dsn.contains("@pg-fallback-host:"), "got: {}", dsn);
        });
    }

    #[test]
    fn pgport_fallback_used_when_postgres_port_absent() {
        with_env(&[("PGPORT", "4444")], || {
            let dsn = build_dsn();
            assert!(dsn.contains(":4444/"), "got: {}", dsn);
        });
    }

    #[test]
    fn pguser_fallback_used_when_postgres_user_absent() {
        with_env(&[("PGUSER", "pg-fallback-user")], || {
            let dsn = build_dsn();
            assert!(dsn.starts_with("postgresql://pg-fallback-user:"), "got: {}", dsn);
        });
    }

    #[test]
    fn pgpassword_fallback_used_when_postgres_password_absent() {
        with_env(&[("PGPASSWORD", "pg-pass-fallback")], || {
            let dsn = build_dsn();
            assert!(dsn.contains("://myuser:pg-pass-fallback@"), "got: {}", dsn);
        });
    }

    #[test]
    fn pgdatabase_fallback_used_when_postgres_db_absent() {
        with_env(&[("PGDATABASE", "pg-db-fallback")], || {
            let dsn = build_dsn();
            assert!(dsn.ends_with("/pg-db-fallback"), "got: {}", dsn);
        });
    }

    #[test]
    fn pool_max_defaults_to_ten() {
        with_env(&[], || {
            assert_eq!(pool_max_connections(), 10);
        });
    }

    #[test]
    fn pool_max_reads_env() {
        with_env(&[("DB_POOL_MAX", "200")], || {
            assert_eq!(pool_max_connections(), 200);
        });
    }

    #[test]
    fn pool_max_falls_back_on_zero() {
        with_env(&[("DB_POOL_MAX", "0")], || {
            assert_eq!(pool_max_connections(), 10);
        });
    }

    #[test]
    fn pool_max_falls_back_on_non_numeric() {
        with_env(&[("DB_POOL_MAX", "not-a-number")], || {
            assert_eq!(pool_max_connections(), 10);
        });
    }

    #[test]
    fn pool_max_accepts_one() {
        with_env(&[("DB_POOL_MAX", "1")], || {
            assert_eq!(pool_max_connections(), 1);
        });
    }

    #[test]
    fn pool_max_accepts_large_value() {
        with_env(&[("DB_POOL_MAX", "1000")], || {
            assert_eq!(pool_max_connections(), 1000);
        });
    }
}

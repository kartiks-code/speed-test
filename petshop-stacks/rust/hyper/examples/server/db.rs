//! Database connection pool factory for the petstore example server.
//!
//! Pure DSN-building and pool-sizing helpers live in `petstore_server::db_config`
//! so they can be unit-tested and mutation-tested without a live database.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Create and return a Postgres connection pool.
pub async fn create_pool() -> PgPool {
    // Load a .env file in the rust/ directory if present (optional, non-fatal).
    let _ = dotenvy::from_filename(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env"),
    );

    let dsn = petstore_server::db_config::build_dsn();
    PgPoolOptions::new()
        .min_connections(1)
        .max_connections(petstore_server::db_config::pool_max_connections())
        .connect(&dsn)
        .await
        .unwrap_or_else(|err| panic!("Failed to connect to PostgreSQL ({}): {}", dsn, err))
}

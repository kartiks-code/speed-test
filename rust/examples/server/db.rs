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
        .unwrap_or_else(|e| panic!("Failed to connect to PostgreSQL ({dsn}): {e}"))
}

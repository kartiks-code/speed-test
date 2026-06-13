pub mod pet;
pub mod store;
pub mod user;

use sqlx::PgPool;
use sqlx::Row;

/// Get the next id from a Postgres sequence.
pub async fn next_id(pool: &PgPool, sequence: &str) -> Result<i64, String> {
    let row = sqlx::query(&format!(r#"SELECT nextval('{sequence}') AS next_id"#))
        .fetch_one(pool)
        .await
        .map_err(|e| format!("DB error getting next id: {e}"))?;
    let id: i64 = row
        .try_get("next_id")
        .map_err(|e| format!("DB error reading next id: {e}"))?;
    Ok(id)
}

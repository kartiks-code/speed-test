use actix_web::{web, HttpResponse};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

use crate::helpers::row_to_order;
use crate::models::Order;

use super::next_id;

pub async fn get_inventory(pool: web::Data<PgPool>) -> HttpResponse {
    let rows = sqlx::query(
        r#"SELECT status::text, CAST(COUNT(*) AS int) AS cnt
           FROM pet WHERE status IS NOT NULL GROUP BY status"#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let mut inventory: HashMap<String, i32> = HashMap::new();
            for row in &rows {
                let status: String = row.get("status");
                let cnt: i32 = row.get("cnt");
                inventory.insert(status, cnt);
            }
            HttpResponse::Ok().json(inventory)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error getting inventory: {e}"))
        }
    }
}

pub async fn place_order(pool: web::Data<PgPool>, body: web::Json<Order>) -> HttpResponse {
    let order = body.into_inner();

    let id = match order.id {
        Some(id) => id,
        None => match next_id(&pool, "order_id_seq").await {
            Ok(id) => id,
            Err(e) => return HttpResponse::InternalServerError().body(e),
        },
    };

    let status_str = order.status.as_ref().map(|s| s.to_string());
    let ship_date_naive = order.ship_date.map(|dt| dt.naive_utc());

    let result = sqlx::query(
        r#"
        INSERT INTO "order" (id, pet_id, quantity, ship_date, status, complete)
        VALUES ($1, $2, $3, $4, $5::order_status, $6)
        ON CONFLICT (id) DO UPDATE
            SET pet_id    = EXCLUDED.pet_id,
                quantity  = EXCLUDED.quantity,
                ship_date = EXCLUDED.ship_date,
                status    = EXCLUDED.status,
                complete  = EXCLUDED.complete
        "#,
    )
    .bind(id)
    .bind(order.pet_id)
    .bind(order.quantity)
    .bind(ship_date_naive)
    .bind(&status_str)
    .bind(order.complete)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            let saved = Order { id: Some(id), ..order };
            HttpResponse::Ok().json(saved)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error placing order: {e}"))
        }
    }
}

pub async fn delete_order(pool: web::Data<PgPool>, path: web::Path<i64>) -> HttpResponse {
    let order_id = path.into_inner();

    let result = sqlx::query(r#"DELETE FROM "order" WHERE id = $1"#)
        .bind(order_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::NotFound().body("Order not found"),
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error deleting order: {e}"))
        }
    }
}

pub async fn get_order_by_id(pool: web::Data<PgPool>, path: web::Path<i64>) -> HttpResponse {
    let order_id = path.into_inner();

    let row = sqlx::query(
        r#"SELECT id, pet_id, quantity, ship_date, status::text, complete
           FROM "order" WHERE id = $1"#,
    )
    .bind(order_id)
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(None) => HttpResponse::NotFound().body("Order not found"),
        Ok(Some(row)) => {
            let order = row_to_order(
                row.get("id"),
                row.get("pet_id"),
                row.get("quantity"),
                row.get("ship_date"),
                row.get("status"),
                row.get("complete"),
            );
            HttpResponse::Ok().json(order)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error fetching order: {e}"))
        }
    }
}

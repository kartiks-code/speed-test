use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::{PgPool, Row};

use crate::helpers::row_to_user;
use crate::models::User;

use super::next_id;

#[derive(Deserialize)]
pub struct LoginQuery {
    pub username: Option<String>,
    pub password: Option<String>,
}

pub async fn create_user(pool: web::Data<PgPool>, body: web::Json<User>) -> HttpResponse {
    let user = body.into_inner();

    let id = match user.id {
        Some(id) => id,
        None => match next_id(&pool, "user_id_seq").await {
            Ok(id) => id,
            Err(e) => return HttpResponse::InternalServerError().body(e),
        },
    };

    let result = sqlx::query(
        r#"
        INSERT INTO "user" (id, username, first_name, last_name, email, password, phone, user_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (username) DO UPDATE
            SET id          = EXCLUDED.id,
                first_name  = EXCLUDED.first_name,
                last_name   = EXCLUDED.last_name,
                email       = EXCLUDED.email,
                password    = EXCLUDED.password,
                phone       = EXCLUDED.phone,
                user_status = EXCLUDED.user_status
        "#,
    )
    .bind(id)
    .bind(&user.username)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.email)
    .bind(&user.password)
    .bind(&user.phone)
    .bind(user.user_status)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(User { id: Some(id), ..user }),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error creating user: {e}"))
        }
    }
}

pub async fn create_users_with_list_input(
    pool: web::Data<PgPool>,
    body: web::Json<Vec<User>>,
) -> HttpResponse {
    let users = body.into_inner();
    if users.is_empty() {
        return HttpResponse::BadRequest().body("No users provided");
    }

    let mut last_user: Option<User> = None;

    for u in users {
        let result = sqlx::query(
            r#"
            INSERT INTO "user" (id, username, first_name, last_name, email, password, phone, user_status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (username) DO UPDATE
                SET id          = EXCLUDED.id,
                    first_name  = EXCLUDED.first_name,
                    last_name   = EXCLUDED.last_name,
                    email       = EXCLUDED.email,
                    password    = EXCLUDED.password,
                    phone       = EXCLUDED.phone,
                    user_status = EXCLUDED.user_status
            "#,
        )
        .bind(u.id)
        .bind(&u.username)
        .bind(&u.first_name)
        .bind(&u.last_name)
        .bind(&u.email)
        .bind(&u.password)
        .bind(&u.phone)
        .bind(u.user_status)
        .execute(pool.get_ref())
        .await;

        match result {
            Ok(_) => last_user = Some(u),
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("DB error creating user in list: {e}"))
            }
        }
    }

    HttpResponse::Ok().json(last_user.unwrap())
}

pub async fn login_user(
    pool: web::Data<PgPool>,
    query: web::Query<LoginQuery>,
) -> HttpResponse {
    let username = match query.username.as_deref() {
        None => return HttpResponse::BadRequest().body("Username is required"),
        Some(u) => u,
    };
    let password = query.password.as_deref().unwrap_or("");

    let row = sqlx::query(r#"SELECT password FROM "user" WHERE username = $1"#)
        .bind(username)
        .fetch_optional(pool.get_ref())
        .await;

    match row {
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error during login: {e}"))
        }
        Ok(row) => {
            let stored_pw: Option<String> = row.as_ref().and_then(|r| r.get("password"));
            if stored_pw.as_deref() != Some(password) {
                return HttpResponse::Unauthorized().body("Invalid username or password");
            }
            let token = format!("logged-in-{username}");
            HttpResponse::Ok().body(token)
        }
    }
}

pub async fn logout_user() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn delete_user(pool: web::Data<PgPool>, path: web::Path<String>) -> HttpResponse {
    let username = path.into_inner();

    let result = sqlx::query(r#"DELETE FROM "user" WHERE username = $1"#)
        .bind(&username)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::NotFound().body("User not found"),
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error deleting user: {e}"))
        }
    }
}

pub async fn get_user_by_name(pool: web::Data<PgPool>, path: web::Path<String>) -> HttpResponse {
    let username = path.into_inner();

    let row = sqlx::query(
        r#"SELECT id, username, first_name, last_name, email, password, phone, user_status
           FROM "user" WHERE username = $1"#,
    )
    .bind(&username)
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Ok(Some(row)) => {
            let user = row_to_user(
                row.get("id"),
                row.get("username"),
                row.get("first_name"),
                row.get("last_name"),
                row.get("email"),
                row.get("password"),
                row.get("phone"),
                row.get("user_status"),
            );
            HttpResponse::Ok().json(user)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error fetching user: {e}"))
        }
    }
}

pub async fn update_user(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    body: web::Json<User>,
) -> HttpResponse {
    let username = path.into_inner();
    let user = body.into_inner();

    let result = sqlx::query(
        r#"
        UPDATE "user"
        SET id          = $2,
            first_name  = $3,
            last_name   = $4,
            email       = $5,
            password    = $6,
            phone       = $7,
            user_status = $8
        WHERE username = $1
        "#,
    )
    .bind(&username)
    .bind(user.id)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.email)
    .bind(&user.password)
    .bind(&user.phone)
    .bind(user.user_status)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::NotFound().body("User not found"),
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error updating user: {e}"))
        }
    }
}

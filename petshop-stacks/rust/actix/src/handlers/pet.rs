use actix_web::{web, HttpResponse};
use bytes::Bytes;
use serde::Deserialize;
use sqlx::{PgPool, Row};

use crate::helpers::{row_to_pet};
use crate::models::{ApiResponse, Pet};

use super::next_id;

#[derive(Deserialize)]
pub struct StatusQuery {
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct TagsQuery {
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdatePetForm {
    pub name: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct UploadQuery {
    #[serde(rename = "additionalMetadata")]
    pub additional_metadata: Option<String>,
}

pub async fn add_pet(pool: web::Data<PgPool>, body: web::Json<Pet>) -> HttpResponse {
    let pet = body.into_inner();

    if pet.name.is_empty() || pet.photo_urls.is_empty() {
        return HttpResponse::BadRequest().body("Name and photoUrls must not be empty");
    }

    let id = match pet.id {
        Some(id) => id,
        None => match next_id(&pool, "pet_id_seq").await {
            Ok(id) => id,
            Err(e) => return HttpResponse::InternalServerError().body(e),
        },
    };

    let category_json = pet
        .category
        .as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default());
    let photo_urls_json = match serde_json::to_string(&pet.photo_urls) {
        Ok(j) => j,
        Err(e) => return HttpResponse::InternalServerError().body(format!("JSON error: {e}")),
    };
    let tags_json = pet
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default());
    let status_str = pet.status.as_ref().map(|s| s.to_string());

    let result = sqlx::query(
        r#"
        INSERT INTO pet (id, name, category, photo_urls, tags, status)
        VALUES ($1, $2, $3, $4::json, $5::json, $6::pet_status)
        ON CONFLICT (id) DO UPDATE
            SET name       = EXCLUDED.name,
                category   = EXCLUDED.category,
                photo_urls = EXCLUDED.photo_urls,
                tags       = EXCLUDED.tags,
                status     = EXCLUDED.status
        "#,
    )
    .bind(id)
    .bind(&pet.name)
    .bind(&category_json)
    .bind(&photo_urls_json)
    .bind(&tags_json)
    .bind(&status_str)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            let saved = Pet { id: Some(id), ..pet };
            HttpResponse::Ok().json(saved)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("DB error inserting pet: {e}")),
    }
}

pub async fn find_pets_by_status(
    pool: web::Data<PgPool>,
    query: web::Query<StatusQuery>,
) -> HttpResponse {
    let status_str = query
        .status
        .clone()
        .unwrap_or_else(|| "available".to_string());

    let rows = sqlx::query(
        r#"SELECT id, name, category, photo_urls::text, tags::text, status::text
           FROM pet WHERE status = $1::pet_status"#,
    )
    .bind(&status_str)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let pets: Vec<Pet> = rows
                .iter()
                .map(|row| {
                    row_to_pet(
                        row.get("id"),
                        row.get("name"),
                        row.get("category"),
                        row.get::<String, _>("photo_urls"),
                        row.get("tags"),
                        row.get("status"),
                    )
                })
                .collect();
            HttpResponse::Ok().json(pets)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error finding pets by status: {e}"))
        }
    }
}

pub async fn find_pets_by_tags(
    pool: web::Data<PgPool>,
    query: web::Query<TagsQuery>,
) -> HttpResponse {
    let tag_list = &query.tags;
    if tag_list.is_empty() {
        return HttpResponse::Ok().json(Vec::<Pet>::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT id, name, category, photo_urls::text, tags::text, status::text
        FROM pet
        WHERE tags IS NOT NULL
          AND EXISTS (
              SELECT 1
              FROM json_array_elements(tags) AS t
              WHERE t->>'name' = ANY($1)
          )
        "#,
    )
    .bind(tag_list.as_slice())
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let pets: Vec<Pet> = rows
                .iter()
                .map(|row| {
                    row_to_pet(
                        row.get("id"),
                        row.get("name"),
                        row.get("category"),
                        row.get::<String, _>("photo_urls"),
                        row.get("tags"),
                        row.get("status"),
                    )
                })
                .collect();
            HttpResponse::Ok().json(pets)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error finding pets by tags: {e}"))
        }
    }
}

pub async fn update_pet(pool: web::Data<PgPool>, body: web::Json<Pet>) -> HttpResponse {
    let pet = body.into_inner();

    let id = match pet.id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("Pet ID is required for update"),
    };

    let category_json = pet
        .category
        .as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default());
    let photo_urls_json = match serde_json::to_string(&pet.photo_urls) {
        Ok(j) => j,
        Err(e) => return HttpResponse::InternalServerError().body(format!("JSON error: {e}")),
    };
    let tags_json = pet
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default());
    let status_str = pet.status.as_ref().map(|s| s.to_string());

    let result = sqlx::query(
        r#"
        UPDATE pet
        SET name       = $2,
            category   = $3,
            photo_urls = $4::json,
            tags       = $5::json,
            status     = $6::pet_status
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(&pet.name)
    .bind(&category_json)
    .bind(&photo_urls_json)
    .bind(&tags_json)
    .bind(&status_str)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::NotFound().body("Pet not found"),
        Ok(_) => HttpResponse::Ok().json(Pet { id: Some(id), ..pet }),
        Err(e) => HttpResponse::InternalServerError().body(format!("DB error updating pet: {e}")),
    }
}

pub async fn delete_pet(pool: web::Data<PgPool>, path: web::Path<i64>) -> HttpResponse {
    let pet_id = path.into_inner();

    let result = sqlx::query(r#"DELETE FROM pet WHERE id = $1"#)
        .bind(pet_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("DB error deleting pet: {e}")),
    }
}

pub async fn get_pet_by_id(pool: web::Data<PgPool>, path: web::Path<i64>) -> HttpResponse {
    let pet_id = path.into_inner();

    let row = sqlx::query(
        r#"SELECT id, name, category, photo_urls::text, tags::text, status::text
           FROM pet WHERE id = $1"#,
    )
    .bind(pet_id)
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(None) => HttpResponse::NotFound().body("Pet not found"),
        Ok(Some(row)) => {
            let pet = row_to_pet(
                row.get("id"),
                row.get("name"),
                row.get("category"),
                row.get::<String, _>("photo_urls"),
                row.get("tags"),
                row.get("status"),
            );
            HttpResponse::Ok().json(pet)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("DB error fetching pet: {e}")),
    }
}

pub async fn update_pet_with_form(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
    form: web::Form<UpdatePetForm>,
) -> HttpResponse {
    let pet_id = path.into_inner();
    let name = form.name.clone();
    let status_enum: Option<String> = form.status.as_deref().and_then(|s| {
        s.parse::<crate::models::PetStatus>()
            .ok()
            .map(|ps| ps.to_string())
    });

    let result = sqlx::query(
        r#"
        UPDATE pet
        SET name   = COALESCE($2, name),
            status = COALESCE($3::pet_status, status)
        WHERE id = $1
        "#,
    )
    .bind(pet_id)
    .bind(&name)
    .bind(&status_enum)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::BadRequest().body("Pet not found"),
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error updating pet with form: {e}"))
        }
    }
}

pub async fn upload_file(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
    query: web::Query<UploadQuery>,
    body: Bytes,
) -> HttpResponse {
    let pet_id = path.into_inner();

    let exists: Result<bool, _> =
        sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM pet WHERE id = $1)"#)
            .bind(pet_id)
            .fetch_one(pool.get_ref())
            .await;

    match exists {
        Ok(false) => return HttpResponse::NotFound().body("Pet not found"),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("DB error checking pet: {e}"))
        }
        Ok(true) => {}
    }

    let content: Vec<u8> = body.to_vec();
    let size = content.len();
    let additional_metadata = query.additional_metadata.clone();

    let result = sqlx::query(
        r#"INSERT INTO pet_photo (id, pet_id, content_type, metadata, content)
           VALUES (nextval('pet_photo_id_seq'), $1, $2, $3, $4)"#,
    )
    .bind(pet_id)
    .bind("application/octet-stream")
    .bind(additional_metadata.as_deref())
    .bind(&content)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            let message = match &additional_metadata {
                Some(meta) => format!("File uploaded for pet {pet_id}, {size} bytes ({meta})"),
                None => format!("File uploaded for pet {pet_id}, {size} bytes"),
            };
            HttpResponse::Ok().json(ApiResponse {
                code: Some(200),
                r#type: Some("application/octet-stream".to_string()),
                message: Some(message),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("DB error saving photo: {e}"))
        }
    }
}

//! Main library entry point for petstore_server implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper::service::{service_fn, Service};
use log::info;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::task::{Context, Poll};
use swagger::{Has, XSpanIdString};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use tokio::net::TcpListener;
use sqlx::PgPool;
use serde_json;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use petstore_server::models;

pub async fn create(addr: &str, https: bool) {
    let addr: SocketAddr = addr.parse().expect("Failed to parse bind address");
    let listener = TcpListener::bind(&addr).await.unwrap();

    let pool = crate::db::create_pool().await;
    let server = Server::new(pool);

    let service = MakeService::new(server);
    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    #[allow(unused_mut)]
    let mut service =
        petstore_server::server::context::MakeAddContext::<_, EmptyContext>::new(
            service
        );

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).expect("Failed to create SSL Acceptor");

            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM).expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem").expect("Failed to set certificate chain");
            ssl.check_private_key().expect("Failed to check private key");

            let tls_acceptor = ssl.build();

            info!("Starting a server (with https)");
            loop {
                if let Ok((tcp, addr)) = listener.accept().await {
                    let ssl = Ssl::new(tls_acceptor.context()).unwrap();
                    let service = service.call(addr);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::SslStream::new(ssl, tcp).map_err(|_| ())?;
                        let service = service.await.map_err(|_| ())?;

                        http1::Builder::new()
                            .serve_connection(TokioIo::new(tls), service)
                            .await
                            .map_err(|_| ())
                    });
                }
            }
        }
    } else {
        info!("Starting a server (over http, so no TLS)");
        println!("Listening on http://{}", addr);

        loop {
            let (tcp_stream, addr) = listener.accept().await.expect("Failed to accept connection");

            let service = service.call(addr).await.unwrap();
            let io = TokioIo::new(tcp_stream);
            tokio::task::spawn(async move {
                let result = http1::Builder::new()
                    .serve_connection(io, service)
                    .await;
                if let Err(err) = result {
                    println!("Error serving connection: {err:?}");
                }
            });
        }
    }
}

pub struct Server<C> {
    pool: PgPool,
    marker: std::marker::PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new(pool: PgPool) -> Self {
        Server { pool, marker: std::marker::PhantomData }
    }
}

impl<C> Clone for Server<C> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            marker: std::marker::PhantomData,
        }
    }
}

use jsonwebtoken::{decode, encode, errors::Error as JwtError, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use swagger::auth::Authorization;
use crate::server_auth;

use petstore_server::{
    Api,
    AddPetResponse,
    FindPetsByStatusResponse,
    FindPetsByTagsResponse,
    UpdatePetResponse,
    DeletePetResponse,
    GetPetByIdResponse,
    UpdatePetWithFormResponse,
    UploadFileResponse,
    GetInventoryResponse,
    PlaceOrderResponse,
    DeleteOrderResponse,
    GetOrderByIdResponse,
    CreateUserResponse,
    CreateUsersWithListInputResponse,
    LoginUserResponse,
    LogoutUserResponse,
    DeleteUserResponse,
    GetUserByNameResponse,
    UpdateUserResponse,
};
use petstore_server::server::MakeService;
use std::error::Error;
use swagger::ApiError;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a database row's optional TEXT/JSON column values into a models::Pet.
fn row_to_pet(
    id: Option<i64>,
    name: String,
    category_text: Option<String>,
    photo_urls_text: String,
    tags_text: Option<String>,
    status_text: Option<String>,
) -> models::Pet {
    let category = category_text.as_deref().and_then(|s| serde_json::from_str(s).ok());
    let photo_urls: Vec<String> = serde_json::from_str(&photo_urls_text).unwrap_or_default();
    let tags: Option<Vec<models::Tag>> = tags_text
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());
    let status = status_text
        .as_deref()
        .and_then(|s| s.parse::<models::PetStatus>().ok());

    models::Pet {
        id,
        name,
        category,
        photo_urls,
        tags,
        status,
    }
}

/// Convert a database row into a models::Order.
fn row_to_order(
    id: Option<i64>,
    pet_id: Option<i64>,
    quantity: Option<i32>,
    ship_date_naive: Option<chrono::NaiveDateTime>,
    status_text: Option<String>,
    complete: Option<bool>,
) -> models::Order {
    let ship_date = ship_date_naive
        .map(|nd| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(nd, chrono::Utc));
    let status = status_text
        .as_deref()
        .and_then(|s| s.parse::<models::OrderStatus>().ok());
    models::Order { id, pet_id, quantity, ship_date, status, complete }
}

/// Convert a database row into a models::User.
fn row_to_user(
    id: Option<i64>,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    phone: Option<String>,
    user_status: Option<i32>,
) -> models::User {
    models::User { id, username, first_name, last_name, email, password, phone, user_status }
}

/// Get the next available id for a table by doing MAX(id)+1.
async fn next_id(pool: &PgPool, table: &str) -> Result<i64, ApiError> {
    let row = sqlx::query(&format!(r#"SELECT COALESCE(MAX(id), 0) + 1 AS next_id FROM {table}"#))
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError(format!("DB error getting next id: {e}")))?;
    let id: i64 = row.try_get("next_id")
        .map_err(|e| ApiError(format!("DB error reading next id: {e}")))?;
    Ok(id)
}

use sqlx::Row;

// ---------------------------------------------------------------------------
// Api implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl<C> Api<C> for Server<C> where C: Has<XSpanIdString> + Send + Sync
{
    // -----------------------------------------------------------------------
    // Pet operations
    // -----------------------------------------------------------------------

    /// Add a new pet to the store.
    async fn add_pet(
        &self,
        pet: models::Pet,
        context: &C) -> Result<AddPetResponse, ApiError>
    {
        info!("add_pet({:?}) - X-Span-ID: {:?}", pet, context.get().0.clone());

        if pet.name.is_empty() || pet.photo_urls.is_empty() {
            return Ok(AddPetResponse::InvalidInput);
        }

        let id = match pet.id {
            Some(id) => id,
            None => next_id(&self.pool, "pet").await?,
        };

        let category_json = pet.category.as_ref().map(|c| serde_json::to_string(c).unwrap_or_default());
        let photo_urls_json = serde_json::to_string(&pet.photo_urls)
            .map_err(|e| ApiError(format!("JSON error: {e}")))?;
        let tags_json = pet.tags.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default());
        let status_str = pet.status.as_ref().map(|s| s.to_string());

        sqlx::query(
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
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error inserting pet: {e}")))?;

        let saved = models::Pet { id: Some(id), ..pet };
        Ok(AddPetResponse::SuccessfulOperation(saved))
    }

    /// Finds Pets by status.
    async fn find_pets_by_status(
        &self,
        status: Option<models::FindPetsByStatusStatusParameter>,
        context: &C) -> Result<FindPetsByStatusResponse, ApiError>
    {
        info!("find_pets_by_status({:?}) - X-Span-ID: {:?}", status, context.get().0.clone());

        let status_str = status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "available".to_string());

        let rows = sqlx::query(
            r#"SELECT id, name, category, photo_urls::text, tags::text, status::text
               FROM pet WHERE status = $1::pet_status"#,
        )
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error finding pets by status: {e}")))?;

        let pets: Vec<models::Pet> = rows
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

        Ok(FindPetsByStatusResponse::SuccessfulOperation(pets))
    }

    /// Finds Pets by tags.
    async fn find_pets_by_tags<'a>(
        &self,
        tags: Option<&'a Vec<String>>,
        context: &C) -> Result<FindPetsByTagsResponse, ApiError>
    {
        info!("find_pets_by_tags({:?}) - X-Span-ID: {:?}", tags, context.get().0.clone());

        let tag_list = match tags {
            None => return Ok(FindPetsByTagsResponse::SuccessfulOperation(vec![])),
            Some(t) if t.is_empty() => return Ok(FindPetsByTagsResponse::SuccessfulOperation(vec![])),
            Some(t) => t,
        };

        // Use json_array_elements to check if any tag name matches any of the requested tags.
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
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error finding pets by tags: {e}")))?;

        let pets: Vec<models::Pet> = rows
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

        Ok(FindPetsByTagsResponse::SuccessfulOperation(pets))
    }

    /// Update an existing pet.
    async fn update_pet(
        &self,
        pet: models::Pet,
        context: &C) -> Result<UpdatePetResponse, ApiError>
    {
        info!("update_pet({:?}) - X-Span-ID: {:?}", pet, context.get().0.clone());

        let id = match pet.id {
            Some(id) => id,
            None => return Ok(UpdatePetResponse::InvalidIDSupplied),
        };

        let category_json = pet.category.as_ref().map(|c| serde_json::to_string(c).unwrap_or_default());
        let photo_urls_json = serde_json::to_string(&pet.photo_urls)
            .map_err(|e| ApiError(format!("JSON error: {e}")))?;
        let tags_json = pet.tags.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default());
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
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error updating pet: {e}")))?;

        if result.rows_affected() == 0 {
            return Ok(UpdatePetResponse::PetNotFound);
        }

        Ok(UpdatePetResponse::SuccessfulOperation(models::Pet { id: Some(id), ..pet }))
    }

    /// Deletes a pet.
    async fn delete_pet(
        &self,
        pet_id: i64,
        api_key: Option<String>,
        context: &C) -> Result<DeletePetResponse, ApiError>
    {
        info!("delete_pet({}, {:?}) - X-Span-ID: {:?}", pet_id, api_key, context.get().0.clone());

        sqlx::query(r#"DELETE FROM pet WHERE id = $1"#)
            .bind(pet_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError(format!("DB error deleting pet: {e}")))?;

        Ok(DeletePetResponse::SuccessfulOperation)
    }

    /// Find pet by identifier.
    async fn get_pet_by_id(
        &self,
        pet_id: i64,
        context: &C) -> Result<GetPetByIdResponse, ApiError>
    {
        info!("get_pet_by_id({}) - X-Span-ID: {:?}", pet_id, context.get().0.clone());

        let row = sqlx::query(
            r#"SELECT id, name, category, photo_urls::text, tags::text, status::text
               FROM pet WHERE id = $1"#,
        )
        .bind(pet_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error fetching pet: {e}")))?;

        match row {
            None => Ok(GetPetByIdResponse::PetNotFound),
            Some(row) => {
                let pet = row_to_pet(
                    row.get("id"),
                    row.get("name"),
                    row.get("category"),
                    row.get::<String, _>("photo_urls"),
                    row.get("tags"),
                    row.get("status"),
                );
                Ok(GetPetByIdResponse::SuccessfulOperation(pet))
            }
        }
    }

    /// Updates a pet in the store with form data.
    async fn update_pet_with_form(
        &self,
        pet_id: i64,
        name: Option<String>,
        status: Option<String>,
        context: &C) -> Result<UpdatePetWithFormResponse, ApiError>
    {
        info!("update_pet_with_form({}, {:?}, {:?}) - X-Span-ID: {:?}", pet_id, name, status, context.get().0.clone());

        // Parse status string into the enum type string we can cast in Postgres.
        let status_enum: Option<String> = status.as_deref().and_then(|s| {
            s.parse::<models::PetStatus>().ok().map(|ps| ps.to_string())
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
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error updating pet with form: {e}")))?;

        if result.rows_affected() == 0 {
            return Ok(UpdatePetWithFormResponse::InvalidInput);
        }

        Ok(UpdatePetWithFormResponse::SuccessfullyUpdated)
    }

    /// Uploads an image.
    async fn upload_file(
        &self,
        pet_id: i64,
        additional_metadata: Option<String>,
        body: Option<swagger::ByteArray>,
        context: &C) -> Result<UploadFileResponse, ApiError>
    {
        info!("upload_file({}, {:?}, {:?}) - X-Span-ID: {:?}", pet_id, additional_metadata, body, context.get().0.clone());

        // Verify the pet exists; no file persistence (mirrors sibling behaviour).
        let exists: bool = sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM pet WHERE id = $1)"#)
            .bind(pet_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError(format!("DB error checking pet: {e}")))?;

        if !exists {
            return Ok(UploadFileResponse::UnexpectedError(models::Error {
                code: "404".to_string(),
                message: "Pet not found".to_string(),
            }));
        }

        Ok(UploadFileResponse::SuccessfulOperation(models::ApiResponse {
            code: Some(200),
            r#type: Some("ok".to_string()),
            message: additional_metadata,
        }))
    }

    // -----------------------------------------------------------------------
    // Store operations
    // -----------------------------------------------------------------------

    /// Returns pet inventories by status.
    async fn get_inventory(
        &self,
        context: &C) -> Result<GetInventoryResponse, ApiError>
    {
        info!("get_inventory() - X-Span-ID: {:?}", context.get().0.clone());

        let rows = sqlx::query(
            r#"SELECT status::text, CAST(COUNT(*) AS int) AS cnt
               FROM pet WHERE status IS NOT NULL GROUP BY status"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error getting inventory: {e}")))?;

        let mut inventory: HashMap<String, i32> = HashMap::new();
        for row in &rows {
            let status: String = row.get("status");
            let cnt: i32 = row.get("cnt");
            inventory.insert(status, cnt);
        }

        Ok(GetInventoryResponse::SuccessfulOperation(inventory))
    }

    /// Place an order for a pet.
    async fn place_order(
        &self,
        order: Option<models::Order>,
        context: &C) -> Result<PlaceOrderResponse, ApiError>
    {
        info!("place_order({:?}) - X-Span-ID: {:?}", order, context.get().0.clone());

        let order = match order {
            None => return Ok(PlaceOrderResponse::InvalidInput),
            Some(o) => o,
        };

        let id = match order.id {
            Some(id) => id,
            None => next_id(&self.pool, r#""order""#).await?,
        };

        let status_str = order.status.as_ref().map(|s| s.to_string());
        let ship_date_naive = order.ship_date.map(|dt| dt.naive_utc());

        sqlx::query(
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
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error placing order: {e}")))?;

        let saved = models::Order { id: Some(id), ..order };
        Ok(PlaceOrderResponse::SuccessfulOperation(saved))
    }

    /// Delete purchase order by identifier.
    async fn delete_order(
        &self,
        order_id: i64,
        context: &C) -> Result<DeleteOrderResponse, ApiError>
    {
        info!("delete_order({}) - X-Span-ID: {:?}", order_id, context.get().0.clone());

        let result = sqlx::query(r#"DELETE FROM "order" WHERE id = $1"#)
            .bind(order_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError(format!("DB error deleting order: {e}")))?;

        if result.rows_affected() == 0 {
            return Ok(DeleteOrderResponse::OrderNotFound);
        }

        Ok(DeleteOrderResponse::SuccessfulOperation)
    }

    /// Find purchase order by identifier.
    async fn get_order_by_id(
        &self,
        order_id: i64,
        context: &C) -> Result<GetOrderByIdResponse, ApiError>
    {
        info!("get_order_by_id({}) - X-Span-ID: {:?}", order_id, context.get().0.clone());

        let row = sqlx::query(
            r#"SELECT id, pet_id, quantity, ship_date, status::text, complete
               FROM "order" WHERE id = $1"#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error fetching order: {e}")))?;

        match row {
            None => Ok(GetOrderByIdResponse::OrderNotFound),
            Some(row) => {
                let order = row_to_order(
                    row.get("id"),
                    row.get("pet_id"),
                    row.get("quantity"),
                    row.get("ship_date"),
                    row.get("status"),
                    row.get("complete"),
                );
                Ok(GetOrderByIdResponse::SuccessfulOperation(order))
            }
        }
    }

    // -----------------------------------------------------------------------
    // User operations
    // -----------------------------------------------------------------------

    /// Create user.
    async fn create_user(
        &self,
        user: Option<models::User>,
        context: &C) -> Result<CreateUserResponse, ApiError>
    {
        info!("create_user({:?}) - X-Span-ID: {:?}", user, context.get().0.clone());

        let user = match user {
            None => return Ok(CreateUserResponse::UnexpectedError(models::Error {
                code: "400".to_string(),
                message: "No user body provided".to_string(),
            })),
            Some(u) => u,
        };

        sqlx::query(
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
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.phone)
        .bind(user.user_status)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error creating user: {e}")))?;

        Ok(CreateUserResponse::SuccessfulOperation(user))
    }

    /// Creates list of users with given input array.
    async fn create_users_with_list_input<'a>(
        &self,
        user: Option<&'a Vec<models::User>>,
        context: &C) -> Result<CreateUsersWithListInputResponse, ApiError>
    {
        info!("create_users_with_list_input({:?}) - X-Span-ID: {:?}", user, context.get().0.clone());

        let users = match user {
            None => return Ok(CreateUsersWithListInputResponse::UnexpectedError(
                models::Error { code: "400".to_string(), message: "No users provided".to_string() }
            )),
            Some(u) if u.is_empty() => return Ok(CreateUsersWithListInputResponse::UnexpectedError(
                models::Error { code: "400".to_string(), message: "No users provided".to_string() }
            )),
            Some(u) => u,
        };

        let mut last_user = None;
        for u in users {
            sqlx::query(
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
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError(format!("DB error creating user in list: {e}")))?;
            last_user = Some(u.clone());
        }

        Ok(CreateUsersWithListInputResponse::SuccessfulOperation(last_user.unwrap()))
    }

    /// Logs user into the system.
    async fn login_user(
        &self,
        username: Option<String>,
        password: Option<String>,
        context: &C) -> Result<LoginUserResponse, ApiError>
    {
        info!("login_user({:?}, {:?}) - X-Span-ID: {:?}", username, password, context.get().0.clone());

        let username = match username {
            None => return Ok(LoginUserResponse::InvalidUsername),
            Some(u) => u,
        };
        let password = password.unwrap_or_default();

        let row = sqlx::query(
            r#"SELECT password FROM "user" WHERE username = $1"#,
        )
        .bind(&username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error during login: {e}")))?;

        let stored_pw: Option<String> = row.as_ref().and_then(|r| r.get("password"));

        if row.is_none() || stored_pw.as_deref() != Some(&password) {
            return Ok(LoginUserResponse::InvalidUsername);
        }

        let token = format!("logged-in-{username}");
        let expires_after = chrono::Utc::now() + chrono::Duration::hours(1);

        Ok(LoginUserResponse::SuccessfulOperation {
            body: token,
            x_rate_limit: Some(1000),
            x_expires_after: Some(expires_after),
        })
    }

    /// Logs out current logged in user session.
    async fn logout_user(
        &self,
        context: &C) -> Result<LogoutUserResponse, ApiError>
    {
        info!("logout_user() - X-Span-ID: {:?}", context.get().0.clone());
        // Stateless — no session to clear.
        Ok(LogoutUserResponse::SuccessfulOperation)
    }

    /// Delete user.
    async fn delete_user(
        &self,
        username: String,
        context: &C) -> Result<DeleteUserResponse, ApiError>
    {
        info!("delete_user(\"{}\") - X-Span-ID: {:?}", username, context.get().0.clone());

        let result = sqlx::query(r#"DELETE FROM "user" WHERE username = $1"#)
            .bind(&username)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError(format!("DB error deleting user: {e}")))?;

        if result.rows_affected() == 0 {
            return Ok(DeleteUserResponse::UserNotFound);
        }

        Ok(DeleteUserResponse::SuccessfulOperation)
    }

    /// Get user by user name.
    async fn get_user_by_name(
        &self,
        username: String,
        context: &C) -> Result<GetUserByNameResponse, ApiError>
    {
        info!("get_user_by_name(\"{}\") - X-Span-ID: {:?}", username, context.get().0.clone());

        let row = sqlx::query(
            r#"SELECT id, username, first_name, last_name, email, password, phone, user_status
               FROM "user" WHERE username = $1"#,
        )
        .bind(&username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error fetching user: {e}")))?;

        match row {
            None => Ok(GetUserByNameResponse::UserNotFound),
            Some(row) => {
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
                Ok(GetUserByNameResponse::SuccessfulOperation(user))
            }
        }
    }

    /// Update user.
    async fn update_user(
        &self,
        username: String,
        user: Option<models::User>,
        context: &C) -> Result<UpdateUserResponse, ApiError>
    {
        info!("update_user(\"{}\", {:?}) - X-Span-ID: {:?}", username, user, context.get().0.clone());

        let user = match user {
            None => return Ok(UpdateUserResponse::UnexpectedError(models::Error {
                code: "400".to_string(),
                message: "No user body provided".to_string(),
            })),
            Some(u) => u,
        };

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
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError(format!("DB error updating user: {e}")))?;

        if result.rows_affected() == 0 {
            return Ok(UpdateUserResponse::UnexpectedError(models::Error {
                code: "404".to_string(),
                message: "User not found".to_string(),
            }));
        }

        Ok(UpdateUserResponse::SuccessfulOperation)
    }
}

// ---------------------------------------------------------------------------
// Unit tests for pure helpers and persistence-conversion conventions.
//
// These tests are database-free: they exercise the row<->model conversion
// helpers, enum string conversions, and the JSON/timestamp encoding
// conventions documented in AGENTS.md. Anything that genuinely needs Postgres
// (e.g. `next_id`) is intentionally not tested here.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{row_to_order, row_to_pet, row_to_user};
    use petstore_server::models;

    // --- Pet row -> model -----------------------------------------------

    #[test]
    fn row_to_pet_decodes_all_columns() {
        let pet = row_to_pet(
            Some(7),
            "Fido".to_string(),
            Some(r#"{"id":1,"name":"dogs"}"#.to_string()),
            r#"["http://example.com/a.jpg","http://example.com/b.jpg"]"#.to_string(),
            Some(r#"[{"id":3,"name":"cute"}]"#.to_string()),
            Some("available".to_string()),
        );

        assert_eq!(pet.id, Some(7));
        assert_eq!(pet.name, "Fido");
        assert_eq!(
            pet.category,
            Some(models::Category {
                id: Some(1),
                name: Some("dogs".to_string())
            })
        );
        assert_eq!(
            pet.photo_urls,
            vec![
                "http://example.com/a.jpg".to_string(),
                "http://example.com/b.jpg".to_string()
            ]
        );
        assert_eq!(
            pet.tags,
            Some(vec![models::Tag {
                id: Some(3),
                name: Some("cute".to_string())
            }])
        );
        assert_eq!(pet.status, Some(models::PetStatus::Available));
    }

    #[test]
    fn row_to_pet_handles_null_optionals() {
        let pet = row_to_pet(None, "Rex".to_string(), None, "[]".to_string(), None, None);

        assert_eq!(pet.id, None);
        assert_eq!(pet.category, None);
        assert!(pet.photo_urls.is_empty());
        assert_eq!(pet.tags, None);
        assert_eq!(pet.status, None);
    }

    #[test]
    fn row_to_pet_tolerates_malformed_json_columns() {
        // Malformed photo_urls falls back to an empty Vec; malformed
        // category/tags/status fall back to None rather than panicking.
        let pet = row_to_pet(
            Some(1),
            "Spot".to_string(),
            Some("not json".to_string()),
            "also not json".to_string(),
            Some("{".to_string()),
            Some("teleporting".to_string()),
        );

        assert!(pet.photo_urls.is_empty());
        assert_eq!(pet.category, None);
        assert_eq!(pet.tags, None);
        assert_eq!(pet.status, None);
    }

    // --- Order row -> model ---------------------------------------------

    #[test]
    fn row_to_order_decodes_and_assumes_utc_for_ship_date() {
        let naive = chrono::NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_opt(3, 4, 5)
            .unwrap();

        let order = row_to_order(
            Some(10),
            Some(20),
            Some(2),
            Some(naive),
            Some("placed".to_string()),
            Some(true),
        );

        assert_eq!(order.id, Some(10));
        assert_eq!(order.pet_id, Some(20));
        assert_eq!(order.quantity, Some(2));
        assert_eq!(order.status, Some(models::OrderStatus::Placed));
        assert_eq!(order.complete, Some(true));

        let ship_date = order.ship_date.expect("ship_date should be set");
        assert_eq!(ship_date.naive_utc(), naive);
    }

    #[test]
    fn row_to_order_handles_null_ship_date_and_status() {
        let order = row_to_order(Some(1), None, None, None, None, None);
        assert_eq!(order.ship_date, None);
        assert_eq!(order.status, None);
    }

    // --- User row -> model ----------------------------------------------

    #[test]
    fn row_to_user_maps_fields_directly() {
        let user = row_to_user(
            Some(5),
            Some("alice".to_string()),
            Some("Alice".to_string()),
            Some("Smith".to_string()),
            Some("alice@example.com".to_string()),
            Some("secret".to_string()),
            Some("555-1234".to_string()),
            Some(1),
        );

        assert_eq!(user.id, Some(5));
        assert_eq!(user.username, Some("alice".to_string()));
        assert_eq!(user.first_name, Some("Alice".to_string()));
        assert_eq!(user.last_name, Some("Smith".to_string()));
        assert_eq!(user.email, Some("alice@example.com".to_string()));
        assert_eq!(user.password, Some("secret".to_string()));
        assert_eq!(user.phone, Some("555-1234".to_string()));
        assert_eq!(user.user_status, Some(1));
    }

    // --- Enum <-> string conversions ------------------------------------

    #[test]
    fn pet_status_display_and_fromstr_round_trip() {
        for (variant, text) in [
            (models::PetStatus::Available, "available"),
            (models::PetStatus::Pending, "pending"),
            (models::PetStatus::Sold, "sold"),
        ] {
            assert_eq!(variant.to_string(), text);
            assert_eq!(text.parse::<models::PetStatus>().unwrap(), variant);
        }
    }

    #[test]
    fn order_status_display_and_fromstr_round_trip() {
        for (variant, text) in [
            (models::OrderStatus::Placed, "placed"),
            (models::OrderStatus::Approved, "approved"),
            (models::OrderStatus::Delivered, "delivered"),
        ] {
            assert_eq!(variant.to_string(), text);
            assert_eq!(text.parse::<models::OrderStatus>().unwrap(), variant);
        }
    }

    #[test]
    fn invalid_enum_strings_are_rejected() {
        assert!("nonsense".parse::<models::PetStatus>().is_err());
        assert!("nonsense".parse::<models::OrderStatus>().is_err());
        // Conversion is case-sensitive.
        assert!("Available".parse::<models::PetStatus>().is_err());
    }

    // --- JSON encoding conventions --------------------------------------

    #[test]
    fn photo_urls_json_round_trip() {
        let urls = vec!["http://a/1.jpg".to_string(), "http://a/2.jpg".to_string()];
        let json = serde_json::to_string(&urls).unwrap();
        assert_eq!(json, r#"["http://a/1.jpg","http://a/2.jpg"]"#);

        let decoded: Vec<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, urls);
    }

    #[test]
    fn tags_json_round_trip() {
        let tags = vec![
            models::Tag {
                id: Some(1),
                name: Some("a".to_string()),
            },
            models::Tag {
                id: Some(2),
                name: Some("b".to_string()),
            },
        ];
        let json = serde_json::to_string(&tags).unwrap();
        let decoded: Vec<models::Tag> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, tags);
    }

    #[test]
    fn category_serializes_as_json_string_and_round_trips_through_row_to_pet() {
        // AGENTS.md: category is stored as a JSON string in a TEXT column. The
        // write path uses serde_json::to_string; the read path is row_to_pet.
        let category = models::Category {
            id: Some(42),
            name: Some("reptiles".to_string()),
        };
        let stored = serde_json::to_string(&category).unwrap();
        assert_eq!(stored, r#"{"id":42,"name":"reptiles"}"#);

        let pet = row_to_pet(
            Some(1),
            "Iggy".to_string(),
            Some(stored),
            "[]".to_string(),
            None,
            None,
        );
        assert_eq!(pet.category, Some(category));
    }

    // --- chrono NaiveDateTime conversion (ship_date) --------------------

    #[test]
    fn ship_date_naive_utc_conversion_round_trips() {
        // Mirrors place_order (DateTime -> naive_utc) and row_to_order
        // (naive -> DateTime<Utc>).
        let original = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2030, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 58)
                .unwrap(),
            chrono::Utc,
        );

        let naive = original.naive_utc();
        let restored =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive, chrono::Utc);

        assert_eq!(restored, original);
    }
}

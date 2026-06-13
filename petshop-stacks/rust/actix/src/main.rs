use actix_web::{middleware, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;

mod db_config;
mod handlers;
mod helpers;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let dsn = db_config::build_dsn();
    let max_connections = db_config::pool_max_connections();

    log::info!("Connecting to database with pool size {max_connections}");

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&dsn)
        .await
        .expect("Failed to connect to database");

    let pool_data = web::Data::new(pool);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    log::info!("Starting server on {bind_addr}");

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                let response = actix_web::HttpResponse::BadRequest()
                    .body(format!("JSON error: {err}"));
                actix_web::error::InternalError::from_response(err, response).into()
            }))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/v3")
                    // Pet endpoints — static paths must come before parameterised ones
                    .route("/pet/findByStatus", web::get().to(handlers::pet::find_pets_by_status))
                    .route("/pet/findByTags", web::get().to(handlers::pet::find_pets_by_tags))
                    .route("/pet", web::post().to(handlers::pet::add_pet))
                    .route("/pet", web::put().to(handlers::pet::update_pet))
                    .route("/pet/{petId}/uploadImage", web::post().to(handlers::pet::upload_file))
                    .route("/pet/{petId}", web::get().to(handlers::pet::get_pet_by_id))
                    .route("/pet/{petId}", web::post().to(handlers::pet::update_pet_with_form))
                    .route("/pet/{petId}", web::delete().to(handlers::pet::delete_pet))
                    // Store endpoints
                    .route("/store/inventory", web::get().to(handlers::store::get_inventory))
                    .route("/store/order", web::post().to(handlers::store::place_order))
                    .route("/store/order/{orderId}", web::get().to(handlers::store::get_order_by_id))
                    .route("/store/order/{orderId}", web::delete().to(handlers::store::delete_order))
                    // User endpoints — static paths must come before parameterised ones
                    .route("/user/createWithList", web::post().to(handlers::user::create_users_with_list_input))
                    .route("/user/login", web::get().to(handlers::user::login_user))
                    .route("/user/logout", web::get().to(handlers::user::logout_user))
                    .route("/user", web::post().to(handlers::user::create_user))
                    .route("/user/{username}", web::get().to(handlers::user::get_user_by_name))
                    .route("/user/{username}", web::put().to(handlers::user::update_user))
                    .route("/user/{username}", web::delete().to(handlers::user::delete_user)),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}

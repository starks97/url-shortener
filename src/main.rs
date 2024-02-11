use actix_cors::Cors;
use actix_web::{
    http::header,
    web::{Data, ServiceConfig},
};

use shuttle_actix_web::ShuttleActixWeb;
use shuttle_secrets::SecretStore;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use redis::Client;
mod config;

mod config_secrets;

pub struct AppState {
    db: Pool<Postgres>,
    secrets: config_secrets::Config,
    redis_client: Client,
}

mod api;
mod errors;
mod jwt_auth;
mod models;
mod token;

use api::{handler::config_handler, health_route::health_checker_handler};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config_data = config_secrets::Config::init(&secret_store);

    let database_url = config_data.database_url.to_owned();
    let pool: Pool<Postgres> = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Connection to the db is successful!");
            pool
        }
        Err(err) => {
            print!("Failed to connect{:?}", err);
            std::process::exit(1)
        }
    };

    let add_migrations = sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("Migrations ran successfully: {:?}", add_migrations);

    println!("Server started successfully 🚀!");

    let redis_client = match Client::open(config_data.redis_url.to_owned()) {
        Ok(client) => {
            println!("✅Connection to the redis is successful!");
            client
        }
        Err(e) => {
            println!("🔥 Error connecting to Redis: {}", e);
            std::process::exit(1);
        }
    };

    let config = move |cfg: &mut ServiceConfig| {
        Cors::default()
            .allowed_origin(&config_data.client_origin) // Set your allowed origin(s)
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH"]) // Set allowed HTTP methods
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ]) // Set allowed headers
            .supports_credentials();

        cfg.app_data(Data::new(AppState {
            db: pool.clone(),
            secrets: config_data.clone(),
            redis_client: redis_client.clone(),
        }))
        .service(health_checker_handler)
        .configure(config_handler);
    };

    Ok(config.into())
}

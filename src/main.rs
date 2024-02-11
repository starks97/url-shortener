use actix_cors::Cors;
use actix_web::{
    http::header,
    web::{Data, ServiceConfig},
    App, HttpServer,
};
use dotenv::dotenv;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use redis::Client;
mod config;

pub struct AppState {
    db: Pool<Postgres>,
    env: config::Config,
    redis_client: Client,
}

mod api;
mod errors;
mod jwt_auth;
mod models;
mod token;

use api::{handler::config_handler, health_route::health_checker_handler};

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // Check if the logger has been initialized
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info")
    }
    if !log::log_enabled!(log::Level::Info) {
        env_logger::init();
    }

    dotenv().ok();

    let config_data = config::Config::init();

    let database_url = config_data.database_url.clone();
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
            env: config_data.clone(),
            redis_client: redis_client.clone(),
        }))
        .service(health_checker_handler)
        .configure(config_handler);
    };

    Ok(config.into())
}

use actix_cors::Cors;
use actix_web::{http::header, web::Data, App, HttpServer};
use dotenv::dotenv;
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info")
    }
    dotenv().ok();
    env_logger::init();

    let config = config::Config::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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

    let redis_client = match Client::open(config.redis_url.to_owned()) {
        Ok(client) => {
            println!("✅Connection to the redis is successful!");
            client
        }
        Err(e) => {
            println!("🔥 Error connecting to Redis: {}", e);
            std::process::exit(1);
        }
    };

    HttpServer::new(move || {
        Cors::default()
            .allowed_origin(&config.client_origin)
            .allowed_origin("http://localhost:3000/")
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();
        App::new()
            .app_data(Data::new(AppState {
                db: pool.clone(),
                env: config.clone(),
                redis_client: redis_client.clone(),
            }))
            .service(health_checker_handler)
            .configure(config_handler)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

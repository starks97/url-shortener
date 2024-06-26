use actix_web::web::{Data, ServiceConfig};

use shuttle_actix_web::ShuttleActixWeb;

use shuttle_runtime::SecretStore;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use redis::Client;

use url_shortener_api::{
    api::{handler::config_handler, health_route::health_checker_handler},
    app_state::AppState,
    config_secrets,
};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config_data = config_secrets::Config::init(&secrets);

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
        cfg.app_data(Data::new(AppState {
            db: pool.clone(),
            secrets: config_data.clone(),
            redis_client: redis_client.clone(),
        }))
        .service(health_checker_handler)
        .configure(|ctx| config_handler(ctx, &config_data));
    };

    Ok(config.into())
}

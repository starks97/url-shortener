use config_secrets::Config;
use redis::Client;
use sqlx::PgPool;

use crate::config_secrets;

pub struct AppState {
    pub db: PgPool,
    pub secrets: Config,
    pub redis_client: Client,
}

impl AppState {
    pub fn new(db: PgPool, secrets: Config, redis_client: Client) -> Self {
        Self {
            db,
            secrets,
            redis_client,
        }
    }
}

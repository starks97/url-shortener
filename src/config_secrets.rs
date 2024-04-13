use serde::Deserialize;
use shuttle_runtime::SecretStore;

pub fn get_secret(secret_store: &SecretStore, secret_name: &str) -> String {
    secret_store
        .get(secret_name)
        .unwrap_or_else(|| panic!("{} must be set", secret_name))
}

fn parse_duration(duration_str: &str) -> Option<i64> {
    let mut numeric_part = String::new();
    for c in duration_str.chars() {
        if c.is_numeric() {
            numeric_part.push(c);
        } else {
            break;
        }
    }
    numeric_part.parse::<i64>().ok()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub client_origin: String,
    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_expires_in: String,
    pub access_token_max_age: i64,
    pub refresh_token_private_key: String,
    pub refresh_token_public_key: String,
    pub refresh_token_expires_in: String,
    pub refresh_token_max_age: i64,
}

impl Config {
    pub fn init(secret_store: &SecretStore) -> Config {
        let database_url = get_secret(secret_store, "DATABASE_URL");
        let redis_url = get_secret(secret_store, "REDIS_URL");
        let client_origin = get_secret(secret_store, "CLIENT_ORIGIN");
        let access_token_private_key = get_secret(secret_store, "ACCESS_TOKEN_PRIVATE_KEY");
        let access_token_public_key = get_secret(secret_store, "ACCESS_TOKEN_PUBLIC_KEY");
        let access_token_expires_in = get_secret(secret_store, "ACCESS_TOKEN_EXPIRED_IN");
        let access_token_max_age = get_secret(secret_store, "ACCESS_TOKEN_MAXAGE");
        let refresh_token_private_key = get_secret(secret_store, "REFRESH_TOKEN_PRIVATE_KEY");
        let refresh_token_public_key = get_secret(secret_store, "REFRESH_TOKEN_PUBLIC_KEY");
        let refresh_token_expires_in = get_secret(secret_store, "REFRESH_TOKEN_EXPIRED_IN");
        let refresh_token_max_age = get_secret(secret_store, "REFRESH_TOKEN_MAXAGE");

        Config {
            database_url,
            redis_url,
            client_origin,
            access_token_private_key,
            access_token_public_key,
            access_token_expires_in,
            access_token_max_age: parse_duration(&access_token_max_age)
                .unwrap_or_else(|| panic!("Invalid duration: {}", access_token_max_age)),
            refresh_token_private_key,
            refresh_token_public_key,
            refresh_token_expires_in,
            refresh_token_max_age: parse_duration(&refresh_token_max_age)
                .unwrap_or_else(|| panic!("Invalid duration: {}", refresh_token_max_age)),
        }
    }
}

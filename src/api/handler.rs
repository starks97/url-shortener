use actix_cors::Cors;
use actix_web::{http::header, web};

use super::auth::{
    create_user, get_me_handler, get_users, logout_handler, refresh_access_token_handler,
};

use super::url::{
    create_url, delete_url, get_all_url_record, get_url_by_id, redirect_to_original_url, update_url,
};
use crate::config_secrets;

use log::info;

pub fn config_handler(config: &mut web::ServiceConfig, config_data: &config_secrets::Config) {
    info!("Configuring routes...");
    let cors = Cors::default()
        .allowed_origin(&config_data.client_origin)
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
        ])
        .supports_credentials()
        .max_age(3600);

    let scope = web::scope("/api")
        .service(get_users)
        .service(create_user)
        .service(refresh_access_token_handler)
        .service(logout_handler)
        .service(get_me_handler)
        .service(create_url)
        .service(get_all_url_record)
        .service(update_url)
        .service(delete_url)
        .service(get_url_by_id)
        .service(redirect_to_original_url)
        .wrap(cors);

    config.service(scope);
}

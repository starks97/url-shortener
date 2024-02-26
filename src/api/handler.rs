use actix_cors::Cors;
use actix_web::{http::header, web};

use super::auth::{
    create_user, get_me_handler, get_users, logout_handler, refresh_access_token_handler,
};

use super::url::{
    create_url, delete_url, get_all_url_record, get_url_by_id, redirect_to_original_url, update_url,
};

pub fn config_handler(config: &mut web::ServiceConfig) {
    let cors = Cors::default()
        .allowed_origin("http://127.0.0.1:4323")
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH"]) // Set allowed HTTP methods
        .allowed_headers(vec![
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::HeaderName::from_static("sentry-trace"),
            header::HeaderName::from_static("baggage"),
        ]) // Set allowed headers
        .allowed_header(header::CONTENT_TYPE) // Set allowed headers
        .expose_headers(&[header::CONTENT_DISPOSITION])
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

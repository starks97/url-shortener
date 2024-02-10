use actix_web::web;

use super::auth::{
    create_user, get_me_handler, get_users, logout_handler, refresh_access_token_handler,
};

use super::url::{
    create_url, delete_url, get_all_url_record, get_url_by_id, redirect_to_original_url, update_url,
};

pub fn config_handler(config: &mut web::ServiceConfig) {
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
        .service(redirect_to_original_url);

    config.service(scope);
}

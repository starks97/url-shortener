use actix_web::HttpRequest;
use actix_web::{
    cookie::{time::Duration as ActixWebDuration, Cookie},
    get, post, web, HttpResponse, Responder,
};
use uuid::Uuid;

use validator::Validate;

use crate::models::user::{LoginUserSchema, RegisterUserSchema, User};

use super::reponse::{filter_user_record, UserResponse};

use crate::AppState;

use crate::token::token::{generate_jwt_token, verify_jwt_token};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use redis::AsyncCommands;

use crate::jwt_auth::JwtMiddleware;

use crate::custom_error::{
    handle_validation_error, CustomDBError, CustomError, CustomHttpError, ValidationModelsErrors
};

#[post("/auth/login")]
pub async fn get_users(
    body: web::Json<LoginUserSchema>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, CustomError> {
    let is_valid = body.validate();

    if let Err(validation_error) = is_valid {
        return handle_validation_error(validation_error);
    }

    let user = match sqlx::query_as!(
        User,
        r#"SELECT * FROM "users" WHERE email = $1"#,
        body.email.to_string()
    )
    .fetch_optional(&data.db)
    .await
    {
        Ok(user) => {
            if let Some(user) = user {
                user
            } else {
                print!("user not found");
                return Err(CustomError::HttpError(CustomHttpError::EmailNotFound));
            }
        }
        Err(e) => {
            print!("there is not a client with that");
            return Err(CustomError::DatabaseError(CustomDBError::DatabaseError(e.to_string())));
        }
    };

    let is_valid = PasswordHash::new(&user.password)
        .and_then(|parsed_hash| {
            Argon2::default().verify_password(body.password.as_bytes(), &parsed_hash)
        })
        .map_or(false, |_| true);

    if !is_valid {
        return Err(CustomError::HttpError(
            CustomHttpError::CredentialsNotCorrect,
        ));
    }

    let access_token_details = match generate_jwt_token(
        user.id,
        data.secrets.access_token_max_age,
        data.secrets.access_token_private_key.to_owned(),
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            println!("something happend generating the access_token, {:?}", e);
            return Err(CustomError::HttpError(CustomHttpError::TokenNotGenerated));
        }
    };

    let refresh_token_details = match generate_jwt_token(
        user.id,
        data.secrets.refresh_token_max_age,
        data.secrets.refresh_token_private_key.to_owned(),
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            println!("something happend generating the refresh_token, {:?}", e);
            return Err(CustomError::HttpError(CustomHttpError::TokenNotGenerated));
        }
    };

    let mut redis_client = match data.redis_client.get_async_connection().await {
        Ok(redis_client) => redis_client,
        Err(e) => {
            println!("something happened to connect to redis: {:?}", e);
            return Err(CustomError::HttpError(CustomHttpError::InternalServerError));
        }
    };

    let access_result: redis::RedisResult<()> = redis_client
        .set_ex(
            access_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (data.secrets.access_token_max_age * 60) as usize,
        )
        .await;

    if let Err(e) = access_result {
        println!("something happened to access to redis: {:?}", e);
        return Err(CustomError::HttpError(CustomHttpError::InternalServerError));
    }

    let refresh_result: redis::RedisResult<()> = redis_client
        .set_ex(
            refresh_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (data.secrets.refresh_token_max_age * 60) as usize,
        )
        .await;

    if let Err(e) = refresh_result {
        print!("something happened to access to redis: {:?}", e);
        return Err(CustomError::HttpError(CustomHttpError::RedisProblem));
    }

    let access_cookie = Cookie::build("access_token", access_token_details.token.clone().unwrap())
        .path("/")
        .max_age(ActixWebDuration::new(
            data.secrets.access_token_max_age * 60,
            0,
        ))
        .http_only(true)
        .domain("localhost")
        .same_site(actix_web::cookie::SameSite::Lax)
        .secure(true)
        .finish();

    let refresh_cookie = Cookie::build(
        "refresh_token",
        refresh_token_details.token.clone().unwrap(),
    )
    .path("/")
    .max_age(ActixWebDuration::new(
        data.secrets.refresh_token_max_age * 60,
        0,
    ))
    .http_only(true)
    .domain("localhost")
    .secure(true)
    .same_site(actix_web::cookie::SameSite::Lax)
    .finish();

    let logged_in_cookie = Cookie::build("logged_in", "true")
        .path("/")
        .max_age(ActixWebDuration::new(
            data.secrets.access_token_max_age * 60,
            0,
        ))
        .http_only(false)
        .domain("localhost")
        .secure(true)
        .finish();

    Ok(HttpResponse::Ok()
    .cookie(access_cookie)
    .cookie(refresh_cookie)
    .cookie(logged_in_cookie)
  
    .json(serde_json::json!({"status": "success", "access_token": access_token_details.token.unwrap()})))
}

#[post("/auth/register")]
pub async fn create_user(
    body: web::Json<RegisterUserSchema>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, CustomError> {
    let is_valid = body.validate();

    if let Err(validation_error) = is_valid {
        return handle_validation_error(validation_error);
    }

    match sqlx::query_as!(
        User,
        r#"SELECT * FROM "users" WHERE email = $1"#,
        body.email.to_string(),
    )
    .fetch_optional(&data.db)
    .await
    {
        Ok(user) => {
            if user.is_some() {
                return Err(CustomError::HttpError(CustomHttpError::UserAlreadyExists));
            }
        }
        Err(e) => {
            println!("Error occurred while querying the database");
            // Error occurred while querying the database
            return Err(CustomError::DatabaseError(CustomDBError::DatabaseError(e.to_string())));
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash_pass = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .expect("Error hashing password")
        .to_string();

    match sqlx::query_as!(
        User,
        r#"INSERT INTO "users" (name, email, password) VALUES ($1, $2, $3) RETURNING *"#,
        body.name.to_string(),
        body.email.to_string().to_lowercase(),
        hash_pass
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(user) => Ok(HttpResponse::Ok().json(UserResponse {
            status: "success".to_string(),
            data: filter_user_record(&user),
        })),
        Err(err) => {
            println!("Error occurred while creating user: {:?}", err);
            // Error occurred while creating user

            return Err(CustomError::DatabaseError(CustomDBError::DatabaseError(err.to_string())));
        }
    }
}

#[get("/auth/refresh")]
async fn refresh_access_token_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> Result<HttpResponse, CustomError> {
    let message = "could not refresh access token";

    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => {
            return Err(CustomError::HttpError(CustomHttpError::TokenNotProvided))
        }
    };

    let refresh_token_details = match verify_jwt_token(
        data.secrets.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return Err(CustomError::HttpError(CustomHttpError::TokenNotMatch));
        }
    };

    let result = data.redis_client.get_async_connection().await;
    let mut redis_client = match result {
        Ok(redis_client) => redis_client,
        Err(e) => {
            return Err(CustomError::HttpError(CustomHttpError::RedisProblem));
        }
    };
    let redis_result: redis::RedisResult<String> = redis_client
        .get(refresh_token_details.token_uuid.to_string())
        .await;

    let user_id = match redis_result {
        Ok(value) => value,
        Err(_) => {
            return Err(CustomError::HttpError(CustomHttpError::UserNotInRedis));
        }
    };

    let user_id_uuid = Uuid::parse_str(&user_id).unwrap();
    let query_result = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id_uuid)
        .fetch_optional(&data.db)
        .await
        .unwrap();

    if query_result.is_none() {
        return Err(CustomError::HttpError(CustomHttpError::TokenNotMatch));
    }

    let user = query_result.unwrap();

    let access_token_details = match generate_jwt_token(
        user.id,
        data.secrets.access_token_max_age,
        data.secrets.access_token_private_key.to_owned(),
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return Err(CustomError::HttpError(CustomHttpError::TokenNotGenerated))
    }
    };

    let redis_result: redis::RedisResult<()> = redis_client
        .set_ex(
            access_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (data.secrets.access_token_max_age * 60) as usize,
        )
        .await;

    if redis_result.is_err() {
        return Err(CustomError::HttpError(CustomHttpError::RedisProblem));
    }

    let access_cookie = Cookie::build("access_token", access_token_details.token.clone().unwrap())
        .path("/")
        .max_age(ActixWebDuration::new(
            data.secrets.access_token_max_age * 60,
            0,
        ))
        .http_only(true)
        .domain("localhost")
        .same_site(actix_web::cookie::SameSite::Lax)
        .secure(true)
        .finish();

    let logged_in_cookie = Cookie::build("logged_in", "true")
        .path("/")
        .max_age(ActixWebDuration::new(
            data.secrets.access_token_max_age * 60,
            0,
        ))
        .http_only(false)
        .domain("localhost")
        .secure(true)
        .finish();

    Ok(HttpResponse::Ok()
    .cookie(access_cookie)
    .cookie(logged_in_cookie)
    .json(serde_json::json!({"status": "success", "access_token": access_token_details.token.unwrap()})))
}

#[get("/auth/logout")]
async fn logout_handler(
    req: HttpRequest,
    auth_guard: JwtMiddleware,
    data: web::Data<AppState>,
) -> impl Responder {
    let message = "Token is invalid or session has expired";

    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": message}));
        }
    };

    let refresh_token_details = match verify_jwt_token(
        data.secrets.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let mut redis_client = data.redis_client.get_async_connection().await.unwrap();
    let redis_result: redis::RedisResult<usize> = redis_client
        .del(&[
            refresh_token_details.token_uuid.to_string(),
            auth_guard.access_token_uuid.to_string(),
        ])
        .await;

    if redis_result.is_err() {
        return HttpResponse::UnprocessableEntity().json(
            serde_json::json!({"status": "error", "message": format_args!("{:?}", redis_result.unwrap_err())}),
        );
    }

    let access_cookie = Cookie::build("access_token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();
    let refresh_cookie = Cookie::build("refresh_token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();
    let logged_in_cookie = Cookie::build("logged_in", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .cookie(logged_in_cookie)
        .json(serde_json::json!({"status": "success"}))
}

#[get("/users/me")]
async fn get_me_handler(jwt_guard: JwtMiddleware) -> impl Responder {
    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!(
             filter_user_record(&jwt_guard.user))
    });

    HttpResponse::Ok().json(json_response)
}

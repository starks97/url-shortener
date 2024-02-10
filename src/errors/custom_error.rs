use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use serde_json;

#[derive(Debug, Display, Error)]
pub enum AuthErrors {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalServerError,

    #[display(fmt = "Bad request")]
    BadRequest,

    #[display(fmt = "Email or Password are invalid, please try again")]
    InvalidCredentials,

    #[display(fmt = "User not found")]
    UserNotFound,

    #[display(fmt = "User already exists")]
    UserAlreadyExists,

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "Forbidden")]
    Forbidden,

    #[display(fmt = "Timeout")]
    Timeout,

    #[display(fmt = "User not created")]
    UserNotCreated,

    #[display(fmt = "Token not generated")]
    TokenNotGenerated,
}

impl error::ResponseError for AuthErrors {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(serde_json::json!({ "error": self.to_string() }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AuthErrors::BadRequest => StatusCode::BAD_REQUEST,
            AuthErrors::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthErrors::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthErrors::Forbidden => StatusCode::FORBIDDEN,
            AuthErrors::Unauthorized => StatusCode::UNAUTHORIZED,
            AuthErrors::UserAlreadyExists => StatusCode::CONFLICT,
            AuthErrors::Timeout => StatusCode::URI_TOO_LONG,
            AuthErrors::UserNotFound => StatusCode::NOT_FOUND,
            AuthErrors::UserNotCreated => StatusCode::NOT_IMPLEMENTED,
            AuthErrors::TokenNotGenerated => StatusCode::BAD_GATEWAY,
        }
    }
}

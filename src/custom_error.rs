use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use base64::display;
use derive_more::{Display, Error};

use sqlx::Error as SqlxError;
use validator::ValidationErrors;
#[derive(Debug)]
pub enum CustomDBError {
    UniqueConstraintViolation(String),
    DatabaseError(String),
    OtherError(String),
}

impl From<SqlxError> for CustomDBError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::Database(db_error) => {
                if let Some(code) = db_error.code() {
                    if code == "23505" {
                        return CustomDBError::UniqueConstraintViolation(db_error.to_string());
                    }
                }

                CustomDBError::DatabaseError(db_error.to_string())
            }
            _ => CustomDBError::OtherError(error.to_string()),
        }
    }
}

//this form its just only when you want to handle the behave of the error and return a custom message.

impl std::fmt::Display for CustomDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.error_response())
    }
}

impl ResponseError for CustomDBError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CustomDBError::UniqueConstraintViolation(msg) => {
                HttpResponse::Conflict().json(serde_json::json!({
                    "status": "error",
                    "message": format!("{}", msg)
                }))
            }
            CustomDBError::DatabaseError(code) => {
                HttpResponse::Conflict().json(serde_json::json!({
                    "status": "error",
                    "message": format!("Database error: {}", code)
                }))
            }
            CustomDBError::OtherError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "message": format!("Other error: {}", msg)
                }))
            }
        }
    }
}

#[derive(Debug)]
pub enum ValidationModelsErrors {
    Error(String),
}

impl ValidationModelsErrors {
    fn error_message(&self) -> String {
        match self {
            ValidationModelsErrors::Error(msg) => msg.to_string(),
        }
    }
}

impl std::fmt::Display for ValidationModelsErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl ResponseError for ValidationModelsErrors {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": self.error_message()
        }))
    }
}

#[derive(Debug, Display)]
pub enum CustomError {
    DatabaseError(CustomDBError),
    HttpError(CustomHttpError),
    Validation(ValidationModelsErrors),
}

// Implementación de ResponseError para CombinedError
impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CustomError::DatabaseError(db_error) => db_error.error_response(),
            CustomError::HttpError(http_error) => http_error.error_response(),
            CustomError::Validation(validation_error) => validation_error.error_response(),
        }
    }
}

impl From<CustomDBError> for CustomError {
    fn from(error: CustomDBError) -> Self {
        CustomError::DatabaseError(error)
    }
}

impl From<CustomHttpError> for CustomError {
    fn from(error: CustomHttpError) -> Self {
        CustomError::HttpError(error)
    }
}

impl From<ValidationModelsErrors> for CustomError {
    fn from(error: ValidationModelsErrors) -> Self {
        CustomError::Validation(error)
    }
}

#[derive(Debug, Display, Error)]
pub enum CustomHttpError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,
    #[display(fmt = "Unauthorized")]
    _Unauthorized,
    #[display(fmt = "The email was provided were not found, please try again.")]
    EmailNotFound,
    #[display(
        fmt = "The token was provided were not found, please provide a valid token by login "
    )]
    TokenNotProvided,
    #[display(fmt = "Service Unavailable")]
    _ServiceUnavailable,
    #[display(fmt = "Credentials not correct, please check the email and password.")]
    CredentialsNotCorrect,
    #[display(fmt = "Token not generated, please try again.")]
    TokenNotGenerated,
    #[display(fmt = "There was a problem with the Redis connection.")]
    RedisProblem,
    #[display(fmt = "The email provided already exists, please use another one.")]
    UserAlreadyExists,
    #[display(fmt = "The token provided is expired or not valid, please login to get a new one.")]
    TokenNotMatch,
    #[display(fmt = "There was a problem to find the user in Redis, please try again.")]
    UserNotInRedis,
}

impl error::ResponseError for CustomHttpError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(serde_json::json!({
                "status": "error",
                "message": format!("{}", self)
            }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            CustomHttpError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CustomHttpError::_Unauthorized => StatusCode::BAD_REQUEST,
            CustomHttpError::_ServiceUnavailable => StatusCode::CONFLICT,
            CustomHttpError::EmailNotFound => StatusCode::NOT_FOUND,
            CustomHttpError::TokenNotProvided => StatusCode::FORBIDDEN,
            CustomHttpError::CredentialsNotCorrect => StatusCode::UNAUTHORIZED,
            CustomHttpError::TokenNotGenerated => StatusCode::NOT_IMPLEMENTED,
            CustomHttpError::RedisProblem => StatusCode::SERVICE_UNAVAILABLE,
            CustomHttpError::UserAlreadyExists => StatusCode::CONFLICT,
            CustomHttpError::TokenNotMatch => StatusCode::FORBIDDEN,
            CustomHttpError::UserNotInRedis => StatusCode::NOT_FOUND,
        }
    }
}

pub fn handle_validation_error(
    validation_error: ValidationErrors,
) -> Result<HttpResponse, CustomError> {
    let error_message = validation_error
        .field_errors()
        .values()
        .map(|errors| {
            errors
                .iter()
                .map(|err| err.to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .collect::<Vec<String>>()
        .join(", ");

    Err(CustomError::Validation(ValidationModelsErrors::Error(
        error_message,
    )))
}

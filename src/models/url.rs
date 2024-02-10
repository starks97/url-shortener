use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use validator::Validate;

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Url {
    pub id: Uuid,
    pub original_url: String,
    pub short_url: String,
    pub user_id: Option<Uuid>,
    pub views: Option<i32>, // New field: views
    #[serde(default)] // Handle deserialization when the field is missing in JSON data
    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUrl {
    #[validate(url)]
    pub original_url: String,
    #[validate(length(min = 5, max = 10, code = "code_str"))]
    pub short_url: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUrl {
    #[validate(url)]
    pub original_url: String,
    #[validate(length(min = 5, max = 10, code = "code_str"))]
    pub short_url: String,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct UrlRecord {
    pub user_id: Uuid,
    pub username: String,
    pub url_id: Uuid,
    pub views: Option<i32>,
    pub original_url: String,
    pub short_url: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UrlQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UrlPath {
    pub url_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UrlPathRedirect {
    pub short_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OriginalUrl {
    pub original_url: String,
}
use chrono;
use serde::Serialize;

use crate::models::{url::Url, user::User};

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub createdAt: Option<chrono::DateTime<chrono::Utc>>,
    pub updatedAt: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Debug)]
pub struct UserData {
    pub user: FilteredUser,
}

#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub status: String,
    pub data: UserData,
}

pub fn filter_user_record(user: &User) -> UserData {
    let filtered_user = FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        createdAt: Some(user.created_at.unwrap()),
        updatedAt: Some(user.updated_at.unwrap()),
    };

    UserData {
        user: filtered_user,
    }
}

pub struct _login_response {
    pub status: String,
    pub message: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct FilteredUrl {
    pub id: String,
    pub short_url: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Debug)]
pub struct UrlData {
    pub url: FilteredUrl,
}

#[derive(Serialize, Debug)]
pub struct UrlResponse {
    pub status: String,
    pub data: UrlData,
}

pub fn filter_url_record(url: &Url) -> UrlData {
    let filtered_url = FilteredUrl {
        id: url.id.to_string(),
        short_url: url.short_url.to_string(),
        created_at: Some(url.created_at.unwrap()),
        updated_at: Some(url.created_at.unwrap()),
    };

    UrlData { url: filtered_url }
}

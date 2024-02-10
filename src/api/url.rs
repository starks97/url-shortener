use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse, Responder};

use validator::Validate;

use crate::models::url::{
    CreateUrl, OriginalUrl, UpdateUrl, Url, UrlPath, UrlPathRedirect, UrlQuery, UrlRecord,
};

use crate::models::user::User;

use crate::AppState;

use crate::token::token::verify_jwt_token;

use super::reponse::{filter_url_record, UrlResponse};

#[post("/url")]
pub async fn create_url(
    body: web::Json<CreateUrl>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let is_valid = body.validate();

    if is_valid.is_err() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"status": "fail", "message": is_valid.unwrap_err()}));
    };

    let message = "Token is invalid or session has expired";

    let access_token = match req.cookie("access_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": message}));
        }
    };

    let access_token_details =
        match verify_jwt_token(data.env.access_token_public_key.to_owned(), &access_token) {
            Ok(token_details) => token_details,
            Err(e) => {
                return HttpResponse::Forbidden().json(
                    serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}),
                );
            }
        };

    let is_valid_user = match sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
        access_token_details.user_id.to_owned()
    )
    .fetch_optional(&data.db)
    .await
    {
        Ok(Some(user)) => user.id,
        Ok(None) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": "User not found"}));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let new_url: Url = match sqlx::query_as!(
        Url,
        r#"
        INSERT INTO urls (original_url, short_url, user_id, views)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        body.original_url.to_string(),
        body.short_url.to_string(),
        is_valid_user.to_owned(),
        0
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(url) => url,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    HttpResponse::Ok().json(UrlResponse {
        status: "success".to_string(),
        data: filter_url_record(&new_url),
    })
}

#[get("/url")]
pub async fn get_all_url_record(
    data: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<UrlQuery>,
) -> impl Responder {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);

    let access_token = match req.cookie("access_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"status": "fail", "message": "token not found, please login"}),
            );
        }
    };

    let access_token_details =
        match verify_jwt_token(data.env.access_token_public_key.to_owned(), &access_token) {
            Ok(token_details) => token_details,
            Err(e) => {
                return HttpResponse::Forbidden().json(
                    serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}),
                );
            }
        };

    let get_url_by_user_result = match sqlx::query!(
            r#"SELECT u.id AS user_id, u.name AS username, url.id AS url_id, url.original_url, url.short_url, url.views, url.created_at, url.updated_at
            FROM users u
            LEFT JOIN urls url ON u.id = url.user_id
            WHERE u.id = $1 LIMIT $2 OFFSET $3"#,
            access_token_details.user_id,
            limit, // Limit
            offset  // Offset
        )
        .fetch_all(&data.db)
        .await
        {
            Ok(records) => {
                if records.is_empty() {
                    return HttpResponse::NotFound().json(serde_json::json!({"status": "fail", "message": "No records were found"}));
                }

                let mut url_records: Vec<UrlRecord> = vec![];

                for record in records {
                    url_records.push(UrlRecord {
                        user_id: record.user_id,
                        username: record.username,
                        url_id: record.url_id,
                        original_url: record.original_url,
                        short_url: record.short_url,
                        views: record.views,
                        created_at: record.created_at,
                        updated_at: record.updated_at,
                    });
                }

                HttpResponse::Ok().json(serde_json::json!({"status": "success", "data": url_records}))

            }
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}))
        };

    get_url_by_user_result
}

#[patch("/url/{url_id}")]
pub async fn update_url(
    data: web::Data<AppState>,
    body: web::Json<UpdateUrl>,
    req: HttpRequest,
    path: web::Path<UrlPath>,
) -> impl Responder {
    let is_valid = body.validate();

    if is_valid.is_err() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"status": "fail", "message": is_valid.unwrap_err()}));
    };
    let access_token = match req.cookie("access_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"status": "fail", "message": "token not found, please login"}),
            );
        }
    };

    match verify_jwt_token(data.env.access_token_public_key.to_owned(), &access_token) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let update_result = sqlx::query!(
        r#"UPDATE "urls" SET original_url = $1, short_url = $2 WHERE id = $3"#,
        body.original_url.to_string(),
        body.short_url.to_string(),
        path.url_id.clone()
    )
    .execute(&data.db)
    .await;

    match update_result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "URL updated successfully"
        })),
        Err(e) => {
            println!("Error updating URL: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Failed to update URL"
            }))
        }
    }
}

#[delete("/url/{url_id}")]
pub async fn delete_url(
    path: web::Path<UrlPath>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let access_token = match req.cookie("access_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"status": "fail", "message": "token not found, please login"}),
            );
        }
    };

    match verify_jwt_token(data.env.access_token_public_key.to_owned(), &access_token) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let delete_result = sqlx::query!(r#"DELETE FROM "urls" WHERE id = $1"#, path.url_id.clone())
        .execute(&data.db)
        .await;

    match delete_result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "URL deleted successfully"
        })),
        Err(e) => {
            println!("Error deleting URL: {:?}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": "Failed to delete URL"
            }))
        }
    }
}

#[get("/url/{url_id}")]
pub async fn get_url_by_id(
    path: web::Path<UrlPath>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let access_token = match req.cookie("access_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"status": "fail", "message": "token not found, please login"}),
            );
        }
    };

    match verify_jwt_token(data.env.access_token_public_key.to_owned(), &access_token) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let url_response = match sqlx::query_as!(
        Url,
        r#"SELECT * FROM urls WHERE id = $1"#,
        path.url_id.clone()
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(url) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "data": url
        })),
        Err(e) => {
            println!("Error fetching URL: {:?}", e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": "Failed to find the URL with the given ID"
            }));
        }
    };
    url_response
}

#[get("/url/redirect/{short_url}")]
pub async fn redirect_to_original_url(
    data: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<UrlPathRedirect>,
) -> impl Responder {
    let access_token = match req.cookie("access_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"status": "fail", "message": "token not found, please login"}),
            );
        }
    };

    match verify_jwt_token(data.env.access_token_public_key.to_owned(), &access_token) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let original_url = match sqlx::query_as!(
        OriginalUrl,
        r#"UPDATE urls
    SET views = views + 1
    WHERE short_url = $1
    RETURNING original_url"#,
        path.short_url.clone()
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(url) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "data": url
        })),
        Err(e) => {
            println!("Error fetching URL: {:?}", e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": "Failed to fetch URL"
            }));
        }
    };

    original_url
}

use actix_web::{delete, get, http, patch, post, web, HttpResponse, Responder};

use validator::Validate;

use crate::models::url::{
    CreateUrl, OriginalUrl, UpdateUrl, Url, UrlCategory, UrlPath, UrlPathRedirect, UrlQuery, UrlRecord
};

use crate::app_state::AppState;

use crate::jwt_auth::JwtMiddleware;

use crate::custom_error::{handle_validation_error, CustomDBError, CustomError};

use crate::utils::slugify::slugify;



#[post("/url")]
pub async fn create_url(
    body: web::Json<CreateUrl>,
    data: web::Data<AppState>,
    auth_guard: JwtMiddleware,
) -> Result<HttpResponse, CustomError> {
    let is_valid = body.validate();

    if let Err(validation_error) = is_valid {
        return handle_validation_error(validation_error);
    }

    let new_url: Url = match sqlx::query_as!(
        Url,
        r#"
        INSERT INTO urls (original_url, short_url, user_id, views, category, slug)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        body.original_url.to_string(),
        body.short_url.to_string(),
        auth_guard.user.id,
        0,
        body.category.to_string().into(),
        slugify(&body.short_url)
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(url) => url,
        Err(e) => {
            println!("Error creating URL: {:?}", e);
            /*return Err(CustomError::DatabaseError(
                CustomDBError::OtherError("Something happend".to_string()),
                ),
            );*/

            return Err(CustomError::DatabaseError(CustomDBError::OtherError(
                e.to_string(),
            )));
        }
    };

    Ok(HttpResponse::Created().json(serde_json::json!({
        "status": "success",
        "data": new_url
    })))
}

#[get("/url")]
pub async fn get_all_url_record(
    data: web::Data<AppState>,
    query: web::Query<UrlQuery>,
    auth_guard: JwtMiddleware,
) -> impl Responder {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);
    let category = query.category.clone().unwrap_or(UrlCategory::All);

    let get_url_by_user_result = match sqlx::query!(
    
        r#"
        SELECT u.id AS user_id, u.name AS username, url.id AS url_id, url.original_url, url.short_url, url.views, url.created_at, url.updated_at
        FROM users u
        LEFT JOIN urls url ON u.id = url.user_id
        WHERE u.id = $1 AND url.category = $2
        ORDER BY url.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        auth_guard.user.id,
        category.to_string().into(),
        limit,
        offset

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
                    //when its inside of the loop we need to clone because it was move before and exist in other variable.
                    let short_url = record.short_url.clone(); 
                    url_records.push(UrlRecord {
                        user_id: record.user_id,
                        username: record.username,
                        url_id: record.url_id,
                        original_url: record.original_url,
                        short_url: record.short_url,
                        views: record.views,
                        category: category.clone(),
                        slug: slugify(&short_url),
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
    _auth_guard: JwtMiddleware,
    path: web::Path<UrlPath>,
) -> Result<HttpResponse, CustomError> {
    let is_valid = body.validate();

    if let Err(validation_error) = is_valid {
        return handle_validation_error(validation_error);
    }

    let update_result = if let Some(original_url) = &body.original_url {
        sqlx::query!(
            r#"UPDATE urls SET original_url = $1 WHERE id = $2"#,
            original_url,
            path.url_id.clone()
        )
    } else if let Some(short_url) = &body.short_url {
        sqlx::query!(
            r#"UPDATE urls SET short_url = $1 WHERE id = $2"#,
            short_url,
            path.url_id.clone()
        )
    } else if let Some(category) = &body.category {
        sqlx::query!(
            r#"UPDATE urls SET category = $1 WHERE id = $2"#,
            category.to_string(),
            path.url_id.clone()
        )
    } else {
        return Err(CustomError::DatabaseError(CustomDBError::DatabaseError(
            "No valid fields to update".to_string(),
        )));
    }
    .execute(&data.db)
    .await;

    match update_result {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "URL updated successfully"
        }))),
        Err(e) => {
            println!("Error updating URL: {:?}", e);
            Err(CustomError::DatabaseError(CustomDBError::OtherError(
                e.to_string(),
            )))
        }
    }
}

#[delete("/url/{url_id}")]
pub async fn delete_url(
    path: web::Path<UrlPath>,
    data: web::Data<AppState>,
    _auth_guard: JwtMiddleware,
) -> impl Responder {
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
    _auth_guard: JwtMiddleware,
) -> impl Responder {
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
    path: web::Path<UrlPathRedirect>,
) -> impl Responder {
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
        Ok(row) => row.original_url,
        Err(e) => {
            println!("Error fetching URL: {:?}", e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": "Failed to fetch URL"
            }));
        }
    };

    HttpResponse::Found()
        .append_header((http::header::LOCATION, original_url.clone()))
        .finish()
}

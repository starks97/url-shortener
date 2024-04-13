use actix_web::{delete, get, http, patch, post, web,  HttpResponse, Responder};

use validator::Validate;

use crate::models::url::{
    CreateUrl, OriginalUrl, UpdateUrl, Url, UrlPath, UrlPathRedirect, UrlQuery, UrlRecord,
};

use crate::AppState;

use crate::jwt_auth::JwtMiddleware;

#[post("/url")]
pub async fn create_url(
    body: web::Json<CreateUrl>,
    data: web::Data<AppState>,
    auth_guard: JwtMiddleware,
) -> impl Responder {
    let is_valid = body.validate();

    if is_valid.is_err() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"status": "fail", "message": is_valid.unwrap_err()}));
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
        auth_guard.user.id,
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

    HttpResponse::Ok().json(serde_json::json!({"status": "success", "data": new_url}))
}

#[get("/url")]
pub async fn get_all_url_record(
    data: web::Data<AppState>,
    query: web::Query<UrlQuery>,
    auth_guard: JwtMiddleware,
) -> impl Responder {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);

    let get_url_by_user_result = match sqlx::query!(
            r#"SELECT u.id AS user_id, u.name AS username, url.id AS url_id, url.original_url, url.short_url, url.views, url.created_at, url.updated_at
            FROM users u
            LEFT JOIN urls url ON u.id = url.user_id
            WHERE u.id = $1 LIMIT $2 OFFSET $3"#,
            auth_guard.user.id,
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
    _auth_guard: JwtMiddleware,
    path: web::Path<UrlPath>,
) -> impl Responder {

    let is_valid = body.validate();

    if is_valid.is_err() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"status": "fail", "message": is_valid.unwrap_err()}));
    };
    
    let original_url = match &body.original_url {
        Some(url) => Some(url.clone()),
        None => None,
    };

    let short_url = match &body.short_url {
        Some(short) => Some(short),
        None => None,
    };

    let update_result = if original_url.is_some() {
        sqlx::query!(
            r#"UPDATE urls SET original_url = $1 WHERE id = $2"#,
            original_url,
            path.url_id.clone()
        )
       
    } else if short_url.is_some() {
        sqlx::query!(
            r#"UPDATE urls SET short_url = $1 WHERE id = $2"#,
            short_url,
            path.url_id.clone()
        )
        
    } else {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "status": "fail",
            "message": "No valid fields to update"
        }));
    }
    .execute(&data.db)
    .await;

    match update_result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "updated successfully"
        })),
        Err(e) => {
            println!("Error updating URL: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "fail",
                "message": format_args!("{:?}",e)
            }))
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

use actix_web::{web, HttpResponse};
use sqlx::MySqlPool;
use std::env;
use crate::error::AppError;
use crate::middleware::Auth;
use crate::models::{CreateNote, UpdateNote};
use crate::services::NoteService;
use crate::utils::Claims;

async fn list(pool: web::Data<MySqlPool>, claims: web::ReqData<Claims>) -> Result<HttpResponse, AppError> {
    let notes = NoteService::find_all(pool.get_ref(), claims.sub).await?;
    Ok(HttpResponse::Ok().json(notes))
}

async fn get_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let note = NoteService::find_by_id(pool.get_ref(), id).await?;
    Ok(HttpResponse::Ok().json(note))
}

async fn create(
    pool: web::Data<MySqlPool>,
    body: web::Json<CreateNote>,
    claims: web::ReqData<Claims>
) -> Result<HttpResponse, AppError> {
    tracing::info!("Creating note");
    let note = NoteService::create(pool.get_ref(), body.into_inner(), claims.sub).await?;
    Ok(HttpResponse::Created().json(note))
}

async fn update(
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
    body: web::Json<UpdateNote>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let note = NoteService::update(pool.get_ref(), id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(note))
}

async fn delete(
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    NoteService::delete(pool.get_ref(), id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is required");

    cfg.service(
        web::scope("/notes")
            .wrap(Auth { jwt_secret })
            .route("", web::get().to(list))
            .route("", web::post().to(create))
            .route("/{id}", web::get().to(get_by_id))
            .route("/{id}", web::put().to(update))
            .route("/{id}", web::delete().to(delete))
    );
}
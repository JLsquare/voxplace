use std::sync::RwLock;
use actix_web::{get, HttpRequest, HttpResponse, post, Responder};
use actix_web::web::{Data, Json, Path};
use chrono::Utc;
use rand::{Rng, thread_rng};
use serde_derive::Deserialize;
use crate::app_state::AppState;
use crate::user::check_user;

pub struct Comment {
    pub comment_id: i64,
    pub user_id: i64,
    pub place_id: Option<i64>,
    pub post_id: Option<i64>,
    pub content: String,
    pub created_at: i64,
}

impl Comment {
    pub fn new(
        comment_id: i64,
        user_id: i64,
        place_id: Option<i64>,
        post_id: Option<i64>,
        content: &str,
        created_at: i64,
    ) -> Self {
        Self {
            comment_id,
            user_id,
            place_id,
            post_id,
            content: content.to_string(),
            created_at,
        }
    }
}

#[derive(Deserialize)]
struct CreateCommentRequest {
    place_id: Option<String>,
    post_id: Option<String>,
    content: String,
}

#[get("/api/comment/post/{post_id}")]
async fn get_post_comments(
    data: Data<RwLock<AppState>>,
    path: Path<i64>,
) -> impl Responder {
    let post_id = path.into_inner();

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    match db.get_comments_by_post_id(post_id) {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to get comments : {}", e)),
    }
}

#[get("/api/comment/place/{place_id}")]
async fn get_place_comments(
    data: Data<RwLock<AppState>>,
    path: Path<i64>,
) -> impl Responder {
    let place_id = path.into_inner();

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    match db.get_comments_by_place_id(place_id) {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get comments"),
    }
}

#[post("/api/comment/create")]
async fn create_comment(
    data: Data<RwLock<AppState>>,
    req: HttpRequest,
    json: Json<CreateCommentRequest>,
) -> impl Responder {
    let user_id = match check_user(req) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid user"),
    };

    let place_id = match json.place_id {
        Some(ref id) => match id.parse::<i64>() {
            Ok(id) => Some(id),
            Err(_) => return HttpResponse::BadRequest().body("Invalid place id"),
        },
        None => None,
    };

    let post_id = match json.post_id {
        Some(ref id) => match id.parse::<i64>() {
            Ok(id) => Some(id),
            Err(_) => return HttpResponse::BadRequest().body("Invalid post id"),
        },
        None => None,
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    let comment_id = thread_rng().gen::<i64>();

    let time = Utc::now().timestamp();

    let comment = Comment::new(
        comment_id,
        user_id,
        place_id,
        post_id,
        &json.content,
        time,
    );

    match db.save_new_comment(comment) {
        Ok(_) => HttpResponse::Ok().json("Comment created"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create comment"),
    }
}
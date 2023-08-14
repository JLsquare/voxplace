use std::sync::RwLock;
use actix_web::{get, HttpRequest, HttpResponse, post, Responder};
use actix_web::web::{Data, Json, Path};
use chrono::Utc;
use rand::{Rng, thread_rng};
use serde_derive::Deserialize;
use crate::app_state::AppState;
use crate::user::check_user;

pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub voxel_id: i64,
    pub votes: i64,
    pub author_id: i64,
    pub updated: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Post {
    pub fn new(
        id: i64,
        title: &str,
        content: &str,
        voxel_id: i64,
        votes: i64,
        author_id: i64,
        updated: bool,
        created_at: i64,
        updated_at: i64,
    ) -> Self {
        Self {
            id,
            title: title.to_string(),
            content: content.to_string(),
            voxel_id,
            votes,
            author_id,
            updated,
            created_at,
            updated_at,
        }
    }
}

#[derive(Deserialize)]
struct CreatePostRequest {
    title: String,
    voxel_id: String,
    content: String,
}

#[derive(Deserialize)]
struct VoteRequest {
    post_id: String,
    vote: i64,
}

#[post("/api/post/create")]
async fn create_post(
    data: Data<RwLock<AppState>>,
    json: Json<CreatePostRequest>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match check_user(req) {
        Ok(user_id) => user_id,
        Err(res) => return res,
    };

    let voxel_id = match json.voxel_id.parse::<i64>() {
        Ok(voxel_id) => voxel_id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid voxel id"),
    };

    let post_id = thread_rng().gen::<i64>();

    let time = Utc::now().timestamp();

    let post = Post::new(post_id, &json.title, &json.content, voxel_id, 0, user_id, false, time, time);

    let app_state = match data.write() {
        Ok(app_state) => app_state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    match db.save_new_post(post) {
        Ok(_) => HttpResponse::Ok().json("Post created"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to create post : {}", e)),
    }
}

#[get("/api/post/top/{user_id}/{limit}")]
async fn get_top_posts(
    data: Data<RwLock<AppState>>,
    path: Path<(String, i64)>,
    req: HttpRequest,
) -> impl Responder {
    let (user_id, limit) = path.into_inner();

    let user_id = if user_id == "me" {
        match check_user(req) {
            Ok(uid) => uid,
            Err(res) => return res,
        }
    } else {
        match user_id.parse::<i64>() {
            Ok(uid) => uid,
            Err(_) => return HttpResponse::BadRequest().body("Invalid user id"),
        }
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    if user_id == 0 {
        match db.get_top_posts(limit) {
            Ok(posts) => HttpResponse::Ok().json(posts),
            Err(_) => HttpResponse::InternalServerError().body("Failed to get top posts"),
        }
    } else {
        match db.get_top_user_posts(user_id, limit) {
            Ok(posts) => HttpResponse::Ok().json(posts),
            Err(_) => HttpResponse::InternalServerError().body("Failed to get top posts"),
        }
    }
}

#[get("/api/post/new/{user_id}/{limit}")]
async fn get_new_posts(
    data: Data<RwLock<AppState>>,
    path: Path<(String, i64)>,
    req: HttpRequest,
) -> impl Responder {
    let (user_id, limit) = path.into_inner();

    let user_id = if user_id == "me" {
        match check_user(req) {
            Ok(uid) => uid,
            Err(res) => return res,
        }
    } else {
        match user_id.parse::<i64>() {
            Ok(uid) => uid,
            Err(_) => return HttpResponse::BadRequest().body("Invalid user id"),
        }
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    if user_id == 0 {
        match db.get_new_posts(limit) {
            Ok(posts) => HttpResponse::Ok().json(posts),
            Err(_) => HttpResponse::InternalServerError().body("Failed to get new posts"),
        }
    } else {
        match db.get_new_user_posts(user_id, limit) {
            Ok(posts) => HttpResponse::Ok().json(posts),
            Err(_) => HttpResponse::InternalServerError().body("Failed to get new posts"),
        }
    }
}

#[post("/api/post/vote")]
async fn vote_post(
    data: Data<RwLock<AppState>>,
    json: Json<VoteRequest>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match check_user(req) {
        Ok(user_id) => user_id,
        Err(res) => return res,
    };

    let post_id = match json.post_id.parse::<i64>() {
        Ok(post_id) => post_id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid post id"),
    };

    let app_state = match data.write() {
        Ok(app_state) => app_state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    match db.vote_post(post_id, json.vote) {
        Ok(_) => HttpResponse::Ok().json("Voted"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to vote post : {}", e)),
    }
}

#[get("/api/post/{post_id}")]
async fn get_post(
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

    match db.get_post(post_id) {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get post"),
    }
}
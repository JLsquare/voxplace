use std::sync::RwLock;
use actix_web::{get, HttpRequest, HttpResponse, post, Responder};
use actix_web::web::{Data, Json, Path};
use bcrypt::{DEFAULT_COST, hash};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use rand::{Rng, thread_rng};
use serde_derive::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::voxel::Voxel;

pub struct User {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub voxel_id: i64,
    pub admin: bool,
    pub created_at: i64,
    pub last_connected_at: i64,
}

impl User {
    pub fn new(
        user_id: i64,
        username: &str,
        email: &str,
        password_hash: &str,
        voxel_id: i64,
        admin: bool,
        created_at: i64,
        last_connected_at: i64,
    ) -> Self {
        Self {
            user_id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            voxel_id,
            admin,
            created_at,
            last_connected_at,
        }
    }
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditUserRequest {
    username: String,
    password: String,
    new_username: String,
    new_email: String,
    new_password: String,
}

#[get("/api/user/checkadmin")]
async fn check_admin(
    data: Data<RwLock<AppState>>,
    req: HttpRequest
) -> impl Responder {
    let user_id = match check_user(req) {
        Ok(id) => id,
        Err(res) => return res,
    };

    let is_admin = match check_user_admin(user_id, &data) {
        Ok(is_admin) => is_admin,
        Err(res) => return res,
    };

    HttpResponse::Ok().json(is_admin)
}

#[post("/api/user/register")]
async fn register_user(
    data: Data<RwLock<AppState>>,
    req: Json<RegisterRequest>
) -> impl Responder {
    let password_hash = match hash(&req.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to hash password"),
    };

    let created_at = Utc::now().timestamp();

    let user_id = thread_rng().gen::<i64>();
    let voxel_id = thread_rng().gen::<i64>();
    let voxel_name = format!("{}'s voxel", req.username);

    let mut app_state = match data.write() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to read app state"),
    };

    let voxel = Voxel::new(voxel_id, &voxel_name, 0, (8, 8, 8), None, None, None);

    app_state.add_voxel(voxel);

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to lock database"),
    };

    let user = User::new(
        user_id,
        &req.username,
        &req.email,
        &password_hash,
        voxel_id,
        false,
        created_at,
        created_at,
    );

    match db.register_user(user) {
        Ok(_) => (),
        Err(e) => return HttpResponse::InternalServerError().json(format!("Failed to register user : {}", e)),
    };

    let claim = Claims {
        sub: user_id.to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let key = "secret".as_bytes();

    let token = match encode(&Header::default(), &claim, &EncodingKey::from_secret(key)) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to generate token"),
    };

    HttpResponse::Ok().json(token)
}

#[post("/api/user/login")]
async fn login_user(
    data: Data<RwLock<AppState>>,
    json: Json<LoginRequest>
) -> impl Responder {
    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let last_connected_at = Utc::now().timestamp();

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    let user_id = match db.login_user(&json.username, &json.password) {
        Ok(user_id) => user_id,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid username or password"),
    };

    match db.update_last_connected_at(user_id, last_connected_at)
    {
        Ok(_) => (),
        Err(_) => eprintln!("Failed to update last connected at for user {}", user_id),
    };

    let claim = Claims {
        sub: user_id.to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let key = "secret".as_bytes();

    let token = match encode(&Header::default(), &claim, &EncodingKey::from_secret(key)) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to generate token"),
    };

    HttpResponse::Ok().json(token)
}

#[get("/api/user/profile/{id}")]
async fn get_user_profile(
    data: Data<RwLock<AppState>>,
    path: Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let user_id;
    let path = path.into_inner();

    if path == "me" {
        user_id = match check_user(req) {
            Ok(id) => id,
            Err(res) => return res,
        };
    } else {
        user_id = match path.parse::<i64>() {
            Ok(id) => id,
            Err(_) => return HttpResponse::BadRequest().body("Invalid id"),
        };
    }

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    if path == "me" {
        match db.get_full_user_profile(user_id) {
            Ok(user_profile) => HttpResponse::Ok().json(user_profile),
            Err(_) => HttpResponse::InternalServerError().body("Failed to get user profile"),
        }
    } else {
        match db.get_user_profile(user_id) {
            Ok(user_profile) => HttpResponse::Ok().json(user_profile),
            Err(_) => HttpResponse::InternalServerError().body("Failed to get user profile"),
        }
    }
}

#[get("/api/user/top/{limit}")]
async fn get_top_users(
    data: Data<RwLock<AppState>>,
    path: Path<i64>,
) -> impl Responder {
    let limit = path.into_inner();

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    match db.get_top_users(limit) {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get top users"),
    }
}

#[post("/api/user/edit")]
async fn edit_user(
    data: Data<RwLock<AppState>>,
    json: Json<EditUserRequest>,
) -> impl Responder {
    let app_state = match data.write() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    let user_id = match db.login_user(&json.username, &json.password) {
        Ok(user_id) => user_id,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid username or password"),
    };

    if !json.new_username.is_empty() {
        match db.update_username(user_id, &json.new_username) {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().body("Failed to update username"),
        };
    }

    if !json.new_email.is_empty() {
        match db.update_email(user_id, &json.new_email) {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().body("Failed to update email"),
        };
    }

    if !json.new_password.is_empty() {
        let password_hash = match hash(&json.new_password, DEFAULT_COST) {
            Ok(hash) => hash,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
        };

        match db.update_password(user_id, &password_hash) {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().body("Failed to update password"),
        };
    }

    HttpResponse::Ok().json("User updated")
}

pub fn check_user(req: HttpRequest) -> Result<i64, HttpResponse> {
    let header = match req.headers().get("Authorization") {
        Some(header) => header,
        None => return Err(HttpResponse::Unauthorized().body("No token provided")),
    };

    let token = match header.to_str() {
        Ok(token) => token,
        Err(_) => return Err(HttpResponse::Unauthorized().body("No token provided")),
    };

    let user_id_str = match decode::<Claims>(
        token,
        &DecodingKey::from_secret("secret".as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(c) => c.claims.sub,
        Err(_) => return Err(HttpResponse::Unauthorized().body("Invalid token")),
    };

    let user_id = match user_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return Err(HttpResponse::Unauthorized().body("Invalid token")),
    };

    Ok(user_id)
}

pub fn check_user_admin(user_id: i64, data: &Data<RwLock<AppState>>) -> Result<bool, HttpResponse> {
    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return Err(HttpResponse::InternalServerError().body("Failed to lock state")),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return Err(HttpResponse::InternalServerError().body("Failed to lock database")),
    };

    let is_admin = match db.is_admin(user_id) {
        Ok(admin) => admin,
        Err(_) => return Err(HttpResponse::InternalServerError().body("Failed to check admin")),
    };

    Ok(is_admin)
}

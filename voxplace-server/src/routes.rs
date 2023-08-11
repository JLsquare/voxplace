use crate::app_state::AppState;
use crate::place::Place;
use crate::voxel::Voxel;
use crate::websocket::PlaceWebSocketConnection;
use actix_web::http::header;
use actix_web::web::{Data, Json, Path, Payload};
use actix_web::{get, post, HttpRequest, HttpResponse, Responder, web};
use actix_web_actors::ws;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{thread_rng, Rng};
use serde_derive::{Deserialize, Serialize};
use std::io::Write;
use std::sync::RwLock;
use crate::post::Post;

#[derive(Deserialize)]
struct DrawRequest {
    x: usize,
    y: usize,
    z: usize,
    color: u8,
}

#[derive(Deserialize)]
struct UsernameRequest {
    x: usize,
    y: usize,
    z: usize,
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

#[derive(Deserialize)]
struct CreatePlaceRequest {
    name: String,
    size: (usize, usize, usize),
    cooldown: usize,
}

#[derive(Deserialize)]
struct CreateVoxelRequest {
    name: String,
    size: (usize, usize, usize),
}

#[derive(Deserialize)]
struct CreatePostRequest {
    title: String,
    voxel_id: String,
    content: String,
}

#[derive(Serialize)]
struct DrawResponse {
    username: String,
    cooldown: i64,
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

#[get("/api/place/ws/{id}")]
async fn ws_index(
    req: HttpRequest,
    stream: Payload,
    data: Data<RwLock<AppState>>,
    path: Path<String>,
) -> HttpResponse {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let place = match app_state.places.get(&id) {
        Some(place) => place,
        None => return HttpResponse::BadRequest().body("Invalid place"),
    };

    match ws::start(
        PlaceWebSocketConnection {
            place: place.clone(),
        },
        &req,
        stream,
    ) {
        Ok(response) => response,
        Err(_) => HttpResponse::InternalServerError().body("Failed to start websocket"),
    }
}

#[get("/api/place/all/{id}")]
async fn get_grid(
    data: Data<RwLock<AppState>>,
    path: Path<String>
) -> impl Responder {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let place = match app_state.places.get(&id) {
        Some(place) => place,
        None => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let place = place.read().unwrap();

    let grid: Vec<u8> = place.voxel.grid.iter().map(|cell| cell.load()).collect();

    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    match e.write_all(&grid) {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().body("Failed to compress data"),
    };

    let compressed_data = match e.finish() {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to compress data"),
    };

    HttpResponse::Ok()
        .append_header((header::CONTENT_ENCODING, "gzip"))
        .body(compressed_data)
}

#[get("/api/voxel/all/{id}")]
async fn get_voxel(
    data: Data<RwLock<AppState>>,
    path: Path<String>
) -> impl Responder {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read database"),
    };

    let voxel = match db.get_voxel(id) {
        Ok(info) => info,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read voxel info"),
    };

    let grid: Vec<u8> = voxel.grid.iter().map(|cell| cell.load()).collect();

    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    match e.write_all(&grid) {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().body("Failed to compress data"),
    };

    let compressed_data = match e.finish() {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to compress data"),
    };

    HttpResponse::Ok()
        .append_header((header::CONTENT_ENCODING, "gzip"))
        .body(compressed_data)
}

#[post("/api/voxel/save/{id}")]
async fn save_voxel(
    data: Data<RwLock<AppState>>,
    path: Path<String>,
    body: web::Bytes,
) -> impl Responder {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid voxel"),
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read database"),
    };

    let grid = body.to_vec();

    match db.save_voxel_grid(id, grid) {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().body("Failed to save voxel grid"),
    };

    HttpResponse::Ok().body("Voxel saved")
}

#[get("/api/palette/get/{id}")]
async fn get_palette(
    data: Data<RwLock<AppState>>,
    path: Path<String>
) -> impl Responder {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid voxel"),
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read database"),
    };

    let palette = match db.get_palette(id) {
        Ok(palette) => palette,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read palette"),
    };

    let palette_hex: Vec<String> = palette
        .iter()
        .map(|&(r, g, b)| format!("#{:02x}{:02x}{:02x}", r, g, b))
        .collect();

    HttpResponse::Ok().json(palette_hex)
}

#[post("/api/place/draw/{id}")]
async fn draw_voxel_http(
    data: Data<RwLock<AppState>>,
    req: HttpRequest,
    json: Json<DrawRequest>,
    path: Path<String>,
) -> impl Responder {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let mut app_state = match data.write() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let place = match app_state.places.get(&id) {
        Some(place) => place,
        None => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let user_id = match check_user(req) {
        Ok(id) => id,
        Err(res) => return res,
    };

    let time = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(time) => time.as_secs() as i64,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get time"),
    };

    let username;
    let cooldown;

    {
        let db = match app_state.database.lock() {
            Ok(db) => db,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to read database"),
        };

        let user_cooldown = db.get_user_cooldown(id, user_id).unwrap_or(0);

        if user_cooldown > time {
            return HttpResponse::BadRequest().body("Cooldown not finished");
        }

        let place = place.read().unwrap();

        match db.set_user_cooldown(id, user_id, time + place.cooldown) {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().body("Failed to set cooldown"),
        }

        username = match db.get_username(user_id) {
            Ok(username) => username,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to get username"),
        };

        cooldown = place.cooldown;

        match place.voxel.draw_voxel(json.x, json.y, json.z, json.color) {
            Ok(_) => (),
            Err(e) => return HttpResponse::BadRequest().body(e),
        };

        place.add_place_update(json.x, json.y, json.z, user_id);
    }

    app_state.places_users_updates();

    app_state.update_place_grid(id);

    let response = DrawResponse {
        username,
        cooldown: cooldown + time,
    };

    HttpResponse::Ok().json(response)
}

#[get("/api/place/cooldown/{id}")]
async fn get_cooldown(
    data: Data<RwLock<AppState>>,
    req: HttpRequest,
    path: Path<String>,
) -> impl Responder {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let user_id = match check_user(req) {
        Ok(id) => id,
        Err(res) => return res,
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read database"),
    };

    let cooldown = db.get_user_cooldown(id, user_id).unwrap_or(0);

    HttpResponse::Ok().json(cooldown)
}


#[get("/api/place/infos")]
async fn get_places_info(
    data: Data<RwLock<AppState>>
) -> impl Responder {
    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read database"),
    };

    let places_infos = match db.get_places_infos() {
        Ok(places_infos) => places_infos,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get places infos"),
    };

    HttpResponse::Ok().json(places_infos)
}

#[post("/api/place/username/{id}")]
async fn get_username(
    data: Data<RwLock<AppState>>,
    json: Json<UsernameRequest>,
    path: Path<String>,
) -> impl Responder {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get database"),
    };

    let user_id = match db.get_place_user(
        id,
        json.x as i64,
        json.y as i64,
        json.z as i64,
    ) {
        Ok(user_id) => user_id,
        Err(_) => return HttpResponse::Ok().json("Empty / Server"),
    };

    let username = match db.get_username(user_id) {
        Ok(username) => username,
        Err(_) => return HttpResponse::Ok().json(user_id.to_string()),
    };

    HttpResponse::Ok().json(username)
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

    match db.register_user(
        user_id,
        &req.username,
        &req.email,
        voxel_id,
        &password_hash,
        created_at,
        created_at,
    ) {
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

#[get("/api/user/voxels/all")]
async fn get_user_voxels(
    data: Data<RwLock<AppState>>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match check_user(req) {
        Ok(id) => id,
        Err(res) => return res,
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    let voxels = match db.get_user_voxels(user_id) {
        Ok(voxels) => voxels,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to get user voxels : {}", e)),
    };

    HttpResponse::Ok().json(voxels)
}

#[post("/api/user/voxels/create")]
async fn create_user_voxel(
    data: Data<RwLock<AppState>>,
    req: HttpRequest,
    json: Json<CreateVoxelRequest>,
) -> impl Responder {
    let user_id = match check_user(req) {
        Ok(id) => id,
        Err(res) => return res,
    };

    let voxel_id = thread_rng().gen::<i64>();

    let voxel = Voxel::new(voxel_id, &json.name, 0, json.size, None, None, None);

    let mut app_state = match data.write() {
        Ok(app_state) => app_state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock app state"),
    };

    app_state.add_voxel(voxel);

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock database"),
    };

    match db.save_new_user_voxel(user_id, voxel_id) {
        Ok(_) => HttpResponse::Ok().json("Voxel user link created"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to create voxel user link : {}", e)),
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

    let time = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(time) => time.as_secs() as i64,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get time"),
    };

    let post = Post::new(post_id, &json.title, &json.content, voxel_id, 0, user_id, false, time, time);

    let mut app_state = match data.write() {
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

#[post("/api/place/create")]
async fn create_place(
    data: Data<RwLock<AppState>>,
    json: Json<CreatePlaceRequest>,
    req: HttpRequest,
) -> impl Responder {
    {
        let user_id = match check_user(req) {
            Ok(user_id) => user_id,
            Err(res) => return res,
        };

        let is_admin = match check_user_admin(user_id, &data) {
            Ok(is_admin) => is_admin,
            Err(res) => return res,
        };

        if !is_admin {
            return HttpResponse::Unauthorized().body("You are not an admin");
        }
    }

    let voxel_id = thread_rng().gen::<i64>();
    let place_id = thread_rng().gen::<i64>();

    let mut app_state = match data.write() {
        Ok(app_state) => app_state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock app state"),
    };

    let voxel = Voxel::new(voxel_id, &json.name, 0, json.size, None, None, None);

    let place = Place::new(place_id, true, json.cooldown as i64, voxel);
    app_state.add_place(place);

    HttpResponse::Ok().json("ok")
}

fn check_user(req: HttpRequest) -> Result<i64, HttpResponse> {
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

fn check_user_admin(user_id: i64, data: &Data<RwLock<AppState>>) -> Result<bool, HttpResponse> {
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

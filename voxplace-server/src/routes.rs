use crate::app_state::AppState;
use crate::place::Place;
use crate::voxel::Voxel;
use crate::websocket::PlaceWebSocketConnection;
use actix_web::http::header;
use actix_web::web::{Data, Json, Path, Payload};
use actix_web::{get, post, Error, HttpRequest, HttpResponse, Responder};
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

#[derive(Deserialize)]
struct DrawRequest {
    id: String,
    x: usize,
    y: usize,
    z: usize,
    color: u8,
}

#[derive(Deserialize)]
struct UsernameRequest {
    id: String,
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
    palette: String,
}

#[derive(Serialize)]
struct PlaceInfo {
    name: String,
    id: String,
    size: (usize, usize, usize),
    palette: String,
    online: bool,
    online_users: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    exp: usize,
}

#[get("/api/place/ws/{id}")]
async fn ws_index(
    req: HttpRequest,
    stream: Payload,
    state: Data<RwLock<AppState>>,
    path: Path<String>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner().parse::<i64>().unwrap();
    let app_state = state.read().unwrap();

    let place = match app_state.places.get(&id) {
        Some(place) => place,
        None => return Err(actix_web::error::ErrorNotFound("No such place")),
    };

    ws::start(
        PlaceWebSocketConnection {
            place: place.clone(),
        },
        &req,
        stream,
    )
}

#[get("/api/place/all/{id}")]
async fn get_grid(data: Data<RwLock<AppState>>, path: Path<String>) -> impl Responder {
    let id = path.into_inner().parse::<i64>().unwrap();
    let app_state = data.read().unwrap();

    let place = match app_state.places.get(&id) {
        Some(place) => place,
        None => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let grid: Vec<u8> = place.voxel.grid.iter().map(|cell| cell.load()).collect();

    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(&grid).expect("Failed to write data");
    let compressed_data = e.finish().expect("Failed to finish compression");

    HttpResponse::Ok()
        .append_header((header::CONTENT_ENCODING, "gzip"))
        .body(compressed_data)
}

#[get("/api/place/palette/{id}")]
async fn get_palette(data: Data<RwLock<AppState>>, path: Path<String>) -> impl Responder {
    let id = path.into_inner().parse::<i64>().unwrap();
    let app_state = data.read().unwrap();

    let place = match app_state.places.get(&id) {
        Some(place) => place,
        None => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let palette_hex: Vec<String> = place.voxel.palette
        .iter()
        .map(|&(r, g, b)| format!("#{:02x}{:02x}{:02x}", r, g, b))
        .collect();

    HttpResponse::Ok().json(palette_hex)
}


#[post("/api/place/draw")]
async fn draw_voxel_http(
    data: Data<RwLock<AppState>>,
    req: HttpRequest,
    json: Json<DrawRequest>,
) -> impl Responder {
    let app_state = data.read().unwrap();

    let place_id = match json.id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let place = match app_state.places.get(&place_id) {
        Some(place) => place,
        None => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let token = match req.headers().get("Authorization") {
        Some(token) => token,
        None => return HttpResponse::Unauthorized().body("No token provided"),
    };

    let token_str = match token.to_str() {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to parse token"),
    };

    let user_id = match decode::<Claims>(
        &token_str,
        &DecodingKey::from_secret("secret".as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(c) => c.claims.sub,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let username = match app_state.database.lock().unwrap().get_username(user_id) {
        Ok(username) => username,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get username"),
    };

    if let Err(_) = place
        .voxel
        .draw_voxel(json.x, json.y, json.z, json.color, user_id)
    {
        return HttpResponse::InternalServerError().body("Failed to draw voxel");
    }

    HttpResponse::Ok().json(username)
}

#[get("/api/place/infos")]
async fn get_places_info(data: Data<RwLock<AppState>>) -> impl Responder {
    let app_state = data.read().unwrap();

    let places_infos = match app_state.database.lock().unwrap().get_places_infos() {
        Ok(places_infos) => places_infos,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get places infos"),
    };

    let mut info = Vec::new();

    for place_info in places_infos {
        info.push(PlaceInfo {
            name: place_info.3,
            id: place_info.0.to_string(),
            size: (
                place_info.4 as usize,
                place_info.5 as usize,
                place_info.6 as usize,
            ),
            palette: "TODO".to_string(),
            online: place_info.1,
            online_users: place_info.2 as usize,
        });
    }

    HttpResponse::Ok().json(info)
}

#[post("/api/place/username")]
async fn get_username(data: Data<RwLock<AppState>>, json: Json<UsernameRequest>) -> impl Responder {
    let app_state = data.read().unwrap();

    let place_id = match json.id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid place"),
    };

    let user_id = match app_state.database.lock().unwrap().get_place_user(
        place_id,
        json.x as i64,
        json.y as i64,
        json.z as i64,
    ) {
        Ok(user_id) => user_id,
        Err(_) => return HttpResponse::Ok().json("Empty / Server"),
    };

    let username = match app_state.database.lock().unwrap().get_username(user_id) {
        Ok(username) => username,
        Err(_) => return HttpResponse::Ok().json("Empty / Server"),
    };

    HttpResponse::Ok().json(username)
}

#[get("/api/user/checkadmin")]
async fn check_admin(data: Data<RwLock<AppState>>, req: HttpRequest) -> impl Responder {
    let app_state = data.read().unwrap();

    let token = match req.headers().get("Authorization") {
        Some(token) => token,
        None => return HttpResponse::Unauthorized().body("No token provided"),
    };

    let token_str = match token.to_str() {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to parse token"),
    };

    let user_id = match decode::<Claims>(
        &token_str,
        &DecodingKey::from_secret("secret".as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(c) => c.claims.sub,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let is_admin = match app_state.database.lock().unwrap().is_admin(user_id) {
        Ok(is_admin) => is_admin,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to check admin"),
    };

    HttpResponse::Ok().json(is_admin)
}

#[post("/api/user/register")]
async fn register_user(data: Data<RwLock<AppState>>, req: Json<RegisterRequest>) -> impl Responder {
    let app_state = data.read().unwrap();

    let password_hash = hash(&req.password, DEFAULT_COST).unwrap();

    let created_at = Utc::now().timestamp();

    let user_id = match app_state.database.lock().unwrap().register_user(
        &req.username,
        &req.email,
        &password_hash,
        created_at,
        created_at,
    ) {
        Ok(user_id) => user_id,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to register user"),
    };

    let claim = Claims {
        sub: user_id,
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
async fn login_user(data: Data<RwLock<AppState>>, req: Json<LoginRequest>) -> impl Responder {
    let app_state = data.read().unwrap();

    let last_connected_at = Utc::now().timestamp();

    let result = app_state
        .database
        .lock()
        .unwrap()
        .login_user(&req.username, &req.password);

    let user_id = match result {
        Ok(user_id) => user_id,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid username or password"),
    };

    match app_state
        .database
        .lock()
        .unwrap()
        .update_last_connected_at(user_id, last_connected_at)
    {
        Ok(_) => (),
        Err(_) => eprintln!("Failed to update last connected at for user {}", user_id),
    };

    let claim = Claims {
        sub: user_id,
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let key = "secret".as_bytes();

    let token = match encode(&Header::default(), &claim, &EncodingKey::from_secret(key)) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to generate token"),
    };

    HttpResponse::Ok().json(token)
}

#[post("/api/place/create")]
async fn create_place(
    data: Data<RwLock<AppState>>,
    json: Json<CreatePlaceRequest>,
    req: HttpRequest,
) -> impl Responder {
    {
        let app_state = data.read().unwrap();

        let token = match req.headers().get("Authorization") {
            Some(token) => token,
            None => return HttpResponse::Unauthorized().body("No token provided"),
        };

        let token_str = match token.to_str() {
            Ok(t) => t,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to parse token"),
        };

        let user_id = match decode::<Claims>(
            &token_str,
            &DecodingKey::from_secret("secret".as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(c) => c.claims.sub,
            Err(_) => return HttpResponse::Unauthorized().body("Invalid token"),
        };

        match app_state.database.lock().unwrap().is_admin(user_id) {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().body("Failed to check admin"),
        };
    }

    let voxel_id = thread_rng().gen::<i64>();
    let place_id = thread_rng().gen::<i64>();

    let mut app_state = data.write().unwrap();
    let voxel = Voxel::new(voxel_id, &json.name, None, json.size, None);
    voxel.write().unwrap();
    let place = Place::new(place_id, true, voxel);
    app_state.add_place(place);

    HttpResponse::Ok().json("ok")
}

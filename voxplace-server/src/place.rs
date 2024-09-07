use std::io::Write;
use crate::voxel::Voxel;
use std::sync::{Arc, RwLock};
use actix_web::{get, web, Error, HttpRequest, HttpResponse, post, Responder};
use actix_web::http::header;
use actix_web::web::{Data, Json, Path};
use chrono::Utc;
use flate2::Compression;
use flate2::write::GzEncoder;
use rand::{Rng, thread_rng};
use serde_derive::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::database::place::PlaceUserUpdate;
use crate::user::{check_user, check_user_admin};
use crate::websocket::PlaceWebSocketConnection;

pub struct Place {
    pub id: i64,
    pub online: bool,
    pub cooldown: i64,
    pub voxel: Arc<Voxel>,
    pending_updates: RwLock<Vec<PlaceUserUpdate>>,
    last_grid_update: i64,
}

impl Place {
    pub fn new(id: i64, online: bool, cooldown: i64, voxel: Voxel) -> Self {
        Self {
            id,
            online,
            cooldown,
            voxel: Arc::new(voxel),
            pending_updates: RwLock::new(Vec::new()),
            last_grid_update: 0,
        }
    }

    pub fn add_place_update(&self, x: usize, y: usize, z: usize, user_id: i64) {
        let voxel_update = PlaceUserUpdate {
            x,
            y,
            z,
            user_id,
            place_id: self.id,
        };
        self.pending_updates.write().unwrap().push(voxel_update);
    }

    pub fn get_place_updates(&self) -> Vec<PlaceUserUpdate> {
        let mut pending_updates = self.pending_updates.write().unwrap();
        let updates: Vec<PlaceUserUpdate> = pending_updates.drain(..).collect();
        updates
    }

    pub fn last_grid_update(&self) -> i64 {
        self.last_grid_update
    }

    pub fn set_last_grid_update(&mut self, last_grid_update: i64) {
        self.last_grid_update = last_grid_update;
    }
}

#[derive(Deserialize)]
struct DrawRequest {
    x: usize,
    y: usize,
    z: usize,
    color: u8,
}

#[derive(Deserialize)]
struct CreatePlaceRequest {
    name: String,
    size: (usize, usize, usize),
    cooldown: usize,
}

#[derive(Serialize)]
struct DrawResponse {
    username: String,
    cooldown: i64,
}

#[derive(Deserialize)]
struct UsernameRequest {
    x: usize,
    y: usize,
    z: usize,
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

    let time = Utc::now().timestamp();

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

#[get("/api/place/ws/{id}")]
async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<RwLock<AppState>>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner().parse::<i64>().map_err(|_| actix_web::error::ErrorBadRequest("Invalid place"))?;

    let app_state = data.read().map_err(|_| actix_web::error::ErrorInternalServerError("Failed to read app state"))?;

    let place = app_state.places.get(&id).ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid place"))?;

    let ws_connection = PlaceWebSocketConnection::new(place.clone());

    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    actix_web::rt::spawn(async move {
        ws_connection.run(session, msg_stream).await;
    });

    Ok(response)
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
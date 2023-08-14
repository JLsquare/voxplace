use std::io::Write;
use crate::websocket::PlaceWebSocketConnection;
use actix::{Addr, Message};
use crossbeam::atomic::AtomicCell;
use rand::{Rng, thread_rng};
use std::sync::{Mutex, RwLock};
use actix_web::{get, HttpRequest, HttpResponse, post, Responder, web};
use actix_web::http::header;
use actix_web::web::{Data, Json, Path};
use chrono::Utc;
use flate2::Compression;
use flate2::write::GzEncoder;
use serde_derive::Deserialize;
use crate::app_state::AppState;
use crate::user::check_user;

#[derive(Message, Clone, Copy)]
#[rtype(result = "()")]
pub struct UpdateMessage(pub usize, pub usize, pub usize, pub u8);

pub struct Voxel {
    pub id: i64,
    pub name: String,
    pub grid_size: (usize, usize, usize),
    pub grid: Vec<AtomicCell<u8>>,
    pub palette_id: i64,
    pub created_at: i64,
    pub last_modified_at: i64,
    sessions: Mutex<Vec<Addr<PlaceWebSocketConnection>>>,
}

impl Voxel {
    pub fn new(
        id: i64,
        name: &str,
        palette_id: i64,
        grid_size: (usize, usize, usize),
        grid: Option<Vec<AtomicCell<u8>>>,
        created_at: Option<i64>,
        last_modified_at: Option<i64>,
    ) -> Self {
        let grid = grid.unwrap_or_else(|| Voxel::generate_random_grid(grid_size));

        Self {
            id,
            name: name.to_string(),
            grid_size,
            grid,
            palette_id,
            sessions: Mutex::new(Vec::new()),
            created_at: created_at.unwrap_or_else(|| Utc::now().timestamp()),
            last_modified_at: last_modified_at.unwrap_or_else(|| Utc::now().timestamp()),
        }
    }

    pub fn draw_voxel(
        &self,
        x: usize,
        y: usize,
        z: usize,
        color: u8,
    ) -> Result<(), String> {
        let grid = &self.grid;
        let at_bottom = y == 0;
        let mut has_neighbor = false;

        if x >= self.grid_size.0 || y >= self.grid_size.1 || z >= self.grid_size.2 {
            return Err("Out of bounds".to_string());
        }

        if x > 0 && grid[self.get_index(x - 1, y, z)].load() > 0 {
            has_neighbor = true;
        }
        if x < self.grid_size.0 - 1 && grid[self.get_index(x + 1, y, z)].load() > 0 {
            has_neighbor = true;
        }
        if y > 0 && grid[self.get_index(x, y - 1, z)].load() > 0 {
            has_neighbor = true;
        }
        if y < self.grid_size.1 - 1 && grid[self.get_index(x, y + 1, z)].load() > 0 {
            has_neighbor = true;
        }
        if z > 0 && grid[self.get_index(x, y, z - 1)].load() > 0 {
            has_neighbor = true;
        }
        if z < self.grid_size.2 - 1 && grid[self.get_index(x, y, z + 1)].load() > 0 {
            has_neighbor = true;
        }

        if at_bottom || has_neighbor || grid[self.get_index(x, y, z)].load() > 0 {
            grid[self.get_index(x, y, z)].store(color);
            self.broadcast(UpdateMessage(x, y, z, color));
            Ok(())
        } else {
            Err("Voxel has no neighbors".to_string())
        }
    }

    pub fn add_session(&self, session: Addr<PlaceWebSocketConnection>) {
        self.sessions.lock().unwrap().push(session);
    }

    fn broadcast(&self, update_message: UpdateMessage) {
        let sessions = self.sessions.lock().unwrap();
        for session in sessions.iter() {
            session.do_send(update_message);
        }
    }

    fn get_index(&self, x: usize, y: usize, z: usize) -> usize {
        x * self.grid_size.0 * self.grid_size.0 + y * self.grid_size.1 + z
    }

    #[allow(dead_code)]
    fn generate_random_grid(grid_size: (usize, usize, usize)) -> Vec<AtomicCell<u8>> {
        let mut rng = rand::thread_rng();
        let mut grid_data = Vec::new();
        for _ in 0..grid_size.0 {
            for y in 0..grid_size.1 {
                for _ in 0..grid_size.2 {
                    let voxel_spawn_rate =
                        1.0 / (1.0 + ((y as f64 / grid_size.1 as f64) * 16.0 - 1.0).exp());
                    if rng.gen::<f64>() < voxel_spawn_rate {
                        grid_data.push(rng.gen_range(1..=32));
                    } else {
                        grid_data.push(0);
                    }
                }
            }
        }
        grid_data.into_iter().map(AtomicCell::new).collect()
    }

    fn generate_empty_grid(grid_size: (usize, usize, usize)) -> Vec<AtomicCell<u8>> {
        let mut grid_data = Vec::new();
        for _ in 0..grid_size.0 {
            for _ in 0..grid_size.1 {
                for _ in 0..grid_size.2 {
                    grid_data.push(0);
                }
            }
        }
        grid_data.into_iter().map(AtomicCell::new).collect()
    }
}

#[derive(Deserialize)]
struct CreateVoxelRequest {
    name: String,
    size: (usize, usize, usize),
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

#[get("/api/voxel/user/{user_id}")]
async fn get_user_voxels(
    data: Data<RwLock<AppState>>,
    path: Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let path = path.into_inner();
    let user_id = if path == "me" {
        match check_user(req) {
            Ok(id) => id,
            Err(_) => return HttpResponse::Unauthorized().body("Unauthorized"),
        }
    } else {
        match path.parse::<i64>() {
            Ok(id) => id,
            Err(_) => return HttpResponse::BadRequest().body("Invalid user"),
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

    let voxels = match db.get_user_voxels(user_id) {
        Ok(voxels) => voxels,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to get user voxels : {}", e)),
    };

    HttpResponse::Ok().json(voxels)
}

#[post("/api/voxel/create")]
async fn create_voxel(
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
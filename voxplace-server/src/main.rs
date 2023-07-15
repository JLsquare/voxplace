#[macro_use]
extern crate rusqlite;

use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::web::{Data, Path, Payload};
use actix_web::{get, post, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use flate2::write::GzEncoder;
use flate2::Compression;
use rand::Rng;
use rusqlite::{Connection, Result};
use serde_json::json;
use std::collections::HashMap;
use std::io::prelude::*;
use std::sync::{Arc, Mutex, RwLock};

type Grid = Vec<u8>;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("database.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS VoxelObject (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                grid_size INTEGER NOT NULL,
                grid BLOB NOT NULL,
                voxel_type TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS VoxelUpdate (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                voxel_object_id INTEGER NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                z INTEGER NOT NULL,
                color INTEGER NOT NULL,
                username TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (voxel_object_id) REFERENCES VoxelObject (id)
            )",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn add_voxel_object(
        &self,
        name: &str,
        grid_size: usize,
        grid: Vec<u8>,
        voxel_type: &str,
    ) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT INTO VoxelObject (name, grid_size, grid, voxel_type) VALUES (?, ?, ?, ?)",
        )?;
        stmt.execute(params![name, grid_size, grid, voxel_type])?;
        Ok(conn.last_insert_rowid() as usize)
    }

    pub fn add_voxel_update(
        &self,
        voxel_object_id: usize,
        x: usize,
        y: usize,
        z: usize,
        color: u8,
        username: &str,
    ) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT INTO VoxelUpdate (voxel_object_id, x, y, z, color, username, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            voxel_object_id,
            x,
            y,
            z,
            color,
            username,
            timestamp
        ])?;
        Ok(())
    }
}

struct AppState {
    database: Database,
    voxel_objects: HashMap<String, Arc<VoxelObject>>,
}

impl AppState {
    fn new(database: Database) -> Self {
        let voxel_objects = HashMap::new();
        Self {
            database,
            voxel_objects,
        }
    }

    fn add_voxel_object(&mut self, voxel_object: VoxelObject) {
        self.database
            .add_voxel_object(
                &voxel_object.name,
                voxel_object.grid_size,
                voxel_object.grid.read().unwrap().clone(),
                &voxel_object.voxel_type,
            )
            .unwrap();
        let voxel_object_name = voxel_object.name.clone();
        let voxel_object_arc = Arc::new(voxel_object);
        self.voxel_objects
            .insert(voxel_object_name, voxel_object_arc);
    }
}

struct VoxelObject {
    name: String,
    grid_size: usize,
    grid: RwLock<Grid>,
    sessions: Mutex<Vec<Addr<MyWs>>>,
    voxel_type: String,
}

impl VoxelObject {
    fn new(name: &str, grid_size: usize, voxel_type: &str) -> Self {
        let mut rng = rand::thread_rng();
        let mut grid_data: Vec<u8> = Vec::with_capacity(grid_size * grid_size * grid_size);
        for _ in 0..grid_size {
            for y in 0..grid_size {
                for _ in 0..grid_size {
                    if voxel_type == "place_random" {
                        let voxel_spawn_rate =
                            1.0 / (1.0 + ((y as f64 / grid_size as f64) * 16.0 - 1.0).exp());
                        if rng.gen::<f64>() < voxel_spawn_rate {
                            grid_data.push(rng.gen_range(1..=32));
                        } else {
                            grid_data.push(0);
                        }
                    } else {
                        grid_data.push(0);
                    }
                }
            }
        }

        Self {
            name: name.to_string(),
            grid_size,
            grid: RwLock::new(grid_data),
            sessions: Mutex::new(Vec::new()),
            voxel_type: voxel_type.to_string(),
        }
    }

    fn add_session(&self, session: Addr<MyWs>) {
        self.sessions.lock().unwrap().push(session);
    }

    fn broadcast(&self, update_message: UpdateMessage) {
        let sessions = self.sessions.lock().unwrap();
        for session in sessions.iter() {
            session.do_send(update_message);
        }
    }

    fn get_index(&self, x: usize, y: usize, z: usize) -> usize {
        x * self.grid_size * self.grid_size + y * self.grid_size + z
    }
}

#[derive(Message, Clone, Copy)]
#[rtype(result = "()")]
struct UpdateMessage(usize, usize, usize, u8);

struct MyWs {
    voxel_object: Arc<VoxelObject>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.voxel_object.add_session(addr);
    }
}

impl Handler<UpdateMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: UpdateMessage, ctx: &mut Self::Context) {
        ctx.text(
            serde_json::to_string(&json!({
                "type": "update",
                "x": msg.0,
                "y": msg.1,
                "z": msg.2,
                "color": msg.3,
            }))
            .unwrap(),
        );
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("/api/place/{name}/ws")]
async fn ws_index(
    req: HttpRequest,
    stream: Payload,
    state: Data<Mutex<AppState>>,
    path: Path<String>,
) -> Result<HttpResponse, Error> {
    println!("test");
    let name = path.into_inner();
    println!("New websocket connection {:?}", req.connection_info());

    let app_state = state.lock().unwrap();
    let voxel_object = app_state.voxel_objects.get(&name);

    match voxel_object {
        Some(voxel_object) => ws::start(
            MyWs {
                voxel_object: voxel_object.clone(),
            },
            &req,
            stream,
        ),
        None => Err(actix_web::error::ErrorNotFound("No such voxel object")),
    }
}

#[get("/api/place/{name}/all")]
async fn get_grid(data: Data<Mutex<AppState>>, path: Path<String>) -> impl Responder {
    let name = path.into_inner();
    let app_state = data.lock().unwrap();
    let grid = app_state
        .voxel_objects
        .get(&name)
        .unwrap()
        .grid
        .read()
        .unwrap();

    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(&grid).expect("Failed to write data");
    let compressed_data = e.finish().expect("Failed to finish compression");

    HttpResponse::Ok()
        .append_header((header::CONTENT_ENCODING, "gzip"))
        .body(compressed_data)
}

#[post("/api/place/{name}/draw/{x}/{y}/{z}/{color}/{username}")]
async fn draw_voxel(
    data: Data<Mutex<AppState>>,
    path: Path<(String, usize, usize, usize, u8, String)>,
) -> impl Responder {
    let (name, x, y, z, color, username) = path.into_inner();

    let app_state = data.lock().unwrap();
    let voxel_object = app_state.voxel_objects.get(&name).unwrap();
    let mut grid = voxel_object.grid.write().unwrap();
    let at_bottom = y == 0;
    let mut has_neighbor = false;

    if x > 0 && grid[voxel_object.get_index(x - 1, y, z)] > 0 {
        has_neighbor = true;
    }
    if x < voxel_object.grid_size - 1 && grid[voxel_object.get_index(x + 1, y, z)] > 0 {
        has_neighbor = true;
    }
    if y > 0 && grid[voxel_object.get_index(x, y - 1, z)] > 0 {
        has_neighbor = true;
    }
    if y < voxel_object.grid_size - 1 && grid[voxel_object.get_index(x, y + 1, z)] > 0 {
        has_neighbor = true;
    }
    if z > 0 && grid[voxel_object.get_index(x, y, z - 1)] > 0 {
        has_neighbor = true;
    }
    if z < voxel_object.grid_size - 1 && grid[voxel_object.get_index(x, y, z + 1)] > 0 {
        has_neighbor = true;
    }

    if at_bottom || has_neighbor || grid[voxel_object.get_index(x, y, z)] > 0 {
        grid[voxel_object.get_index(x, y, z)] = color;
        voxel_object.broadcast(UpdateMessage(x, y, z, color));
        println!("Updated voxel at {}, {}, {}, by {}", x, y, z, username);
        HttpResponse::Ok().body("OK")
    } else {
        println!("Voxel at {}, {}, {} has no neighbors", x, y, z);
        HttpResponse::BadRequest().body("Voxel has no neighbors")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let param = if args.len() > 1 { &args[1] } else { "" };
    let db = Database::new();
    let app_state = Data::new(Mutex::new(AppState::new(db.unwrap())));

    {
        let mut app_state_guard = app_state.lock().unwrap();
        let temp_voxel_object = VoxelObject::new("temp", 128, param);
        app_state_guard.add_voxel_object(temp_voxel_object);
    }

    println!("Starting server on port 8000");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(app_state.clone())
            .service(get_grid)
            .service(ws_index)
            .service(draw_voxel)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

use std::sync::{RwLock, Mutex};
use actix::{Actor, StreamHandler, Addr, AsyncContext, Handler, Message};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, get, post};
use actix_web_actors::ws;
use actix_cors::Cors;
use rand::Rng;
use serde_json::json;
use rusqlite::{Connection, Result};

type Grid = Vec<u8>;

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
                        let voxel_spawn_rate = 1.0 / (1.0 + ((y as f64 / grid_size as f64) * 16.0 - 1.0).exp());
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
    state: web::Data<VoxelObject>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.state.add_session(addr);
    }
}

impl Handler<UpdateMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: UpdateMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&json!({
            "type": "update",
            "x": msg.0,
            "y": msg.1,
            "z": msg.2,
            "color": msg.3,
        })).unwrap());
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

#[get("/api/place/{name}/ws/")]
async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<VoxelObject>,
) -> Result<HttpResponse, Error> {
    println!("New websocket connection {:?}", req.connection_info());
    ws::start(MyWs { state }, &req, stream)
}

#[get("/api/place/{name}/all")]
async fn get_grid(data: web::Data<VoxelObject>, name: web::Path<String>) -> impl Responder {
    let grid = data.grid.read().unwrap();
    HttpResponse::Ok().body(grid.clone())
}

#[post("/api/place/{name}/draw/{x}/{y}/{z}/{color}/{username}")]
async fn draw_voxel(data: web::Data<VoxelObject>, path: web::Path<(String, usize, usize, usize, u8, String)>) -> impl Responder {
    let (name, x, y, z, color, username) = path.into_inner();

    let mut grid = data.grid.write().unwrap();
    let at_bottom = y == 0;
    let mut has_neighbor = false;

    if x > 0 && grid[data.get_index(x - 1, y, z)] > 0 {
        has_neighbor = true;
    }
    if x < data.grid_size - 1 && grid[data.get_index(x + 1, y, z)] > 0 {
        has_neighbor = true;
    }
    if y > 0 && grid[data.get_index(x, y - 1, z)] > 0 {
        has_neighbor = true;
    }
    if y < data.grid_size - 1 && grid[data.get_index(x, y + 1, z)] > 0 {
        has_neighbor = true;
    }
    if z > 0 && grid[data.get_index(x, y, z - 1)] > 0 {
        has_neighbor = true;
    }
    if z < data.grid_size - 1 && grid[data.get_index(x, y, z + 1)] > 0 {
        has_neighbor = true;
    }

    if at_bottom || has_neighbor || grid[data.get_index(x, y, z)] > 0 {
        grid[data.get_index(x, y, z)] = color;
        data.broadcast(UpdateMessage(x, y, z, color));
        println!("Updated voxel at {}, {}, {}", x, y, z);
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
    let app_state = web::Data::new(VoxelObject::new("Test", 128, param));
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
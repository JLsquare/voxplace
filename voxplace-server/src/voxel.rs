use crate::websocket::PlaceWebSocketConnection;
use actix::{Addr, Message};
use crossbeam::atomic::AtomicCell;
use rand::Rng;
use std::sync::{Mutex};

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
        let grid = grid.unwrap_or_else(|| Voxel::generate_empty_grid(grid_size));

        Self {
            id,
            name: name.to_string(),
            grid_size,
            grid,
            palette_id,
            sessions: Mutex::new(Vec::new()),
            created_at: created_at.unwrap_or_else(|| chrono::Utc::now().timestamp()),
            last_modified_at: last_modified_at.unwrap_or_else(|| chrono::Utc::now().timestamp()),
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

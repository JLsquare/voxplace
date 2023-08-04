use crate::voxel::Voxel;
use std::sync::{Arc, RwLock};
use crate::database::place::PlaceUserUpdate;

pub struct Place {
    pub id: i64,
    pub online: bool,
    pub cooldown: i64,
    pub voxel: Arc<Voxel>,
    pending_updates: RwLock<Vec<PlaceUserUpdate>>,
}

impl Place {
    pub fn new(id: i64, online: bool, cooldown: i64, voxel: Voxel) -> Self {
        Self {
            id,
            online,
            cooldown,
            voxel: Arc::new(voxel),
            pending_updates: RwLock::new(Vec::new()),
        }
    }

    pub fn add_place_update(&self, x: usize, y: usize, z: usize, user_id: i64) -> () {
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
}

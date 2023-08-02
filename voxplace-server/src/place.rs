use crate::voxel::Voxel;
use std::sync::Arc;

pub struct Place {
    pub id: i64,
    pub online: bool,
    pub voxel: Arc<Voxel>,
}

impl Place {
    pub fn new(id: i64, online: bool, voxel: Voxel) -> Self {
        Self {
            id,
            online,
            voxel: Arc::new(voxel),
        }
    }
}

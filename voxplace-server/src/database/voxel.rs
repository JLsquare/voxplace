use crate::database::db::Database;
use crate::voxel::Voxel;
use rusqlite::{params, Error};

pub struct VoxelInfo {
    pub voxel_id: i64,
    pub name: String,
    pub path: String,
    pub version: i64,
    pub palette_size: i64,
    pub grid_size_x: i64,
    pub grid_size_y: i64,
    pub grid_size_z: i64,
    pub compression: String,
    pub created_at: i64,
    pub last_modified_at: i64,
}

impl Database {
    pub fn create_voxel_table(&self) -> Result<(), Error> {
        self.conn.lock().unwrap().execute(
            "CREATE TABLE IF NOT EXISTS Voxel (
                voxel_id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                version INTEGER NOT NULL,
                palette_size INTEGER NOT NULL,
                size_x INTEGER NOT NULL,
                size_y INTEGER NOT NULL,
                size_z INTEGER NOT NULL,
                compression TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                last_modified_at DATETIME NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn save_new_voxel(&self, voxel_object: &Voxel) -> Result<(), Error> {
        let version = 1;
        let compression = "gzip";
        let path = format!("voxels/{}.vxl", voxel_object.id);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO Voxel (
                voxel_id,
                name,
                path,
                version,
                palette_size,
                size_x,
                size_y,
                size_z,
                compression,
                created_at,
                last_modified_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            voxel_object.id,
            voxel_object.name,
            path,
            version,
            voxel_object.palette.len(),
            voxel_object.grid_size.0,
            voxel_object.grid_size.1,
            voxel_object.grid_size.2,
            compression,
            timestamp,
            timestamp,
        ])?;
        Ok(())
    }

    pub fn get_voxel_info(&self, id: i64) -> Result<VoxelInfo, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT
                voxel_id,
                name,
                path,
                version,
                palette_size,
                size_x,
                size_y,
                size_z,
                compression,
                created_at,
                last_modified_at
                FROM Voxel WHERE voxel_id = ?1",
        )?;

        let voxel_object = stmt.query_row(params![id], |row| {
            Ok(VoxelInfo {
                voxel_id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                version: row.get(3)?,
                palette_size: row.get(4)?,
                grid_size_x: row.get(5)?,
                grid_size_y: row.get(6)?,
                grid_size_z: row.get(7)?,
                compression: row.get(8)?,
                created_at: row.get(9)?,
                last_modified_at: row.get(10)?,
            })
        })?;

        Ok(voxel_object)
    }
}

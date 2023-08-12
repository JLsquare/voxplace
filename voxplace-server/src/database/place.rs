use crate::database::db::{Database, DatabaseError};
use crate::place::Place;
use rusqlite::params;
use serde_derive::Serialize;

#[derive(Clone, Serialize, Debug)]
pub struct PlaceUserUpdate {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub user_id: i64,
    pub place_id: i64,
}

#[derive(Debug, Serialize)]
pub struct PlaceInfo {
    pub place_id: String,
    pub online: bool,
    pub cooldown: i64,
    pub voxel_id: String,
    pub name: String,
    pub size_x: i64,
    pub size_y: i64,
    pub size_z: i64,
}

impl Database {
    pub fn create_place_table(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Place (
                place_id INTEGER PRIMARY KEY,
                online INTEGER NOT NULL,
                cooldown INTEGER NOT NULL,
                voxel_id INTEGER NOT NULL,
                FOREIGN KEY (voxel_id) REFERENCES Voxel (id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_place_user_table(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS PlaceUser (
                place_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                z INTEGER NOT NULL,
                PRIMARY KEY (place_id, x, y, z)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_place_user_cooldown_table(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS PlaceUserCooldown (
                place_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                cooldown INTEGER NOT NULL,
                PRIMARY KEY (place_id, user_id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn get_user_cooldown(&self, place_id: i64, user_id: i64) -> Result<i64, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT
                cooldown
                FROM PlaceUserCooldown
                WHERE place_id = ? AND user_id = ?",
        )?;
        let mut rows = stmt.query(params![place_id, user_id])?;

        if let Some(row) = rows.next()? {
            let cooldown: i64 = row.get(0)?;
            return Ok(cooldown);
        } else {
            return Err(DatabaseError::NoSuchUser());
        }
    }

    pub fn set_user_cooldown(&self, place_id: i64, user_id: i64, cooldown: i64) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO PlaceUserCooldown (
                place_id,
                user_id,
                cooldown
            ) VALUES (?, ?, ?)",
        )?;
        stmt.execute(params![place_id, user_id, cooldown])?;

        Ok(())
    }

    pub fn save_new_place(&self, place: &Place) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT INTO Place (
                place_id,
                online,
                cooldown,
                voxel_id
            ) VALUES (?, ?, ?, ?)",
        )?;
        stmt.execute(params![place.id, place.online, place.cooldown, place.voxel.id])?;

        Ok(())
    }

    pub fn save_places_users(&self, updates: Vec<PlaceUserUpdate>) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO PlaceUser (
                place_id,
                user_id,
                x,
                y,
                z
            ) VALUES (?, ?, ?, ?, ?)",
        )?;

        for update in updates {
            stmt.execute(params![
                update.place_id,
                update.user_id,
                update.x,
                update.y,
                update.z,
            ])?;
        }

        Ok(())
    }

    pub fn get_places_infos(&self) -> Result<Vec<PlaceInfo>, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT
                Place.place_id,
                Place.online,
                Place.cooldown,
                Voxel.voxel_id,
                Voxel.name,
                Voxel.size_x,
                Voxel.size_y,
                Voxel.size_z
                FROM Place
                INNER JOIN Voxel ON Place.voxel_id = Voxel.voxel_id",
        )?;
        let mut rows = stmt.query(params![])?;
        let mut places = Vec::new();
        while let Some(row) = rows.next()? {
            places.push(PlaceInfo {
                place_id: row.get::<_, i64>(0)?.to_string(),
                online: row.get(1)?,
                cooldown: row.get(2)?,
                voxel_id: row.get::<_, i64>(3)?.to_string(),
                name: row.get(4)?,
                size_x: row.get(5)?,
                size_y: row.get(6)?,
                size_z: row.get(7)?,
            });
        }
        Ok(places)
    }


    pub fn get_place_user(&self, place_id: i64, x: i64, y: i64, z: i64) -> Result<i64, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT
            user_id
            FROM PlaceUser WHERE place_id = ?1 AND x = ?2 AND y = ?3 AND z = ?4",
        )?;

        let user_id = stmt.query_row(params![place_id, x, y, z], |row| row.get(0))?;

        Ok(user_id)
    }
}

use crate::database::db::Database;
use crate::place::Place;
use rusqlite::{params, Error};
use serde_derive::Serialize;

#[derive(Clone, Serialize)]
pub struct PlaceUserUpdate {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub user_id: i64,
    pub place_id: i64,
}

impl Database {
    pub fn create_place_table(&self) -> Result<(), Error> {
        self.conn.lock().unwrap().execute(
            "CREATE TABLE IF NOT EXISTS Place (
                place_id INTEGER PRIMARY KEY,
                online INTEGER NOT NULL,
                voxel_id INTEGER NOT NULL,
                FOREIGN KEY (voxel_id) REFERENCES Voxel (id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_place_user_table(&self) -> Result<(), Error> {
        self.conn.lock().unwrap().execute(
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

    pub fn save_new_place(&self, place: &Place) -> Result<(), Error> {
        println!("Saving place: {}", place.id);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT INTO Place (
                place_id,
                online,
                voxel_id
            ) VALUES (?, ?, ?)",
        )?;
        stmt.execute(params![place.id, place.online, place.voxel.id])?;

        Ok(())
    }

    pub fn save_places_users(&self, updates: Vec<PlaceUserUpdate>) -> Result<(), Error> {
        let conn = self.conn.lock().unwrap();
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

    pub fn get_place_infos(&self, place_id: i64) -> Result<(bool, i64), Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT
                online,
                voxel_id
                FROM Place
                WHERE id = ?",
        )?;
        let mut rows = stmt.query(params![place_id])?;
        if let Some(row) = rows.next()? {
            Ok((row.get(0)?, row.get(1)?))
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }

    pub fn get_places_infos(&self) -> Result<Vec<(i64, bool, i64, String, i64, i64, i64)>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT
            Place.place_id,
            Place.online,
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
            places.push((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ));
        }
        Ok(places)
    }

    pub fn get_place_user(&self, place_id: i64, x: i64, y: i64, z: i64) -> Result<i64, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT
            user_id
            FROM PlaceUser WHERE place_id = ?1 AND x = ?2 AND y = ?3 AND z = ?4",
        )?;

        let user_id = stmt.query_row(params![place_id, x, y, z], |row| Ok(row.get(0)?))?;

        Ok(user_id)
    }
}

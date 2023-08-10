use std::io::{Read, Write};
use crossbeam::atomic::AtomicCell;
use flate2::Compression;
use flate2::write::GzEncoder;
use crate::database::db::{Database, DatabaseError};
use crate::voxel::Voxel;
use rusqlite::params;
use crate::palette::Palette;

impl Database {
    pub fn create_voxel_table(&self) -> rusqlite::Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Voxel (
                voxel_id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                palette_id INTEGER NOT NULL,
                size_x INTEGER NOT NULL,
                size_y INTEGER NOT NULL,
                size_z INTEGER NOT NULL,
                created_at DATETIME NOT NULL,
                last_modified_at DATETIME NOT NULL,
                grid BLOB NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_palette_table(&self) -> rusqlite::Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Palette (
                palette_id INTEGER PRIMARY KEY,
                colors BLOB NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn save_new_palette(&self, palette: Palette) -> rusqlite::Result<(), DatabaseError> {
        let mut bytes: Vec<u8> = Vec::new();
        for color in palette.colors().iter() {
            bytes.push(color.0);
            bytes.push(color.1);
            bytes.push(color.2);
        }

        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO Palette (
                palette_id,
                colors
            ) VALUES (?, ?)",
        )?;
        stmt.execute(params![
            palette.palette_id(),
            bytes
        ])?;

        Ok(())
    }

    pub fn get_palette(&self, palette_id: i64) -> rusqlite::Result<Vec<(u8, u8, u8)>, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT colors FROM Palette WHERE palette_id = ?",
        )?;
        let mut rows = stmt.query(params![palette_id])?;
        let row = rows.next()?.ok_or(DatabaseError::NoSuchPalette())?;
        let colors: Vec<u8> = row.get(0)?;
        let mut result = Vec::new();
        for i in 0..colors.len() / 3 {
            result.push((colors[i * 3], colors[i * 3 + 1], colors[i * 3 + 2]));
        }
        Ok(result)
    }

    pub fn save_new_voxel(&self, voxel_object: &Voxel) -> rusqlite::Result<(), DatabaseError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::new(0, 0))
            .as_secs() as i64;

        let mut bytes = Vec::new();
        let grid: Vec<u8> = voxel_object.grid.iter().map(|cell| cell.load()).collect();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&grid).expect("Failed to write data");
        let compressed_data = encoder.finish().expect("Failed to finish compression");
        bytes.extend(compressed_data);

        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO Voxel (
                voxel_id,
                name,
                palette_id,
                size_x,
                size_y,
                size_z,
                created_at,
                last_modified_at,
                grid
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            voxel_object.id,
            voxel_object.name,
            voxel_object.palette_id,
            voxel_object.grid_size.0,
            voxel_object.grid_size.1,
            voxel_object.grid_size.2,
            timestamp,
            timestamp,
            bytes
        ])?;
        Ok(())
    }

    pub fn get_voxel(&self, id: i64) -> rusqlite::Result<Voxel, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT
                voxel_id,
                name,
                palette_id,
                size_x,
                size_y,
                size_z,
                grid,
                created_at,
                last_modified_at
                FROM Voxel WHERE voxel_id = ?1",
        )?;
        let row: (_, _, _, _, _, _, _, _, _) = stmt.query_row(params![id], |row| {
            Ok((
                row.get(0)?,
                row.get::<_, String>(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get::<_, Vec<u8>>(6)?,
                row.get(7)?,
                row.get(8)?,
            ))
        })?;

        let compressed_grid = row.6;
        let grid: Vec<AtomicCell<u8>> = Self::decompress_grid(&compressed_grid)?
            .into_iter()
            .map(AtomicCell::new)
            .collect();
        let grid = Some(grid);

        Ok(Voxel::new(
            row.0,
            &row.1,
            row.2,
            (row.3, row.4, row.5),
            grid,
            row.7,
            row.8,
        ))
    }

    pub fn save_voxel_grid(&self, id: i64, grid: Vec<u8>) -> rusqlite::Result<(), DatabaseError> {
        let compressed_data = Self::compress_grid(&grid)?;
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.prepare("UPDATE Voxel SET grid = ?1 WHERE voxel_id = ?2")?
            .execute(params![compressed_data, id])?;
        Ok(())
    }

    fn compress_grid(grid: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(grid)?;
        encoder.finish()
    }

    fn decompress_grid(data: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut grid = Vec::new();
        decoder.read_to_end(&mut grid)?;
        Ok(grid)
    }
}

use rusqlite::params;
use crate::database::db::{Database, DatabaseError};
use crate::palette::Palette;

impl Database {
    pub fn create_palette_table(&self) -> rusqlite::Result<(), DatabaseError> {
        self.get_conn()?.execute(
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

        let conn = self.get_conn()?;
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
        let conn = self.get_conn()?;
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
}
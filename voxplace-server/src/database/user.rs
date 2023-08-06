use crate::database::db::Database;
use bcrypt::verify;
use rusqlite::{params, Error};
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct UserProfile {
    pub user_id: String,
    pub username: String,
    pub voxel_id: String,
    pub xp: i64,
    pub created_at: i64,
    pub last_connected_at: i64,
}

impl Database {
    pub fn create_user_table(&self) -> Result<(), Error> {
        self.conn.lock().unwrap().execute(
            "CREATE TABLE IF NOT EXISTS User (
                user_id INTEGER PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                email TEXT NOT NULL,
                voxel_id INTEGER NOT NULL,
                xp INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME NOT NULL,
                last_connected_at DATETIME NOT NULL,
                admin INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        Ok(())
    }

    pub fn register_user(
        &self,
        user_id: i64,
        username: &str,
        email: &str,
        voxel_id: i64,
        password_hash: &str,
        created_at: i64,
        last_connected_at: i64,
    ) -> Result<i64, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO User (
                user_id,
                username,
                password_hash,
                email,
                voxel_id,
                created_at,
                last_connected_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            user_id,
            username,
            password_hash,
            email,
            voxel_id,
            created_at,
            last_connected_at
        ])?;

        Ok(conn.last_insert_rowid())
    }

    pub fn login_user(&self, username: &str, password: &str) -> Result<i64, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT user_id, password_hash FROM User WHERE username = ?")?;
        let mut rows = stmt.query(params![username])?;

        if let Some(row) = rows.next()? {
            let user_id: i64 = row.get(0)?;
            let password_hash: String = row.get(1)?;

            if verify(password, &password_hash).unwrap_or(false) {
                Ok(user_id)
            } else {
                Err(Error::InvalidQuery)
            }
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }

    pub fn is_admin(&self, user_id: i64) -> Result<bool, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT admin FROM User WHERE user_id = ?")?;
        let mut rows = stmt.query(params![user_id])?;

        if let Some(row) = rows.next()? {
            let admin: i64 = row.get(0)?;

            Ok(admin == 1)
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }

    pub fn update_username(&self, user_id: i64, new_username: &str) -> Result<(), Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("UPDATE User SET username = ? WHERE user_id = ?")?;
        stmt.execute(params![new_username, user_id])?;
        Ok(())
    }

    pub fn update_email(&self, user_id: i64, new_email: &str) -> Result<(), Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("UPDATE User SET email = ? WHERE user_id = ?")?;
        stmt.execute(params![new_email, user_id])?;
        Ok(())
    }

    pub fn update_password(&self, user_id: i64, new_password_hash: &str) -> Result<(), Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("UPDATE User SET password_hash = ? WHERE user_id = ?")?;
        stmt.execute(params![new_password_hash, user_id])?;
        Ok(())
    }

    pub fn update_last_connected_at(
        &self,
        user_id: i64,
        new_last_connected_at: i64,
    ) -> Result<(), Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("UPDATE User SET last_connected_at = ? WHERE user_id = ?")?;
        stmt.execute(params![new_last_connected_at, user_id])?;
        Ok(())
    }

    pub fn get_username(&self, user_id: i64) -> Result<String, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT username FROM User WHERE user_id = ?")?;
        let mut rows = stmt.query(params![user_id])?;
        if let Some(row) = rows.next()? {
            let username: String = row.get(0)?;
            Ok(username)
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }

    pub fn get_xp(&self, user_id: i64) -> Result<i64, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT xp FROM User WHERE user_id = ?")?;
        let mut rows = stmt.query(params![user_id])?;
        if let Some(row) = rows.next()? {
            let xp: i64 = row.get(0)?;
            Ok(xp)
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }

    pub fn get_user_profile(&self, user_id: i64) -> Result<UserProfile, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT user_id, username, voxel_id, xp, created_at, last_connected_at FROM User WHERE user_id = ?")?;
        let mut rows = stmt.query(params![user_id])?;
        if let Some(row) = rows.next()? {
            let user_id: i64 = row.get(0)?;
            let username: String = row.get(1)?;
            let voxel_id: i64 = row.get(2)?;
            let xp: i64 = row.get(3)?;
            let created_at: i64 = row.get(4)?;
            let last_connected_at: i64 = row.get(5)?;
            Ok(UserProfile {
                user_id: user_id.to_string(),
                username,
                voxel_id: voxel_id.to_string(),
                xp,
                created_at,
                last_connected_at,
            })
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }
}

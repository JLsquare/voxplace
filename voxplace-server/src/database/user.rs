use crate::database::db::{Database, DatabaseError};
use bcrypt::verify;
use rusqlite::params;
use serde_derive::Serialize;
use crate::user::User;

#[derive(Serialize)]
pub struct UserProfile {
    pub user_id: String,
    pub username: String,
    pub voxel_id: String,
    pub xp: i64,
    pub created_at: i64,
    pub last_connected_at: i64,
}

#[derive(Serialize)]
pub struct FullUserProfile {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub voxel_id: String,
    pub xp: i64,
    pub created_at: i64,
    pub last_connected_at: i64,
}

impl Database {
    pub fn create_user_table(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.execute(
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
        user: User,
    ) -> Result<i64, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
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
            user.user_id,
            user.username,
            user.password_hash,
            user.email,
            user.voxel_id,
            user.created_at,
            user.last_connected_at
        ])?;

        Ok(conn.last_insert_rowid())
    }

    pub fn login_user(&self, username: &str, password: &str) -> Result<i64, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT user_id, password_hash FROM User WHERE username = ?")?;
        let mut rows = stmt.query(params![username])?;

        if let Some(row) = rows.next()? {
            let user_id: i64 = row.get(0)?;
            let password_hash: String = row.get(1)?;

            if verify(password, &password_hash).unwrap_or(false) {
                Ok(user_id)
            } else {
                Err(DatabaseError::InvalidPassword(username.to_string()))
            }
        } else {
            Err(DatabaseError::NoSuchUser())
        }
    }

    pub fn is_admin(&self, user_id: i64) -> Result<bool, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT admin FROM User WHERE user_id = ?")?;
        let mut rows = stmt.query(params![user_id])?;

        if let Some(row) = rows.next()? {
            let admin: i64 = row.get(0)?;

            Ok(admin == 1)
        } else {
            Err(DatabaseError::NoSuchUser())
        }
    }

    pub fn update_username(&self, user_id: i64, new_username: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("UPDATE User SET username = ? WHERE user_id = ?")?;
        stmt.execute(params![new_username, user_id])?;
        Ok(())
    }

    pub fn update_email(&self, user_id: i64, new_email: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("UPDATE User SET email = ? WHERE user_id = ?")?;
        stmt.execute(params![new_email, user_id])?;
        Ok(())
    }

    pub fn update_password(&self, user_id: i64, new_password_hash: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("UPDATE User SET password_hash = ? WHERE user_id = ?")?;
        stmt.execute(params![new_password_hash, user_id])?;
        Ok(())
    }

    pub fn update_last_connected_at(&self, user_id: i64, new_last_connected_at: i64) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("UPDATE User SET last_connected_at = ? WHERE user_id = ?")?;
        stmt.execute(params![new_last_connected_at, user_id])?;
        Ok(())
    }

    pub fn get_username(&self, user_id: i64) -> Result<String, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT username FROM User WHERE user_id = ?")?;
        let mut rows = stmt.query(params![user_id])?;
        if let Some(row) = rows.next()? {
            let username: String = row.get(0)?;
            Ok(username)
        } else {
            Err(DatabaseError::NoSuchUser())
        }
    }

    pub fn get_user_profile(&self, user_id: i64) -> Result<UserProfile, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
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
            Err(DatabaseError::NoSuchUser())
        }
    }

    pub fn get_full_user_profile(&self, user_id: i64) -> Result<FullUserProfile, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT user_id, username, voxel_id, xp, created_at, last_connected_at, email FROM User WHERE user_id = ?")?;
        let mut rows = stmt.query(params![user_id])?;
        if let Some(row) = rows.next()? {
            let user_id: i64 = row.get(0)?;
            let username: String = row.get(1)?;
            let voxel_id: i64 = row.get(2)?;
            let xp: i64 = row.get(3)?;
            let created_at: i64 = row.get(4)?;
            let last_connected_at: i64 = row.get(5)?;
            let email: String = row.get(6)?;
            Ok(FullUserProfile {
                user_id: user_id.to_string(),
                username,
                voxel_id: voxel_id.to_string(),
                xp,
                created_at,
                last_connected_at,
                email,
            })
        } else {
            Err(DatabaseError::NoSuchUser())
        }
    }

    pub fn get_top_users(&self, limit: i64) -> Result<Vec<UserProfile>, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT user_id, username, voxel_id, xp, created_at, last_connected_at FROM User ORDER BY xp DESC LIMIT ?")?;
        let mut rows = stmt.query(params![limit])?;
        let mut users = Vec::new();
        while let Some(row) = rows.next()? {
            let user_id: i64 = row.get(0)?;
            let username: String = row.get(1)?;
            let voxel_id: i64 = row.get(2)?;
            let xp: i64 = row.get(3)?;
            let created_at: i64 = row.get(4)?;
            let last_connected_at: i64 = row.get(5)?;
            users.push(UserProfile {
                user_id: user_id.to_string(),
                username,
                voxel_id: voxel_id.to_string(),
                xp,
                created_at,
                last_connected_at,
            });
        }
        Ok(users)
    }
}

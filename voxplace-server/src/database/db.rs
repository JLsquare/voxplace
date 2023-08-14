use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error during IO: {0}")]
    CompressionError(#[from] std::io::Error),

    #[error("Error during database operation: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Invalid password for user: {0}")]
    InvalidPassword(String),

    #[error("No such user")]
    NoSuchUser(),

    #[error("No such palette")]
    NoSuchPalette(),

    #[error("No such post")]
    NoSuchPost(),

    #[error("Error during database lock: {0}")]
    LockError(String),

    #[error("Invalid vote")]
    InvalidVote(),
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, DatabaseError> {
        let conn = Connection::open("database.db")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init(&self) {
        self.create_palette_table().unwrap();
        self.create_voxel_table().unwrap();
        self.create_place_table().unwrap();
        self.create_user_table().unwrap();
        self.create_place_user_table().unwrap();
        self.create_place_user_cooldown_table().unwrap();
        self.create_user_voxel_table().unwrap();
        self.create_post_table().unwrap();
        self.create_comment_table().unwrap();
    }

    pub fn get_conn(&self) -> Result<MutexGuard<Connection>, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        Ok(conn)
    }
}

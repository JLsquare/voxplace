use rusqlite::{Connection, Error};
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, Error> {
        let conn = Connection::open("database.db")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init(&self) {
        self.create_user_table().unwrap();
        self.create_voxel_table().unwrap();
        self.create_place_table().unwrap();
        self.create_place_user_table().unwrap();
    }
}

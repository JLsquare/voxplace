use rusqlite::params;
use crate::database::db::{Database, DatabaseError};
use crate::post::Post;

impl Database {
    pub fn create_post_table(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Post (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                voxel_id INTEGER NOT NULL,
                votes INTEGER NOT NULL DEFAULT 0,
                author_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (author_id) REFERENCES user(id)
                FOREIGN KEY (voxel_id) REFERENCES voxel(id)
            )",
            [],
        )?;
        Ok(())
    }

    pub fn save_new_post(&self, post: Post) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT INTO post (id, title, content, voxel_id, author_id, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            post.id,
            post.title,
            post.content,
            post.voxel_id,
            post.author_id,
            post.created_at,
        ])?;
        Ok(())
    }

    pub fn get_top_posts(&self, limit: i64) -> Result<Vec<Post>, DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, voxel_id, votes, author_id, created_at FROM post ORDER BY votes DESC LIMIT ?",
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(Post {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                voxel_id: row.get(3)?,
                votes: row.get(4)?,
                author_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn vote_post(&self, post_id: i64, vote: i64) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|e| DatabaseError::LockError(e.to_string()))?;
        conn.execute(
            "UPDATE post SET votes = votes + ? WHERE id = ?",
            [
                &vote,
                &post_id,
            ],
        )?;
        Ok(())
    }
}
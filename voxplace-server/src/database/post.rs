use rusqlite::params;
use serde_derive::Serialize;
use crate::database::db::{Database, DatabaseError};
use crate::post::Post;

#[derive(Serialize)]
pub struct PostInfo {
    pub post_id: String,
    pub title: String,
    pub content: String,
    pub voxel_id: String,
    pub votes: i64,
    pub author_id: String,
    pub updated: bool,
}

impl Database {
    pub fn create_post_table(&self) -> Result<(), DatabaseError> {
        self.get_conn()?.execute(
            "CREATE TABLE IF NOT EXISTS Post (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                voxel_id INTEGER NOT NULL,
                votes INTEGER NOT NULL DEFAULT 0,
                author_id INTEGER NOT NULL,
                updated INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (author_id) REFERENCES user(id)
                FOREIGN KEY (voxel_id) REFERENCES voxel(id)
            )",
            [],
        )?;
        Ok(())
    }

    pub fn create_vote_table(&self) -> Result<(), DatabaseError> {
        self.get_conn()?.execute(
            "CREATE TABLE IF NOT EXISTS Vote (
                user_id INTEGER NOT NULL,
                post_id INTEGER NOT NULL,
                vote INTEGER NOT NULL,
                PRIMARY KEY (user_id, post_id),
                FOREIGN KEY (user_id) REFERENCES user(id),
                FOREIGN KEY (post_id) REFERENCES post(id)
            )",
            [],
        )?;
        Ok(())
    }

    pub fn save_new_post(&self, post: Post) -> Result<(), DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "INSERT INTO post (id, title, content, voxel_id, author_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            post.id,
            post.title,
            post.content,
            post.voxel_id,
            post.author_id,
            post.created_at,
            post.updated_at,
        ])?;
        Ok(())
    }

    pub fn get_top_posts(&self, limit: i64) -> Result<Vec<PostInfo>, DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, voxel_id, votes, author_id, updated FROM post ORDER BY votes DESC LIMIT ?",
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(PostInfo {
                post_id: row.get::<_, i64>(0)?.to_string(),
                title: row.get(1)?,
                content: row.get(2)?,
                voxel_id: row.get::<_, i64>(3)?.to_string(),
                votes: row.get(4)?,
                author_id: row.get::<_, i64>(5)?.to_string(),
                updated: row.get::<_, i64>(6)? == 1,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_top_user_posts(&self, user_id: i64, limit: i64) -> Result<Vec<PostInfo>, DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, voxel_id, votes, author_id, updated FROM post WHERE author_id = ? ORDER BY votes DESC LIMIT ?",
        )?;
        let rows = stmt.query_map([user_id, limit], |row| {
            Ok(PostInfo {
                post_id: row.get::<_, i64>(0)?.to_string(),
                title: row.get(1)?,
                content: row.get(2)?,
                voxel_id: row.get::<_, i64>(3)?.to_string(),
                votes: row.get(4)?,
                author_id: row.get::<_, i64>(5)?.to_string(),
                updated: row.get::<_, i64>(6)? == 1,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_new_posts(&self, limit: i64) -> Result<Vec<PostInfo>, DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, voxel_id, votes, author_id, updated FROM post ORDER BY created_at DESC LIMIT ?",
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(PostInfo {
                post_id: row.get::<_, i64>(0)?.to_string(),
                title: row.get(1)?,
                content: row.get(2)?,
                voxel_id: row.get::<_, i64>(3)?.to_string(),
                votes: row.get(4)?,
                author_id: row.get::<_, i64>(5)?.to_string(),
                updated: row.get::<_, i64>(6)? == 1,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub  fn get_new_user_posts(&self, user_id: i64, limit: i64) -> Result<Vec<PostInfo>, DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, voxel_id, votes, author_id, updated FROM post WHERE author_id = ? ORDER BY created_at DESC LIMIT ?",
        )?;
        let rows = stmt.query_map([user_id, limit], |row| {
            Ok(PostInfo {
                post_id: row.get::<_, i64>(0)?.to_string(),
                title: row.get(1)?,
                content: row.get(2)?,
                voxel_id: row.get::<_, i64>(3)?.to_string(),
                votes: row.get(4)?,
                author_id: row.get::<_, i64>(5)?.to_string(),
                updated: row.get::<_, i64>(6)? == 1,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_post(&self, post_id: i64) -> Result<PostInfo, DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, voxel_id, votes, author_id, updated FROM post WHERE id = ?",
        )?;
        let mut rows = stmt.query_map([post_id], |row| {
            Ok(PostInfo {
                post_id: row.get::<_, i64>(0)?.to_string(),
                title: row.get(1)?,
                content: row.get(2)?,
                voxel_id: row.get::<_, i64>(3)?.to_string(),
                votes: row.get(4)?,
                author_id: row.get::<_, i64>(5)?.to_string(),
                updated: row.get::<_, i64>(6)? == 1,
            })
        })?;
        if let Some(row) = rows.next() {
            Ok(row?)
        } else {
            Err(DatabaseError::NoSuchPost())
        }
    }

    pub fn vote_post(&self, post_id: i64, vote: i64) -> Result<(), DatabaseError> {
        if vote != 1 && vote != -1 {
            return Err(DatabaseError::InvalidVote());
        }
        self.get_conn()?.execute(
            "UPDATE post SET votes = votes + ? WHERE id = ?",
            params![vote, post_id],
        )?;
        Ok(())
    }
}
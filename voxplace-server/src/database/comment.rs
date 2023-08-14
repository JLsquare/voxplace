use rusqlite::params;
use serde_derive::Serialize;
use crate::comment::Comment;
use crate::database::db::{Database, DatabaseError};

#[derive(Serialize)]
pub struct PostComment {
    pub comment_id: String,
    pub user_id: String,
    pub username: String,
    pub post_id: String,
    pub content: String,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct PlaceComment {
    pub comment_id: String,
    pub user_id: String,
    pub username: String,
    pub place_id: String,
    pub content: String,
    pub created_at: i64,
}

impl Database {
    pub fn create_comment_table(&self) -> Result<(), DatabaseError> {
        self.get_conn()?.execute(
            "CREATE TABLE IF NOT EXISTS Comment (
                comment_id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL,
                place_id INTEGER,
                post_id INTEGER,
                content TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                FOREIGN KEY (user_id) REFERENCES User (id),
                FOREIGN KEY (place_id) REFERENCES Place (id),
                FOREIGN KEY (post_id) REFERENCES Post (id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn save_new_comment(&self, comment: Comment) -> Result<(), DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "INSERT INTO Comment (comment_id, user_id, place_id, post_id, content, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            comment.comment_id,
            comment.user_id,
            comment.place_id,
            comment.post_id,
            comment.content,
            comment.created_at,
        ])?;
        Ok(())
    }

    pub fn get_comments_by_post_id(&self, post_id: i64) -> Result<Vec<PostComment>, DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT c.comment_id, c.user_id, c.post_id, c.content, c.created_at, u.username
             FROM Comment AS c
             JOIN User AS u ON c.user_id = u.user_id
             WHERE c.post_id = ?",
        )?;
        let rows = stmt.query_map(params![post_id], |row| {
            Ok(PostComment {
                comment_id: row.get::<_, i64>(0)?.to_string(),
                user_id: row.get::<_, i64>(1)?.to_string(),
                post_id: row.get::<_, i64>(2)?.to_string(),
                content: row.get(3)?,
                created_at: row.get(4)?,
                username: row.get(5)?,
            })
        })?;

        let mut comments = Vec::new();
        for comment in rows {
            comments.push(comment?);
        }

        Ok(comments)
    }

    pub fn get_comments_by_place_id(&self, place_id: i64) -> Result<Vec<PlaceComment>, DatabaseError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT c.comment_id, c.user_id, c.place_id, c.content, c.created_at, u.username
             FROM Comment AS c
             JOIN User AS u ON c.user_id = u.user_id
             WHERE c.place_id = ?",
        )?;
        let rows = stmt.query_map(params![place_id], |row| {
            Ok(PlaceComment {
                comment_id: row.get::<_, i64>(0)?.to_string(),
                user_id: row.get::<_, i64>(1)?.to_string(),
                place_id: row.get::<_, i64>(2)?.to_string(),
                content: row.get(3)?,
                created_at: row.get(4)?,
                username: row.get(5)?,
            })
        })?;

        let mut comments = Vec::new();
        for comment in rows {
            comments.push(comment?);
        }

        Ok(comments)
    }
}
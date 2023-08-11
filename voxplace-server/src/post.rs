pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub voxel_id: i64,
    pub votes: i64,
    pub author_id: i64,
    pub updated: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Post {
    pub fn new(
        id: i64,
        title: &str,
        content: &str,
        voxel_id: i64,
        votes: i64,
        author_id: i64,
        updated: bool,
        created_at: i64,
        updated_at: i64,
    ) -> Self {
        Self {
            id,
            title: title.to_string(),
            content: content.to_string(),
            voxel_id,
            votes,
            author_id,
            updated,
            created_at,
            updated_at,
        }
    }
}
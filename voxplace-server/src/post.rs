pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub voxel_id: i64,
    pub votes: i64,
    pub author_id: i64,
    pub created_at: i64,
}

impl Post {
    pub fn new(
        id: i64,
        title: String,
        content: String,
        voxel_id: i64,
        votes: i64,
        author_id: i64,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            title,
            content,
            voxel_id,
            votes,
            author_id,
            created_at,
        }
    }
}
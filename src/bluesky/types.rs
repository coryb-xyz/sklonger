use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Thread {
    pub posts: Vec<ThreadPost>,
    pub author: Author,
}

#[derive(Debug, Clone)]
pub struct ThreadPost {
    pub uri: String,
    pub cid: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub reply_count: Option<u32>,
    pub repost_count: Option<u32>,
    pub like_count: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Author {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

impl Thread {
    pub fn original_post_url(&self) -> Option<String> {
        self.posts.first().map(|post| {
            let post_id = post.uri.rsplit('/').next().unwrap_or("");
            format!(
                "https://bsky.app/profile/{}/post/{}",
                self.author.handle, post_id
            )
        })
    }
}

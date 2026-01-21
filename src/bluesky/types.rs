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
    pub embed: Option<Embed>,
}

#[derive(Debug, Clone)]
pub enum Embed {
    Images(Vec<EmbedImage>),
    Video(EmbedVideo),
    External(EmbedExternal),
}

#[derive(Debug, Clone)]
pub struct EmbedImage {
    pub thumb_url: String,
    pub fullsize_url: String,
    pub alt: String,
    pub aspect_ratio: Option<AspectRatio>,
}

#[derive(Debug, Clone)]
pub struct EmbedVideo {
    pub thumbnail_url: Option<String>,
    pub playlist_url: String,
    pub alt: Option<String>,
    pub aspect_ratio: Option<AspectRatio>,
}

#[derive(Debug, Clone)]
pub struct EmbedExternal {
    pub uri: String,
    pub title: String,
    pub description: String,
    pub thumb_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
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

use std::sync::Arc;
use std::time::Duration;

use atrium_api::app::bsky::feed::defs::{
    PostViewEmbedRefs, ThreadViewPost, ThreadViewPostParentRefs, ThreadViewPostRepliesItem,
};
use atrium_api::app::bsky::feed::get_post_thread::{OutputThreadRefs, ParametersData};
use atrium_api::client::AtpServiceClient;
use atrium_api::types::Union;
use atrium_xrpc_client::reqwest::ReqwestClient;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::warn;

use super::types::{
    AspectRatio, Author, Embed, EmbedExternal, EmbedImage, EmbedRecord, EmbedVideo, StreamEvent,
    Thread, ThreadPost,
};

/// Parse a `createdAt` field from a JSON value, falling back to UNIX_EPOCH with a warning.
fn parse_created_at(value: &serde_json::Value, context: &str) -> DateTime<Utc> {
    value
        .get("createdAt")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| {
            warn!("failed to parse createdAt from {}, using epoch", context);
            DateTime::UNIX_EPOCH
        })
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("post not found")]
    NotFound,
    #[error("post is blocked")]
    Blocked,
    #[error("rate limited")]
    RateLimited,
    #[error("API error: {0}")]
    Api(String),
    #[error("invalid response structure")]
    InvalidResponse,
}

fn map_api_error(err_str: String) -> ClientError {
    if err_str.contains("429") || err_str.contains("RateLimited") {
        ClientError::RateLimited
    } else {
        ClientError::Api(err_str)
    }
}

fn extract_images_from_view(
    images: &[atrium_api::app::bsky::embed::images::ViewImage],
) -> Vec<EmbedImage> {
    images
        .iter()
        .map(|img| EmbedImage {
            thumb_url: img.thumb.clone(),
            fullsize_url: img.fullsize.clone(),
            alt: img.alt.clone(),
            aspect_ratio: img.aspect_ratio.as_ref().map(extract_aspect_ratio),
        })
        .collect()
}

fn extract_video_from_view(video_view: &atrium_api::app::bsky::embed::video::View) -> EmbedVideo {
    EmbedVideo {
        thumbnail_url: video_view.thumbnail.clone(),
        playlist_url: video_view.playlist.clone(),
        alt: video_view.alt.clone(),
        aspect_ratio: video_view.aspect_ratio.as_ref().map(extract_aspect_ratio),
    }
}

fn extract_aspect_ratio(ar: &atrium_api::app::bsky::embed::defs::AspectRatio) -> AspectRatio {
    AspectRatio {
        width: ar.width.get() as u32,
        height: ar.height.get() as u32,
    }
}

#[derive(Clone)]
pub struct BlueskyClient {
    client: Arc<AtpServiceClient<ReqwestClient>>,
}

impl BlueskyClient {
    pub fn new(base_url: &str, _timeout: Duration) -> Result<Self, ClientError> {
        // Note: timeout is currently unused as ReqwestClient uses its own defaults.
        // This parameter is kept for future configuration needs.
        let xrpc_client = ReqwestClient::new(base_url);
        let client = Arc::new(AtpServiceClient::new(xrpc_client));

        Ok(Self { client })
    }

    pub async fn resolve_handle(&self, handle: &str) -> Result<String, ClientError> {
        let params = atrium_api::com::atproto::identity::resolve_handle::ParametersData {
            handle: handle
                .parse()
                .map_err(|_| ClientError::Api("invalid handle".to_string()))?,
        };

        let result = self
            .client
            .service
            .com
            .atproto
            .identity
            .resolve_handle(params.into())
            .await
            .map_err(|e| map_api_error(e.to_string()))?;

        Ok(result.did.to_string())
    }

    pub async fn get_thread(&self, at_uri: &str) -> Result<Thread, ClientError> {
        // First, find the root by walking up parents with API calls
        let root_uri = self.find_root_uri_async(at_uri).await?;

        // Get author DID from the root post
        let root_view = self.fetch_post_thread_shallow(&root_uri).await?;
        let author_did = root_view.post.author.did.to_string();
        let author = self.extract_author(&root_view)?;

        // Now iteratively follow self-replies, making API calls as needed
        let mut posts = vec![self.extract_post(&root_view)?];
        let mut current_uri = match self.find_self_reply(&root_view, &author_did) {
            Some(uri) => uri,
            None => return Ok(Thread { posts, author }),
        };

        loop {
            let view = self.fetch_post_thread_shallow(&current_uri).await?;
            posts.push(self.extract_post(&view)?);

            // Find the self-reply in this post's replies
            match self.find_self_reply(&view, &author_did) {
                Some(reply_uri) => {
                    current_uri = reply_uri;
                }
                None => break,
            }
        }

        Ok(Thread { posts, author })
    }

    /// Find the root URI by walking up parents with individual API calls.
    /// This avoids stack overflow from deeply nested response structures.
    async fn find_root_uri_async(&self, start_uri: &str) -> Result<String, ClientError> {
        let mut current_uri = start_uri.to_string();

        loop {
            let view = self.fetch_post_thread_shallow(&current_uri).await?;
            let author_did = view.post.author.did.as_str();

            // Check if there's a parent by the same author
            match self.get_parent_uri_if_same_author(&view, author_did) {
                Some(parent_uri) => {
                    current_uri = parent_uri;
                }
                None => {
                    // No more parents by this author, current is the root
                    return Ok(current_uri);
                }
            }
        }
    }

    /// Get parent URI if the parent is by the same author
    fn get_parent_uri_if_same_author(
        &self,
        view: &ThreadViewPost,
        author_did: &str,
    ) -> Option<String> {
        match &view.parent {
            Some(Union::Refs(ThreadViewPostParentRefs::ThreadViewPost(parent))) => {
                if parent.post.author.did.as_str() == author_did {
                    Some(parent.post.uri.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Fetch a single post with shallow context (1 parent, 1 reply level).
    /// This prevents stack overflow from deeply nested response structures.
    async fn fetch_post_thread_shallow(&self, at_uri: &str) -> Result<ThreadViewPost, ClientError> {
        let params = ParametersData {
            uri: at_uri.to_string(),
            depth: Some(1.try_into().unwrap()),
            parent_height: Some(1.try_into().unwrap()),
        };

        let result = self
            .client
            .service
            .app
            .bsky
            .feed
            .get_post_thread(params.into())
            .await
            .map_err(|e| map_api_error(e.to_string()))?;

        match result.thread.clone() {
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsThreadViewPost(view)) => Ok(*view),
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsNotFoundPost(_)) => {
                Err(ClientError::NotFound)
            }
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsBlockedPost(_)) => {
                Err(ClientError::Blocked)
            }
            _ => Err(ClientError::InvalidResponse),
        }
    }

    /// Find the URI of the first self-reply in the post's replies
    fn find_self_reply(&self, view: &ThreadViewPost, author_did: &str) -> Option<String> {
        let replies = view.replies.as_ref()?;
        for reply in replies {
            if let Union::Refs(ThreadViewPostRepliesItem::ThreadViewPost(reply_view)) = reply {
                if reply_view.post.author.did.as_str() == author_did {
                    return Some(reply_view.post.uri.clone());
                }
            }
        }
        None
    }

    pub async fn get_thread_by_handle(
        &self,
        handle: &str,
        post_id: &str,
    ) -> Result<Thread, ClientError> {
        let did = self.resolve_handle(handle).await?;
        let at_uri = format!("at://{}/app.bsky.feed.post/{}", did, post_id);
        self.get_thread(&at_uri).await
    }

    /// Stream thread events as they are fetched from the API.
    /// This allows for progressive rendering of the thread.
    /// Takes ownership of self (cheap clone via Arc) to allow the stream to be 'static.
    pub fn get_thread_streaming(
        self,
        handle: String,
        post_id: String,
    ) -> impl futures::Stream<Item = Result<StreamEvent, ClientError>> {
        async_stream::try_stream! {
            let did = self.resolve_handle(&handle).await?;
            let at_uri = format!("at://{}/app.bsky.feed.post/{}", did, post_id);

            let root_uri = self.find_root_uri_async(&at_uri).await?;
            let root_view = self.fetch_post_thread_shallow(&root_uri).await?;
            let author = self.extract_author(&root_view)?;
            let author_did = author.did.clone();

            yield StreamEvent::Header(author);
            yield StreamEvent::Post(self.extract_post(&root_view)?);

            let mut current_uri = self.find_self_reply(&root_view, &author_did);

            while let Some(uri) = current_uri {
                let view = self.fetch_post_thread_shallow(&uri).await?;
                yield StreamEvent::Post(self.extract_post(&view)?);
                current_uri = self.find_self_reply(&view, &author_did);
            }

            yield StreamEvent::Done;
        }
    }

    fn extract_author(
        &self,
        view: &atrium_api::app::bsky::feed::defs::ThreadViewPost,
    ) -> Result<Author, ClientError> {
        let author = &view.post.author;
        Ok(Author {
            did: author.did.to_string(),
            handle: author.handle.to_string(),
            display_name: author.display_name.clone(),
            avatar_url: author.avatar.clone(),
        })
    }

    fn extract_post(
        &self,
        view: &atrium_api::app::bsky::feed::defs::ThreadViewPost,
    ) -> Result<ThreadPost, ClientError> {
        let post = &view.post;

        let (text, created_at, langs) = self.extract_post_record(&post.record)?;
        let embed = self.extract_embed(&post.embed);

        Ok(ThreadPost {
            uri: post.uri.clone(),
            cid: post.cid.as_ref().to_string(),
            text,
            created_at,
            reply_count: post.reply_count.map(|v| v as u32),
            repost_count: post.repost_count.map(|v| v as u32),
            like_count: post.like_count.map(|v| v as u32),
            embed,
            langs,
        })
    }

    fn extract_embed(&self, embed: &Option<Union<PostViewEmbedRefs>>) -> Option<Embed> {
        let embed = embed.as_ref()?;

        match embed {
            Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(images_view)) => {
                Some(Embed::Images(extract_images_from_view(&images_view.images)))
            }
            Union::Refs(PostViewEmbedRefs::AppBskyEmbedVideoView(video_view)) => {
                Some(Embed::Video(extract_video_from_view(video_view)))
            }
            Union::Refs(PostViewEmbedRefs::AppBskyEmbedExternalView(external_view)) => {
                Some(Embed::External(EmbedExternal {
                    uri: external_view.external.uri.clone(),
                    title: external_view.external.title.clone(),
                    description: external_view.external.description.clone(),
                    thumb_url: external_view.external.thumb.clone(),
                }))
            }
            Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(record_view)) => self
                .extract_record(record_view)
                .map(|record| Embed::Record(Box::new(record))),
            Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordWithMediaView(record_with_media)) => {
                self.extract_record_with_media(record_with_media)
            }
            _ => None,
        }
    }

    fn extract_record_with_media(
        &self,
        record_with_media: &atrium_api::app::bsky::embed::record_with_media::View,
    ) -> Option<Embed> {
        use atrium_api::app::bsky::embed::record_with_media::ViewMediaRefs;

        let record = self.extract_record(&record_with_media.record)?;

        let media = match &record_with_media.media {
            Union::Refs(ViewMediaRefs::AppBskyEmbedImagesView(images_view)) => {
                Embed::Images(extract_images_from_view(&images_view.images))
            }
            Union::Refs(ViewMediaRefs::AppBskyEmbedVideoView(video_view)) => {
                Embed::Video(extract_video_from_view(video_view))
            }
            Union::Refs(ViewMediaRefs::AppBskyEmbedExternalView(external_view)) => {
                Embed::External(EmbedExternal {
                    uri: external_view.external.uri.clone(),
                    title: external_view.external.title.clone(),
                    description: external_view.external.description.clone(),
                    thumb_url: external_view.external.thumb.clone(),
                })
            }
            _ => return None,
        };

        Some(Embed::RecordWithMedia {
            record: Box::new(record),
            media: Box::new(media),
        })
    }

    fn extract_record(
        &self,
        record_view: &atrium_api::app::bsky::embed::record::View,
    ) -> Option<EmbedRecord> {
        use atrium_api::app::bsky::embed::record::ViewRecordRefs;

        match &record_view.record {
            Union::Refs(ViewRecordRefs::ViewRecord(view_record)) => {
                let author = Author {
                    did: view_record.author.did.to_string(),
                    handle: view_record.author.handle.to_string(),
                    display_name: view_record.author.display_name.clone(),
                    avatar_url: view_record.author.avatar.clone(),
                };

                // Extract text and created_at from the record value
                let value = serde_json::to_value(&view_record.value).ok()?;
                let text = value
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let created_at = parse_created_at(&value, "embed record");

                // Recursively extract any embeds within the quoted post
                // Note: embeds in ViewRecord use ViewRecordEmbedsItem, which is a subset
                // For simplicity, we'll skip nested embeds in quoted posts for now
                // Most quoted posts don't have embeds, and this avoids type complexity
                let embed = None;

                Some(EmbedRecord {
                    uri: view_record.uri.clone(),
                    cid: view_record.cid.as_ref().to_string(),
                    author,
                    text,
                    created_at,
                    embed,
                })
            }
            // Handle other record types (not found, blocked, etc.) by returning None
            _ => None,
        }
    }

    fn extract_post_record(
        &self,
        record: &atrium_api::types::Unknown,
    ) -> Result<(String, DateTime<Utc>, Vec<String>), ClientError> {
        let value = serde_json::to_value(record).map_err(|_| ClientError::InvalidResponse)?;

        let text = value
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let created_at = parse_created_at(&value, "post record");

        let langs = value
            .get("langs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok((text, created_at, langs))
    }
}

use std::sync::Arc;
use std::time::Duration;

use atrium_api::app::bsky::feed::defs::{
    ThreadViewPost, ThreadViewPostParentRefs, ThreadViewPostRepliesItem,
};
use atrium_api::app::bsky::feed::get_post_thread::{OutputThreadRefs, ParametersData};
use atrium_api::client::AtpServiceClient;
use atrium_api::types::Union;
use atrium_xrpc_client::reqwest::ReqwestClient;
use chrono::{DateTime, Utc};
use thiserror::Error;

use super::types::{Author, Thread, ThreadPost};

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

#[derive(Clone)]
pub struct BlueskyClient {
    client: Arc<AtpServiceClient<ReqwestClient>>,
}

impl BlueskyClient {
    pub fn new(base_url: &str, _timeout: Duration) -> Result<Self, ClientError> {
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
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("404") || err_str.contains("not found") {
                    ClientError::NotFound
                } else if err_str.contains("429") || err_str.contains("rate") {
                    ClientError::RateLimited
                } else {
                    ClientError::Api(err_str)
                }
            })?;

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
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("404") || err_str.contains("NotFound") {
                    ClientError::NotFound
                } else if err_str.contains("429") || err_str.contains("rate") {
                    ClientError::RateLimited
                } else {
                    ClientError::Api(err_str)
                }
            })?;

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

        let (text, created_at) = self.extract_post_record(&post.record)?;

        Ok(ThreadPost {
            uri: post.uri.clone(),
            cid: post.cid.as_ref().to_string(),
            text,
            created_at,
            reply_count: post.reply_count.map(|v| v as u32),
            repost_count: post.repost_count.map(|v| v as u32),
            like_count: post.like_count.map(|v| v as u32),
        })
    }

    fn extract_post_record(
        &self,
        record: &atrium_api::types::Unknown,
    ) -> Result<(String, DateTime<Utc>), ClientError> {
        let value = serde_json::to_value(record).map_err(|_| ClientError::InvalidResponse)?;

        let text = value
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let created_at = value
            .get("createdAt")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Ok((text, created_at))
    }
}

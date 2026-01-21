use std::sync::Arc;
use std::time::Duration;

use atrium_api::app::bsky::feed::defs::ThreadViewPostParentRefs;
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
        let params = ParametersData {
            uri: at_uri.to_string(),
            depth: Some(0.try_into().unwrap()),
            parent_height: Some(100.try_into().unwrap()),
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

        self.convert_thread_response(result.thread.clone())
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

    fn convert_thread_response(
        &self,
        thread: Union<OutputThreadRefs>,
    ) -> Result<Thread, ClientError> {
        match thread {
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsThreadViewPost(view)) => {
                let mut posts = Vec::new();
                let author = self.extract_author(&view)?;

                self.collect_parent_posts(&view, &author.did, &mut posts)?;

                posts.push(self.extract_post(&view)?);

                Ok(Thread { posts, author })
            }
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsNotFoundPost(_)) => {
                Err(ClientError::NotFound)
            }
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsBlockedPost(_)) => {
                Err(ClientError::Blocked)
            }
            _ => Err(ClientError::InvalidResponse),
        }
    }

    fn collect_parent_posts(
        &self,
        view: &atrium_api::app::bsky::feed::defs::ThreadViewPost,
        author_did: &str,
        posts: &mut Vec<ThreadPost>,
    ) -> Result<(), ClientError> {
        if let Some(Union::Refs(ThreadViewPostParentRefs::ThreadViewPost(parent_view))) =
            &view.parent
        {
            let parent_author_did = parent_view.post.author.did.as_str();

            if parent_author_did == author_did {
                self.collect_parent_posts(parent_view, author_did, posts)?;
                posts.push(self.extract_post(parent_view)?);
            }
        }
        Ok(())
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

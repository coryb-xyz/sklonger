use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct BlueskyUrlParts {
    pub handle: String,
    pub post_id: String,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("URL must be a bsky.app link")]
    NotBlueskyUrl,
    #[error("URL must be a post link (e.g., bsky.app/profile/user/post/id)")]
    NotPostUrl,
}

pub fn parse_bluesky_url(url_str: &str) -> Result<BlueskyUrlParts, ParseError> {
    let url = Url::parse(url_str)?;

    let host = url.host_str().ok_or(ParseError::NotBlueskyUrl)?;
    if host != "bsky.app" {
        return Err(ParseError::NotBlueskyUrl);
    }

    let segments: Vec<&str> = url.path_segments().ok_or(ParseError::NotPostUrl)?.collect();

    if segments.len() < 4 {
        return Err(ParseError::NotPostUrl);
    }

    if segments[0] != "profile" || segments[2] != "post" {
        return Err(ParseError::NotPostUrl);
    }

    let handle = segments[1].to_string();
    let post_id = segments[3].to_string();

    if handle.is_empty() || post_id.is_empty() {
        return Err(ParseError::NotPostUrl);
    }

    Ok(BlueskyUrlParts { handle, post_id })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_url() {
        let result = parse_bluesky_url("https://bsky.app/profile/jay.bsky.team/post/3jwdwj2ctlk26");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts.handle, "jay.bsky.team");
        assert_eq!(parts.post_id, "3jwdwj2ctlk26");
    }

    #[test]
    fn test_parse_http_url() {
        let result = parse_bluesky_url("http://bsky.app/profile/user.bsky.social/post/abc123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_non_bluesky_url() {
        let result = parse_bluesky_url("https://twitter.com/user/status/123");
        assert!(matches!(result, Err(ParseError::NotBlueskyUrl)));
    }

    #[test]
    fn test_reject_profile_only_url() {
        let result = parse_bluesky_url("https://bsky.app/profile/user.bsky.social");
        assert!(matches!(result, Err(ParseError::NotPostUrl)));
    }

    #[test]
    fn test_reject_invalid_url() {
        let result = parse_bluesky_url("not a url");
        assert!(matches!(result, Err(ParseError::InvalidUrl(_))));
    }
}

# Module Reference

This document provides detailed documentation for each module in the codebase. Use this as a reference when working with specific parts of the application.

## Table of Contents

- [main.rs - Entry Point](#mainrs---entry-point)
- [lib.rs - Application Setup](#librs---application-setup)
- [config.rs - Configuration](#configrs---configuration)
- [handlers.rs - HTTP Handlers](#handlersrs---http-handlers)
- [error.rs - Error Types](#errorrs---error-types)
- [logging.rs - Logging](#loggingrs---logging)
- [bluesky/url_parser.rs - URL Parsing](#blueskyurl_parserrs---url-parsing)
- [bluesky/types.rs - Data Types](#blueskytypesrs---data-types)
- [bluesky/client.rs - API Client](#blueskyclientrs---api-client)
- [html/renderer.rs - HTML Generation](#htmlrendererrs---html-generation)
- [html/templates.rs - Templates](#htmltemplatesrs---templates)

---

## main.rs - Entry Point

The application entry point. Sets up the async runtime and starts the server.

### Functions

#### `main()`

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

**Purpose**: Initialize and run the HTTP server.

**Steps**:
1. Load configuration from environment variables
2. Initialize structured logging
3. Create the application router with state
4. Bind to the configured TCP port
5. Serve requests with graceful shutdown

**Error Handling**: Returns `anyhow::Result` to propagate any startup errors. If configuration is invalid or the port is already in use, the error is logged and the process exits.

#### `shutdown_signal()`

```rust
async fn shutdown_signal()
```

**Purpose**: Wait for shutdown signals (SIGTERM on Unix, Ctrl+C everywhere).

**Behavior**: Uses `tokio::select!` to wait for either:
- Unix: SIGTERM signal
- All platforms: Ctrl+C

When received, logs the signal and returns, allowing graceful shutdown to begin.

---

## lib.rs - Application Setup

The library crate root. Exports all modules and provides the application factory.

### Modules Exported

```rust
pub mod bluesky;    // Bluesky API client
pub mod config;     // Configuration loading
pub mod error;      // Error types
pub mod handlers;   // HTTP request handlers
pub mod html;       // HTML rendering
pub mod logging;    // Log initialization
```

### Types

#### `AppState`

```rust
#[derive(Clone)]
pub struct AppState {
    pub client: BlueskyClient,
}
```

**Purpose**: Shared state passed to all request handlers.

**Fields**:
- `client`: The Bluesky API client, wrapped in `Arc` internally for cheap cloning.

**Why Clone?**: Axum clones the state for each request handler. The `BlueskyClient` uses `Arc` internally, so cloning is just incrementing a reference count.

### Functions

#### `create_app()`

```rust
pub fn create_app(config: &Config) -> anyhow::Result<Router>
```

**Purpose**: Build the Axum router with all routes and shared state.

**Parameters**:
- `config`: Application configuration loaded from environment

**Returns**: Configured `Router` ready to serve requests

**Routes Registered**:
| Method | Path | Handler |
|--------|------|---------|
| GET | `/` | `get_thread` (landing or thread) |
| GET | `/thread` | `get_thread` (thread only) |
| GET | `/profile/{handle}/post/{post_id}` | `get_thread_by_path` |
| GET | `/health/live` | `health_live` |
| GET | `/health/ready` | `health_ready` |

---

## config.rs - Configuration

Environment variable loading and validation.

### Types

#### `Config`

```rust
pub struct Config {
    pub port: u16,
    pub log_level: String,
    pub bluesky_api_url: String,
    pub request_timeout_seconds: u64,
}
```

**Fields**:
- `port`: HTTP server port (default: 8080)
- `log_level`: One of: trace, debug, info, warn, error (default: info)
- `bluesky_api_url`: Bluesky API base URL (default: https://public.api.bsky.app)
- `request_timeout_seconds`: HTTP client timeout (default: 10)

#### `ConfigError`

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid {name}: {reason}")]
    InvalidValue { name: String, reason: String },
}
```

**Purpose**: Typed errors for configuration validation failures.

### Functions

#### `Config::from_env()`

```rust
impl Config {
    pub fn from_env() -> Result<Self, ConfigError>
}
```

**Purpose**: Load and validate configuration from environment variables.

**Environment Variables**:
| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `PORT` | u16 | 8080 | HTTP server port |
| `LOG_LEVEL` | string | info | Logging verbosity |
| `BLUESKY_API_URL` | string | https://public.api.bsky.app | API base URL |
| `REQUEST_TIMEOUT_SECONDS` | u64 | 10 | HTTP timeout |

**Validation**: Port must be parseable as u16. Other values have sensible defaults.

---

## handlers.rs - HTTP Handlers

Request handlers for all HTTP endpoints.

### Types

#### `ThreadQuery`

```rust
#[derive(Deserialize)]
pub struct ThreadQuery {
    pub url: Option<String>,
}
```

**Purpose**: Query parameters for thread fetching endpoints.

### Functions

#### `get_thread()`

```rust
pub async fn get_thread(
    State(state): State<AppState>,
    Query(query): Query<ThreadQuery>,
) -> Result<Html<String>, AppError>
```

**Purpose**: Handle `GET /` and `GET /thread` requests.

**Behavior**:
- If `url` query param is empty or missing: return landing page
- Otherwise: parse URL, fetch thread, render HTML

**Errors**:
- `AppError::BadRequest`: Invalid Bluesky URL
- `AppError::NotFound`: Post doesn't exist
- `AppError::RateLimited`: Too many requests to Bluesky
- `AppError::ServiceUnavailable`: Can't reach Bluesky

#### `get_thread_by_path()`

```rust
pub async fn get_thread_by_path(
    State(state): State<AppState>,
    Path((handle, post_id)): Path<(String, String)>,
) -> Result<Html<String>, AppError>
```

**Purpose**: Handle `GET /profile/{handle}/post/{post_id}` requests.

**Behavior**: Directly fetch and render thread using path parameters.

#### `fetch_and_render_thread()`

```rust
async fn fetch_and_render_thread(
    state: &AppState,
    handle: &str,
    post_id: &str,
) -> Result<Html<String>, AppError>
```

**Purpose**: Core logic shared by both thread handlers.

**Steps**:
1. Call `client.get_thread_by_handle()`
2. Log success with metadata
3. Call `render_thread()` to generate HTML
4. Return wrapped in `Html`

#### `health_live()`

```rust
pub async fn health_live() -> impl IntoResponse
```

**Purpose**: Kubernetes liveness probe.

**Returns**: Always `200 OK` if the process is running.

#### `health_ready()`

```rust
pub async fn health_ready(State(state): State<AppState>) -> impl IntoResponse
```

**Purpose**: Kubernetes readiness probe.

**Behavior**: Attempts to resolve "bsky.app" handle. Returns:
- `200 OK` if successful
- `503 Service Unavailable` if API unreachable

#### `map_client_error()`

```rust
fn map_client_error(err: ClientError) -> AppError
```

**Purpose**: Convert low-level client errors to HTTP-appropriate errors.

**Mappings**:
| ClientError | AppError | HTTP Status |
|-------------|----------|-------------|
| NotFound | NotFound | 404 |
| Blocked | NotFound | 404 |
| RateLimited | RateLimited | 429 |
| Http (network) | ServiceUnavailable | 503 |
| Http (timeout) | ServiceUnavailable | 503 |
| Other | Internal | 500 |

---

## error.rs - Error Types

Application-level error types with HTTP response conversion.

### Types

#### `AppError`

```rust
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    RateLimited,
    Internal(anyhow::Error),
    ServiceUnavailable(String),
}
```

**Variants**:
- `BadRequest`: Invalid input from user (400)
- `NotFound`: Resource doesn't exist (404)
- `RateLimited`: Too many requests (429)
- `Internal`: Unexpected server error (500)
- `ServiceUnavailable`: Dependency unavailable (503)

### Trait Implementations

#### `IntoResponse for AppError`

Converts each variant to an HTTP response with:
- Appropriate status code
- Styled error page HTML
- Consistent theming with normal pages

---

## logging.rs - Logging

Structured logging initialization using the `tracing` ecosystem.

### Functions

#### `init()`

```rust
pub fn init(log_level: &str) -> anyhow::Result<()>
```

**Purpose**: Initialize the global tracing subscriber.

**Parameters**:
- `log_level`: Filter string (e.g., "info", "debug", "trace")

**Behavior**:
- Parses the filter string
- Configures formatted output to stdout
- Sets as global default subscriber

**Usage in Code**:
```rust
use tracing::{info, warn, error, debug};

info!(url = %url, "fetching thread");
warn!(error = ?e, "failed to fetch thread");
```

---

## bluesky/url_parser.rs - URL Parsing

Bluesky URL validation and component extraction.

### Types

#### `BlueskyUrlParts`

```rust
pub struct BlueskyUrlParts {
    pub handle: String,
    pub post_id: String,
}
```

**Purpose**: Extracted components from a valid Bluesky post URL.

#### `ParseError`

```rust
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("not a bsky.app URL")]
    NotBlueskyUrl,

    #[error("not a post URL (must be /profile/*/post/*)")]
    NotPostUrl,
}
```

### Functions

#### `parse_bluesky_url()`

```rust
pub fn parse_bluesky_url(url_str: &str) -> Result<BlueskyUrlParts, ParseError>
```

**Purpose**: Parse and validate a Bluesky post URL.

**Valid URL Format**:
```
https://bsky.app/profile/{handle}/post/{post_id}
```

**Validation Steps**:
1. Parse as URL (reject malformed)
2. Check host is "bsky.app"
3. Check path structure is `/profile/{handle}/post/{post_id}`
4. Reject profile pages, feeds, etc.

**Examples**:
```rust
// Valid
parse_bluesky_url("https://bsky.app/profile/user.bsky.social/post/abc123")
// -> Ok(BlueskyUrlParts { handle: "user.bsky.social", post_id: "abc123" })

// Invalid - not a post
parse_bluesky_url("https://bsky.app/profile/user.bsky.social")
// -> Err(ParseError::NotPostUrl)

// Invalid - wrong domain
parse_bluesky_url("https://example.com/profile/user/post/abc")
// -> Err(ParseError::NotBlueskyUrl)
```

---

## bluesky/types.rs - Data Types

Data structures representing Bluesky content.

### Types

#### `Thread`

```rust
pub struct Thread {
    pub posts: Vec<ThreadPost>,
    pub author: Author,
}
```

**Purpose**: A complete self-reply thread.

**Methods**:
- `original_post_url()`: Get URL of first post on bsky.app
- `primary_language()`: Get BCP-47 language code from first post

#### `ThreadPost`

```rust
pub struct ThreadPost {
    pub uri: String,
    pub cid: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub reply_count: Option<u32>,
    pub repost_count: Option<u32>,
    pub like_count: Option<u32>,
    pub embed: Option<Embed>,
    pub langs: Vec<String>,
}
```

**Fields**:
- `uri`: AT Protocol URI (e.g., `at://did:plc:xxx/app.bsky.feed.post/yyy`)
- `cid`: Content Identifier (hash of the post)
- `text`: Post text content
- `created_at`: When the post was created
- `reply_count`, `repost_count`, `like_count`: Engagement metrics
- `embed`: Attached media or link
- `langs`: BCP-47 language codes (e.g., ["en", "es"])

#### `Author`

```rust
pub struct Author {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
```

**Methods**:
- `profile_url()`: Get URL of author's profile on bsky.app

#### `Embed`

```rust
pub enum Embed {
    Images(Vec<EmbedImage>),
    Video(EmbedVideo),
    External(EmbedExternal),
}
```

**Variants**:
- `Images`: One or more attached images
- `Video`: Attached video with HLS playlist
- `External`: Link preview card

#### `EmbedImage`

```rust
pub struct EmbedImage {
    pub thumb_url: String,
    pub fullsize_url: String,
    pub alt: String,
    pub aspect_ratio: Option<AspectRatio>,
}
```

#### `EmbedVideo`

```rust
pub struct EmbedVideo {
    pub thumbnail_url: Option<String>,
    pub playlist_url: String,
    pub aspect_ratio: Option<AspectRatio>,
}
```

#### `EmbedExternal`

```rust
pub struct EmbedExternal {
    pub uri: String,
    pub title: String,
    pub description: String,
    pub thumb_url: Option<String>,
}
```

#### `AspectRatio`

```rust
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
}
```

---

## bluesky/client.rs - API Client

AT Protocol API client for fetching threads.

### Types

#### `BlueskyClient`

```rust
pub struct BlueskyClient {
    client: Arc<AtpServiceClient<ReqwestClient>>,
}
```

**Purpose**: Wrapper around the atrium AT Protocol client.

**Why Arc?**: The client is shared across requests. `Arc` provides thread-safe reference counting.

#### `ClientError`

```rust
pub enum ClientError {
    Http(reqwest::Error),
    NotFound,
    Blocked,
    RateLimited,
    Api(String),
    InvalidResponse,
}
```

### Functions

#### `BlueskyClient::new()`

```rust
pub fn new(base_url: &str, _timeout: Duration) -> Result<Self, ClientError>
```

**Purpose**: Create a new API client.

**Parameters**:
- `base_url`: Bluesky API base URL
- `_timeout`: Reserved for future use

#### `resolve_handle()`

```rust
pub async fn resolve_handle(&self, handle: &str) -> Result<String, ClientError>
```

**Purpose**: Convert a handle to a DID.

**Example**:
```rust
client.resolve_handle("user.bsky.social").await
// -> Ok("did:plc:abc123xyz")
```

#### `get_thread_by_handle()`

```rust
pub async fn get_thread_by_handle(
    &self,
    handle: &str,
    post_id: &str,
) -> Result<Thread, ClientError>
```

**Purpose**: Fetch a complete thread given handle and post ID.

**Steps**:
1. Resolve handle to DID
2. Construct AT URI
3. Delegate to `get_thread()`

#### `get_thread()`

```rust
pub async fn get_thread(&self, at_uri: &str) -> Result<Thread, ClientError>
```

**Purpose**: Fetch a complete thread given an AT URI.

**Algorithm**:
1. **Find root**: Walk up parent chain until we find the first post by this author that either has no parent, or has a parent by a different author
2. **Walk down**: Follow self-replies from root to end

**Why iterative?**: Bluesky's API returns deeply nested JSON for long threads. Recursive deserialization can cause stack overflow. By fetching one post at a time with shallow depth, we avoid this.

#### Helper Functions

- `fetch_post_thread_shallow()`: Fetch with depth=1, parent_height=1
- `find_root_uri_async()`: Walk up parent chain
- `find_self_reply()`: Find first reply by same author
- `get_parent_uri_if_same_author()`: Check if parent is by same author
- `extract_author()`: Build Author from API response
- `extract_post()`: Build ThreadPost from API response
- `extract_embed()`: Handle different embed types

---

## html/renderer.rs - HTML Generation

Thread-to-HTML conversion.

### Functions

#### `render_thread()`

```rust
pub fn render_thread(thread: &Thread) -> String
```

**Purpose**: Convert a Thread to a complete HTML page.

**Output Structure**:
```html
<!DOCTYPE html>
<html lang="en">
<head>...</head>
<body>
  <header>...</header>
  <main class="thread">
    <article class="post">...</article>
    <article class="post">...</article>
  </main>
  <footer>...</footer>
</body>
</html>
```

#### `render_header()`

```rust
fn render_header(author: &Author) -> String
```

**Purpose**: Generate the page header with author info.

**Contents**:
- Author avatar (or initial if no avatar)
- Display name and handle
- Link to profile
- Theme toggle button
- Home link

#### `render_post()`

```rust
fn render_post(post: &ThreadPost, author_handle: &str) -> String
```

**Purpose**: Generate HTML for a single post.

**Contents**:
- Linkified and escaped post text
- Embedded media (if present)
- Timestamp and engagement stats
- Link to original post

#### `render_embed()`

```rust
fn render_embed(embed: &Embed) -> String
```

**Purpose**: Render embedded content based on type.

**Behavior by Type**:
- **Images**: Grid layout, lazy loading, aspect ratio preservation
- **Video**: HLS video player with poster image
- **External**: Card with thumbnail, title, description

#### `linkify_text()`

```rust
fn linkify_text(text: &str) -> String
```

**Purpose**: Convert URLs in text to clickable links.

**Behavior**:
- Detects `http://` and `https://` URLs
- Wraps in `<a>` tags with `rel="noopener"`
- Truncates display text to 40 characters

**Regex**: `https?://[^\s<>]+`

**Optimization**: Uses `OnceLock` to compile regex only once.

---

## html/templates.rs - Templates

HTML templates and embedded CSS/JS.

### Constants

#### `CSS_STYLES`

Complete CSS stylesheet (~700 lines). Features:
- CSS custom properties for theming
- Light and dark mode support
- Mobile-responsive breakpoints
- Sticky header that compacts on scroll
- Print-friendly styles

#### `THEME_SCRIPT`

Runs before page renders:
```javascript
(function() {
    const theme = localStorage.getItem('theme');
    if (theme) {
        document.documentElement.setAttribute('data-theme', theme);
    }
})();
```

#### `THEME_TOGGLE_SCRIPT`

Handles theme switching:
- Click handler for toggle button
- Keyboard support (Enter, Space)
- Persistence to localStorage
- Scroll-based header compaction

### Functions

#### `base_template_with_options()`

```rust
pub fn base_template_with_options(
    title: &str,
    content: &str,
    options: TemplateOptions,
) -> String
```

**Purpose**: Wrap content in a complete HTML5 document.

**Parameters**:
- `title`: Page title
- `content`: Main page content HTML
- `options`: Additional options (favicon, language)

**Output**: Complete HTML document with:
- Proper meta tags
- Embedded CSS and JS
- Favicon from author avatar
- Language attribute from post

#### `landing_page()`

```rust
pub fn landing_page() -> String
```

**Purpose**: Generate the landing page with URL input form.

**Features**:
- Form to enter Bluesky URL
- Theme toggle
- Clear instructions

#### `error_page()`

```rust
pub fn error_page(status_code: u16, title: &str, message: &str) -> String
```

**Purpose**: Generate styled error pages.

**Usage**: Called by `AppError::into_response()` for all error types.

### Types

#### `TemplateOptions`

```rust
pub struct TemplateOptions {
    pub favicon_url: Option<String>,
    pub lang: Option<String>,
}
```

**Fields**:
- `favicon_url`: Author's avatar for page icon
- `lang`: BCP-47 language code for `<html lang="">` attribute

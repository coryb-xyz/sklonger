# Request Lifecycle

This document walks through exactly what happens when a user requests a Bluesky thread. We'll trace the code path from the initial HTTP request to the final HTML response.

## Example Request

We'll follow this request:
```
GET /?url=https://bsky.app/profile/example.bsky.social/post/abc123
```

## Phase 1: Server Startup

Before any requests can be handled, the server must start. This happens in `main.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load configuration from environment
    let config = Config::from_env()?;

    // 2. Initialize structured logging
    logging::init(&config.log_level)?;

    // 3. Build the application (router + state)
    let app = create_app(&config)?;

    // 4. Bind to the configured port
    let listener = TcpListener::bind(addr).await?;

    // 5. Start serving with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
```

### What's `#[tokio::main]`?

This attribute macro transforms our `async fn main()` into a regular `fn main()` that sets up the Tokio async runtime. Without it, we couldn't use `await` in main.

### What's `create_app()`?

Defined in `lib.rs`, this function builds the Axum router:

```rust
pub fn create_app(config: &Config) -> anyhow::Result<Router> {
    // Create the Bluesky API client
    let client = BlueskyClient::new(
        &config.bluesky_api_url,
        Duration::from_secs(config.request_timeout_seconds),
    )?;

    // Wrap in shared state
    let state = AppState { client };

    // Define routes
    Router::new()
        .route("/", get(handlers::get_thread))
        .route("/thread", get(handlers::get_thread))
        .route("/profile/{handle}/post/{post_id}", get(handlers::get_thread_by_path))
        .route("/health/live", get(handlers::health_live))
        .route("/health/ready", get(handlers::health_ready))
        .with_state(state)
}
```

The `AppState` is cloned for each request handler via Axum's `State` extractor.

## Phase 2: Request Routing

When our request arrives, Axum matches it to a handler:

| URL Pattern | Handler | Entry Point |
|-------------|---------|-------------|
| `GET /` | `get_thread` | Query param or landing page |
| `GET /thread` | `get_thread` | Query param only |
| `GET /profile/{handle}/post/{post_id}` | `get_thread_by_path` | Path params |
| `GET /health/live` | `health_live` | Kubernetes liveness |
| `GET /health/ready` | `health_ready` | Kubernetes readiness |

Our request matches `GET /` and goes to `handlers::get_thread`.

## Phase 3: Handler Execution

### 3.1 Parameter Extraction

Axum automatically extracts and deserializes query parameters:

```rust
#[derive(Deserialize)]
pub struct ThreadQuery {
    pub url: Option<String>,
}

pub async fn get_thread(
    State(state): State<AppState>,       // Shared app state
    Query(query): Query<ThreadQuery>,     // Query parameters
) -> Result<Html<String>, AppError> {
    // ...
}
```

For our request, `query.url` is `Some("https://bsky.app/profile/example.bsky.social/post/abc123")`.

### 3.2 Landing Page or Thread?

```rust
let url = match query.url {
    Some(u) if !u.is_empty() => u,
    _ => return Ok(Html(landing_page())),  // No URL = show landing page
};
```

We have a URL, so we continue.

### 3.3 URL Parsing

```rust
let parts = parse_bluesky_url(&url).map_err(|e| AppError::BadRequest(e.to_string()))?;
```

The `parse_bluesky_url` function in `bluesky/url_parser.rs` validates and extracts:

```rust
pub fn parse_bluesky_url(url_str: &str) -> Result<BlueskyUrlParts, ParseError> {
    // Parse the URL
    let url = Url::parse(url_str)?;

    // Must be bsky.app domain
    if url.host_str() != Some("bsky.app") {
        return Err(ParseError::NotBlueskyUrl);
    }

    // Extract path segments: ["profile", "handle", "post", "post_id"]
    let segments: Vec<_> = url.path_segments().collect();

    // Validate structure
    if segments.len() < 4 || segments[0] != "profile" || segments[2] != "post" {
        return Err(ParseError::NotPostUrl);
    }

    Ok(BlueskyUrlParts {
        handle: segments[1].to_string(),
        post_id: segments[3].to_string(),
    })
}
```

For our URL, we get:
- `handle`: "example.bsky.social"
- `post_id`: "abc123"

### 3.4 Thread Fetching

```rust
let thread = fetch_and_render_thread(&state, &parts.handle, &parts.post_id).await?;
```

This delegates to the Bluesky client (covered in Phase 4).

### 3.5 Response

On success, we return `Ok(Html(html_string))`. Axum converts this to:
- Status: 200 OK
- Content-Type: text/html; charset=utf-8
- Body: The HTML string

## Phase 4: Bluesky API Client

The `BlueskyClient` in `bluesky/client.rs` handles all AT Protocol communication.

### 4.1 Handle Resolution

Bluesky uses DIDs (Decentralized Identifiers) internally, not handles. First, we resolve the handle:

```rust
pub async fn resolve_handle(&self, handle: &str) -> Result<String, ClientError> {
    let response = self.client
        .service
        .com
        .atproto
        .identity
        .resolve_handle(/* ... */)
        .await?;

    Ok(response.did.to_string())
}
```

For "example.bsky.social", we might get `did:plc:abc123xyz`.

### 4.2 AT URI Construction

We build an AT Protocol URI:

```
at://did:plc:abc123xyz/app.bsky.feed.post/abc123
```

This uniquely identifies the post across the network.

### 4.3 Finding the Thread Root

A thread might be 50 posts long, and the user might have linked to post #30. We need to find the beginning.

**The Challenge**: Bluesky's API returns nested JSON responses. For deeply-nested threads (100+ posts), parsing this recursively can cause stack overflow.

**The Solution**: We fetch posts one at a time with shallow depth, walking up the parent chain:

```rust
async fn find_root_uri_async(&self, start_uri: &str) -> Result<(String, String), ClientError> {
    let mut current_uri = start_uri.to_string();
    let mut author_did = String::new();

    loop {
        // Fetch just this post with minimal depth
        let response = self.fetch_post_thread_shallow(&current_uri).await?;

        // Get the post's author
        let current_author = extract_author(&response);
        author_did = current_author.did.clone();

        // Check if parent exists and is by same author
        if let Some(parent_uri) = get_parent_uri_if_same_author(&response, &author_did) {
            current_uri = parent_uri;  // Keep walking up
        } else {
            break;  // This is the root!
        }
    }

    Ok((current_uri, author_did))
}
```

### 4.4 Following the Thread Down

Once we have the root, we follow self-replies:

```rust
pub async fn get_thread(&self, at_uri: &str) -> Result<Thread, ClientError> {
    // Find the root post
    let (root_uri, author_did) = self.find_root_uri_async(at_uri).await?;

    // Fetch the root
    let root_response = self.fetch_post_thread_shallow(&root_uri).await?;
    let author = extract_author(&root_response);
    let mut posts = vec![extract_post(&root_response)?];

    // Walk through self-replies
    let mut current_response = root_response;
    loop {
        // Find a reply by the same author
        if let Some(reply_uri) = find_self_reply(&current_response, &author_did) {
            // Fetch that reply
            current_response = self.fetch_post_thread_shallow(&reply_uri).await?;
            posts.push(extract_post(&current_response)?);
        } else {
            break;  // No more self-replies
        }
    }

    Ok(Thread { posts, author })
}
```

### 4.5 Data Extraction

Each post is converted from the API response to our internal types:

```rust
fn extract_post(view: &ThreadViewPost) -> Result<ThreadPost, ClientError> {
    let post = &view.post;
    let record = extract_post_record(post)?;

    Ok(ThreadPost {
        uri: post.uri.to_string(),
        cid: post.cid.to_string(),
        text: record.text,
        created_at: record.created_at,
        reply_count: post.reply_count,
        repost_count: post.repost_count,
        like_count: post.like_count,
        embed: extract_embed(&post.embed),
        langs: record.langs,
    })
}
```

Embeds (images, videos, external links) are also extracted:

```rust
fn extract_embed(embed_opt: &Option<...>) -> Option<Embed> {
    match embed_ref {
        EmbedViewRefs::ImagesView(images) => {
            Some(Embed::Images(extract_images(images)))
        }
        EmbedViewRefs::VideoView(video) => {
            Some(Embed::Video(extract_video(video)))
        }
        EmbedViewRefs::ExternalView(external) => {
            Some(Embed::External(extract_external(external)))
        }
        // ... other types
    }
}
```

## Phase 5: HTML Rendering

With the `Thread` data, we generate HTML in `html/renderer.rs`.

### 5.1 Thread Rendering

```rust
pub fn render_thread(thread: &Thread) -> String {
    let mut html = String::new();

    // Header with author info and theme toggle
    html.push_str(&render_header(&thread.author));

    // Each post in the thread
    html.push_str("<main class=\"thread\">");
    for post in &thread.posts {
        html.push_str(&render_post(post, &thread.author.handle));
    }
    html.push_str("</main>");

    // Footer with link to original
    html.push_str(&render_footer(thread));

    // Wrap in full HTML document
    base_template_with_options(
        &format!("Thread by @{}", thread.author.handle),
        &html,
        TemplateOptions {
            favicon_url: thread.author.avatar_url.clone(),
            lang: thread.primary_language(),
        },
    )
}
```

### 5.2 Post Rendering

Each post becomes an HTML article:

```rust
fn render_post(post: &ThreadPost, author_handle: &str) -> String {
    format!(
        r#"<article class="post">
            <div class="post-content">{text}</div>
            {embed}
            <footer class="post-meta">
                <time datetime="{iso_time}">{formatted_time}</time>
                {stats}
            </footer>
        </article>"#,
        text = linkify_text(&html_escape::encode_text(&post.text)),
        embed = render_embed_if_present(&post.embed),
        // ...
    )
}
```

### 5.3 Text Linkification

URLs in post text become clickable links:

```rust
fn linkify_text(text: &str) -> String {
    // Regex compiled once and cached
    static URL_REGEX: OnceLock<Regex> = OnceLock::new();
    let re = URL_REGEX.get_or_init(|| {
        Regex::new(r"https?://[^\s<>]+").unwrap()
    });

    re.replace_all(text, |caps: &Captures| {
        let url = &caps[0];
        let display = if url.len() > 40 {
            format!("{}...", &url[..40])
        } else {
            url.to_string()
        };
        format!(r#"<a href="{}" rel="noopener">{}</a>"#, url, display)
    }).to_string()
}
```

### 5.4 HTML Safety

All user content is escaped before insertion:

```rust
// Text content - escape HTML entities
html_escape::encode_text(&post.text)  // < becomes &lt;

// Attribute values - escape for attributes
html_escape::encode_quoted_attribute(&url)  // " becomes &quot;
```

This prevents XSS attacks from malicious post content.

## Phase 6: Template Assembly

The final HTML is wrapped in `templates.rs`:

```rust
pub fn base_template_with_options(title: &str, content: &str, options: TemplateOptions) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    <link rel="icon" href="{favicon}">
    <style>{CSS_STYLES}</style>
    <script>{THEME_SCRIPT}</script>
</head>
<body>
    {content}
    <script>{THEME_TOGGLE_SCRIPT}</script>
</body>
</html>"#,
        // ...
    )
}
```

### Theme System

The CSS uses custom properties for theming:

```css
:root {
    --bg-primary: #fafafa;
    --text-primary: #1f2937;
}

@media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) {
        --bg-primary: #121212;
        --text-primary: #e4e4e7;
    }
}

[data-theme="dark"] {
    --bg-primary: #121212;
    --text-primary: #e4e4e7;
}
```

The theme toggle script manages the `data-theme` attribute and persists preferences to localStorage.

## Phase 7: Response Delivery

The complete flow:

```
Thread data
    ↓
render_thread() → HTML string
    ↓
Ok(Html(html)) → Result<Html<String>, AppError>
    ↓
Axum's IntoResponse trait → HTTP Response
    ↓
TCP → User's browser
```

## Error Handling Throughout

At each phase, errors are handled and converted to appropriate HTTP responses:

| Error Source | Error Type | HTTP Status |
|--------------|------------|-------------|
| URL parsing | `ParseError` | 400 Bad Request |
| Handle resolution | `ClientError::NotFound` | 404 Not Found |
| Post not found | `ClientError::NotFound` | 404 Not Found |
| Rate limited | `ClientError::RateLimited` | 429 Too Many Requests |
| Network failure | `ClientError::Http` | 503 Service Unavailable |
| Unexpected | `anyhow::Error` | 500 Internal Server Error |

All error responses use the same styled HTML template as successful responses.

## Health Check Endpoints

### Liveness (`/health/live`)

```rust
pub async fn health_live() -> impl IntoResponse {
    StatusCode::OK  // Always returns 200 if process is running
}
```

Kubernetes uses this to know the process is alive. Failure triggers a pod restart.

### Readiness (`/health/ready`)

```rust
pub async fn health_ready(State(state): State<AppState>) -> impl IntoResponse {
    match state.client.resolve_handle("bsky.app").await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
```

This verifies we can reach the Bluesky API. Failure removes the pod from the load balancer until it recovers.

## Summary

The complete request lifecycle:

1. **Startup**: Config loaded, logging initialized, router built
2. **Routing**: Request matched to handler by URL pattern
3. **Handler**: Parameters extracted, URL validated
4. **Client**: Handle resolved, thread fetched iteratively
5. **Render**: Thread converted to semantic HTML
6. **Template**: Content wrapped with CSS and scripts
7. **Response**: HTML sent to browser

Each component is isolated and testable, following Rust's philosophy of making errors visible at compile time where possible.

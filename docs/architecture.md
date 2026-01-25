# Architecture Overview

This document explains how skeet-longer works, from the moment a user makes a request to when they see the rendered thread. If you're new to Rust, don't worry - we'll explain the concepts as we go.

## What Does This Application Do?

skeet-longer is a web service that:
1. Accepts a Bluesky post URL from a user
2. Fetches the entire self-reply thread (posts by the same author replying to themselves)
3. Renders it as a clean, readable HTML page

Think of it like a "thread unroller" for Bluesky.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              User's Browser                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            HTTP Server (Axum)                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ GET /       │  │ GET /thread │  │ GET /profile│  │ GET /health/*       │ │
│  │ Landing or  │  │ Query param │  │ Path params │  │ Kubernetes probes   │ │
│  │ Thread fetch│  │ Thread fetch│  │ Thread fetch│  │                     │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Request Handlers                                │
│                             (src/handlers.rs)                                │
│                                                                              │
│  1. Parse and validate the Bluesky URL                                       │
│  2. Extract handle and post ID                                               │
│  3. Call the Bluesky client to fetch thread                                  │
│  4. Render HTML response                                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                         ┌────────────┴────────────┐
                         ▼                         ▼
┌───────────────────────────────────┐  ┌─────────────────────────────────────┐
│       Bluesky API Client          │  │         HTML Renderer               │
│       (src/bluesky/)              │  │         (src/html/)                 │
│                                   │  │                                     │
│  - Resolve handle to DID          │  │  - Generate semantic HTML5          │
│  - Fetch thread from API          │  │  - Inline CSS with theming          │
│  - Walk self-reply chain          │  │  - Light/dark mode support          │
│  - Extract post content           │  │  - Mobile responsive                │
└───────────────────────────────────┘  └─────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Bluesky AT Protocol API                                  │
│                   (https://public.api.bsky.app)                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Key Rust Concepts Used

Before diving into the code, here are some Rust concepts you'll encounter:

### Async/Await

Rust uses `async`/`await` for non-blocking I/O. When we fetch data from the Bluesky API, we don't want to block the entire server waiting for a response. Instead:

```rust
// This function is async - it can pause and resume
async fn fetch_thread(url: &str) -> Result<Thread, Error> {
    // .await pauses here until the API responds
    // Other requests can be processed while waiting
    let response = client.get(url).await?;
    Ok(response)
}
```

### Result Types and Error Handling

Rust doesn't have exceptions. Instead, functions that can fail return `Result<T, E>`:

```rust
// Ok(value) for success, Err(error) for failure
fn parse_url(input: &str) -> Result<BlueskyUrlParts, ParseError> {
    if input.is_empty() {
        return Err(ParseError::InvalidUrl("empty input".to_string()));
    }
    // ... parsing logic
    Ok(BlueskyUrlParts { handle, post_id })
}
```

The `?` operator is shorthand for "if this is an error, return it immediately":

```rust
// These are equivalent:
let parts = parse_url(input)?;

let parts = match parse_url(input) {
    Ok(p) => p,
    Err(e) => return Err(e.into()),
};
```

### Ownership and Borrowing

Rust tracks who "owns" data to prevent memory bugs:

```rust
// & means "borrow" - we can read but not modify or take ownership
fn render_post(post: &ThreadPost) -> String {
    // We're just reading post, not taking ownership
    format!("<p>{}</p>", post.text)
}

// Without &, we'd take ownership and the caller couldn't use it anymore
fn consume_post(post: ThreadPost) {
    // post is now ours, caller can't use it
}
```

### Traits (Interfaces)

Traits define shared behavior. Think of them like interfaces in other languages:

```rust
// Any type implementing IntoResponse can be returned from a handler
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Convert our error into an HTTP response
    }
}
```

## Project Structure

```
src/
├── main.rs           # Application entry point
├── lib.rs            # Library crate, router setup
├── config.rs         # Environment configuration
├── handlers.rs       # HTTP request handlers
├── error.rs          # Error types and HTTP responses
├── logging.rs        # Structured logging setup
├── bluesky/
│   ├── mod.rs        # Module exports
│   ├── client.rs     # AT Protocol API client
│   ├── types.rs      # Data structures (Thread, Post, Author)
│   └── url_parser.rs # Bluesky URL validation
└── html/
    ├── mod.rs        # Module exports
    ├── renderer.rs   # Thread-to-HTML conversion
    └── templates.rs  # HTML templates and CSS
```

## The Request Flow

When a user visits `/?url=https://bsky.app/profile/user.bsky.social/post/abc123`:

1. **main.rs** - The server accepts the TCP connection
2. **lib.rs** - The router matches the URL to `handlers::get_thread`
3. **handlers.rs** - Extracts the query parameter, validates it
4. **url_parser.rs** - Parses and validates the Bluesky URL
5. **client.rs** - Fetches the thread from Bluesky's API
6. **renderer.rs** - Converts the thread data to HTML
7. **templates.rs** - Wraps content in the full HTML template
8. **handlers.rs** - Returns the HTML response to the browser

Each step is covered in detail in [Request Lifecycle](./request-lifecycle.md).

## Design Principles

### Stateless

The application stores no state between requests. Every request is independent. This makes it easy to scale horizontally - just run more copies.

### 12-Factor Compliant

Following [12factor.net](https://12factor.net/) methodology:
- Configuration via environment variables
- Logs to stdout (no log files)
- Graceful shutdown on SIGTERM
- No local filesystem writes

### Minimal Dependencies in Output

The HTML output has:
- All CSS inlined (no external stylesheets)
- Minimal JavaScript (only for theme toggle)
- No external fonts or CDNs
- Fast loading on any connection

### Security First

- All user input is HTML-escaped before rendering
- URL validation prevents SSRF attacks
- No cookies or session state to protect

## Next Steps

- [Request Lifecycle](./request-lifecycle.md) - Detailed walkthrough of a request
- [Module Reference](./modules.md) - Documentation for each module
- [Configuration & Deployment](./deployment.md) - Running the application

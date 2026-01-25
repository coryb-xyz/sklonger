# Bluesky Thread Concatenator (skeet-longer)

A Rust web service that fetches Bluesky threads (self-reply chains) and renders them as a single, readable HTML page.

## Project Goals

- Fetch public Bluesky posts and their self-reply chains via AT Protocol
- Serve concatenated threads as clean, accessible HTML
- Follow 12-factor app methodology
- Deploy as a Kubernetes-native containerized service
- Minimal JavaScript, clean CSS, light/dark mode support

## Tech Stack

- **Language**: Rust (latest LTS edition)
- **Web Framework**: TBD - select a lightweight, async HTTP server (consider axum or actix-web)
- **AT Protocol Client**: TBD - research and select appropriate crate from crates.io
- **Container**: Multi-stage build with distroless final image
- **Deployment**: Kubernetes via Helm chart
- **Testing**: Built-in `cargo test`

## 12-Factor Compliance

This project strictly follows [12factor.net](https://12factor.net/) methodology:

1. **Codebase**: Single repo, multiple deploys via Helm values
2. **Dependencies**: Explicit via Cargo.toml, no system dependencies
3. **Config**: Environment variables only (no hardcoded values)
4. **Backing Services**: Treat Bluesky AT Protocol API as attached resource
5. **Build/Release/Run**: Strict separation via multi-stage Docker build
6. **Processes**: Stateless, share-nothing HTTP service
7. **Port Binding**: Self-contained HTTP server, port via env var
8. **Concurrency**: Horizontal scaling via k8s replicas
9. **Disposability**: Fast startup/shutdown, graceful SIGTERM handling
10. **Dev/Prod Parity**: Same container image across environments
11. **Logs**: Structured logging to stdout/stderr only
12. **Admin Processes**: Any admin tasks via one-off k8s jobs

## Project Structure

```
.
├── src/
│   ├── main.rs              # HTTP server setup, graceful shutdown
│   ├── config.rs            # Environment variable configuration
│   ├── handlers.rs          # HTTP request handlers
│   ├── bluesky/
│   │   ├── mod.rs           # AT Protocol client wrapper
│   │   ├── client.rs        # API interaction logic
│   │   └── types.rs         # Bluesky data structures
│   ├── html/
│   │   ├── mod.rs           # HTML generation
│   │   ├── renderer.rs      # Thread-to-HTML conversion
│   │   └── templates.rs     # HTML templates (embedded at compile time)
│   └── lib.rs               # Library crate for testability
├── tests/
│   ├── integration_tests.rs # End-to-end HTTP tests
│   └── fixtures/            # Test data (sample Bluesky responses)
├── static/                  # CSS embedded in HTML template
├── Dockerfile               # Multi-stage build
├── helm/
│   └── bluesky-concatenator/
│       ├── Chart.yaml
│       ├── values.yaml
│       └── templates/
│           ├── deployment.yaml
│           ├── service.yaml
│           └── ingress.yaml
├── Cargo.toml
├── Cargo.lock
└── CLAUDE.md
```

## Key Commands

```bash
# Development
cargo build                          # Build debug binary
cargo run                            # Run locally (set PORT, LOG_LEVEL env vars)
cargo test                           # Run all tests
cargo clippy                         # Lint
cargo fmt                            # Format code

# Container
docker build -t bluesky-concat:dev . # Build container locally
docker run -p 8080:8080 \           # Run container
  -e PORT=8080 \
  -e LOG_LEVEL=info \
  bluesky-concat:dev

# Kubernetes (local testing)
helm install bluesky-concat ./helm/bluesky-concatenator \
  --set image.tag=dev \
  --set config.logLevel=debug

# Helm testing
helm lint ./helm/bluesky-concatenator
helm template ./helm/bluesky-concatenator
```

## Configuration (Environment Variables)

Required environment variables (see `config.rs`):

- `PORT` - HTTP server port (default: 8080)
- `LOG_LEVEL` - Logging verbosity (trace, debug, info, warn, error; default: info)
- `BLUESKY_API_URL` - AT Protocol API base URL (default: https://public.api.bsky.app)
- `RATE_LIMIT_PER_MINUTE` - Max requests per minute (default: 30)
- `REQUEST_TIMEOUT_SECONDS` - HTTP client timeout (default: 10)

## Kubernetes-Native Requirements

### Health Endpoints

Implement these endpoints for k8s probes:

- `GET /health/live` - Liveness probe (always returns 200 OK if process is running)
- `GET /health/ready` - Readiness probe (200 OK if can reach Bluesky API, 503 otherwise)

### Graceful Shutdown

- Listen for SIGTERM
- Stop accepting new requests
- Complete in-flight requests (with timeout)
- Exit cleanly within 30 seconds

## API Integration

### AT Protocol / Bluesky Documentation

- Main docs: https://docs.bsky.app/
- AT Protocol spec: https://atproto.com/specs/atp
- API reference: https://docs.bsky.app/docs/api/

Before implementing API client, search crates.io for existing AT Protocol libraries. Evaluate based on:
- Async support (tokio compatibility)
- Active maintenance
- Public API coverage (specifically `app.bsky.feed.getPostThread`)

### URL Parsing

Input: `https://bsky.app/profile/{handle}/post/{post-id}`
Extract: `{handle}` and `{post-id}` to construct AT Protocol API request

Expected API endpoint: `app.bsky.feed.getPostThread`

## HTTP Interface

### Primary Endpoint

`GET /?url={bluesky_url}` or `GET /thread?url={bluesky_url}`

Example: `GET /?url=https://bsky.app/profile/user.bsky.social/post/abc123`

Returns: HTML page with concatenated thread

### Error Handling

- 400 Bad Request: Invalid/missing URL parameter
- 404 Not Found: Post not found or not accessible
- 429 Too Many Requests: Rate limit exceeded
- 500 Internal Server Error: Bluesky API errors, unexpected failures
- 503 Service Unavailable: Cannot reach Bluesky API

All errors should return minimal HTML error pages (same styling as threads)

## HTML Output Requirements

### Design Principles

- Semantic HTML5
- Mobile-responsive (viewport meta tag, flexible layouts)
- Accessible (proper heading hierarchy, ARIA labels where needed)
- Fast loading (inline CSS, no external dependencies)
- Printable

### Light/Dark Mode

- Use `prefers-color-scheme` media query
- No JavaScript toggle needed initially
- High contrast ratios (WCAG AA minimum)

### CSS Guidelines

- Inline all CSS in HTML `<style>` tag
- Use CSS custom properties for theming
- System font stack (no web fonts)
- Minimal framework footprint (prefer vanilla CSS)

### Content Structure

For each post in thread:
- Author handle and display name
- Post timestamp (formatted as relative or absolute time)
- Post text (preserve line breaks, handle links)
- Post metadata (reply count, like count - optional)
- Clear visual separation between posts

## Code Style

### Rust Conventions

- Follow standard Rust style (rustfmt default config)
- Use `clippy` warnings as errors in CI
- Prefer explicit error types over `Box<dyn Error>`
- Use `anyhow` for application errors, `thiserror` for library errors
- Async runtime: tokio
- No `unwrap()` or `expect()` in production code paths

### Error Handling Pattern

```rust
// Prefer Result types with proper error context
pub async fn fetch_thread(url: &str) -> Result<Thread, AppError> {
    // Implementation with proper error propagation
}
```

### Testing

- Unit tests in same file as implementation (Rust convention)
- Integration tests in `tests/` directory
- Use test fixtures for Bluesky API responses (avoid live API calls in tests)
- Mock HTTP responses for deterministic testing

## Container Build

### Dockerfile Strategy

Multi-stage build:

1. **Builder stage**: Full Rust toolchain, compile release binary
2. **Runtime stage**: Distroless base (gcr.io/distroless/cc-debian12), copy binary only

Key requirements:
- Statically link dependencies where possible
- Binary at `/app/bluesky-concatenator`
- Run as non-root user
- Expose port via `EXPOSE` directive (actual port from env var)
- Health check support

## Helm Chart

### Chart Structure

Follow standard Helm best practices:

- Chart.yaml: API version v2, appropriate keywords
- values.yaml: Sensible defaults, well-commented
- deployment.yaml: Include health probes, resource limits, security context
- service.yaml: ClusterIP service (Ingress handles external access)
- ingress.yaml: Optional, disabled by default

### Required Kubernetes Resources

```yaml
# Deployment
- replicas: 2 (for HA)
- Health probes configured
- Resource limits set
- Security context: runAsNonRoot, readOnlyRootFilesystem
- Environment variables from ConfigMap

# Service
- Type: ClusterIP
- Port: 80 -> container port

# ConfigMap
- All configuration environment variables
```

## Development Workflow

1. Make changes
2. Run `cargo clippy && cargo test`
3. Test locally with `cargo run`
4. Build container and test in local k8s cluster
5. Commit (conventional commits format)

## Do Not

- Do not commit secrets or API keys
- Do not use blocking I/O operations (use async equivalently)
- Do not hardcode configuration values
- Do not write to local filesystem (stateless requirement)
- Do not include emoji in code or comments
- Do not add JavaScript unless absolutely necessary for functionality
- Do not use external CDNs or dependencies in HTML output

## Additional Context

When encountering unfamiliar Bluesky/AT Protocol concepts:
1. Check https://docs.bsky.app/ first
2. Search crates.io for existing solutions
3. Ask for clarification on ambiguous API behaviors

When selecting dependencies:
- Prefer widely-used, well-maintained crates
- Check recent commit activity and issue resolution
- Prefer async-compatible libraries (tokio ecosystem)

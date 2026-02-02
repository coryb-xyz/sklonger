# sklonger

A lightweight web service that fetches Bluesky threads (self-reply chains) and renders them as clean, readable HTML pages.

## What it does

When someone posts a long thread on Bluesky as a series of self-replies, skeet-longer collects all those posts and displays them as a single, seamless document. This makes long-form content easier to read and share.

## Usage

### Via URL parameter

```
https://sklonger.app/?url=https://bsky.app/profile/user.bsky.social/post/abc123
```

### Via direct path (drop-in replacement for bsky.app)

Simply replace `bsky.app` with your instance's domain:

```
https://bsky.app/profile/user.bsky.social/post/abc123
```

becomes:

```
https://sklonger.app/profile/user.bsky.social/post/abc123
```

## Features

- Fetches complete self-reply thread chains
- Renders embedded images and videos
- Supports external link cards
- Light/dark mode (follows system preference)
- Mobile-responsive design
- No JavaScript required for reading
- Kubernetes-ready with health endpoints

## Running locally

```bash
cargo run
```

Then visit `http://localhost:8080/?url=<bluesky-post-url>`

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | HTTP server port |
| `LOG_LEVEL` | `info` | Logging verbosity (trace, debug, info, warn, error) |
| `BLUESKY_API_URL` | `https://public.api.bsky.app` | AT Protocol API endpoint |
| `REQUEST_TIMEOUT_SECONDS` | `10` | HTTP client timeout |

## Docker

```bash
docker build -t skeet-longer .
docker run -p 8080:8080 skeet-longer
```

## Kubernetes

A Helm chart is provided in `skeet-longer-helm/`:

```bash
helm install skeet-longer ./skeet-longer-helm
```

## Health endpoints

- `GET /health/live` - Liveness probe (always returns 200 if running)
- `GET /health/ready` - Readiness probe (200 if Bluesky API is reachable)

## License

MIT

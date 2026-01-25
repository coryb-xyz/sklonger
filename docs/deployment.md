# Configuration and Deployment

This document covers how to configure and run skeet-longer in various environments.

## Configuration

skeet-longer follows the [12-factor app](https://12factor.net/) methodology. All configuration is done through environment variables.

### Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `PORT` | integer | 8080 | HTTP server port |
| `LOG_LEVEL` | string | info | Logging verbosity |
| `BLUESKY_API_URL` | string | https://public.api.bsky.app | Bluesky API base URL |
| `REQUEST_TIMEOUT_SECONDS` | integer | 10 | HTTP client timeout |

### Log Levels

From most to least verbose:
- `trace`: Very detailed debugging, including individual API calls
- `debug`: Useful for debugging, includes request details
- `info`: Normal operation, request summaries
- `warn`: Potential issues that don't prevent operation
- `error`: Errors that affect request handling

For production, use `info`. For debugging, use `debug` or `trace`.

## Running Locally

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- (Optional) Docker for container builds

### Development Build

```bash
# Clone and enter the directory
cd skeet-longer

# Build debug binary
cargo build

# Run with default settings
cargo run

# Run with custom settings
PORT=3000 LOG_LEVEL=debug cargo run
```

The server will start on `http://localhost:8080` (or your configured port).

### Release Build

```bash
# Optimized release build
cargo build --release

# Binary is at target/release/skeet-longer
./target/release/skeet-longer
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_bluesky_url
```

### Linting

```bash
# Check for common issues
cargo clippy

# Auto-format code
cargo fmt
```

## Docker

### Building the Container

The Dockerfile uses a multi-stage build:
1. **Builder stage**: Full Rust toolchain compiles the release binary
2. **Runtime stage**: Minimal distroless image with just the binary

```bash
# Build the image
docker build -t skeet-longer:dev .

# View image size (should be ~30MB)
docker images skeet-longer:dev
```

### Running the Container

```bash
# Basic run
docker run -p 8080:8080 skeet-longer:dev

# With custom configuration
docker run -p 3000:3000 \
  -e PORT=3000 \
  -e LOG_LEVEL=debug \
  skeet-longer:dev

# Detached mode
docker run -d -p 8080:8080 --name skeet-longer skeet-longer:dev
```

### Health Checks

The container responds to health checks on:
- `/health/live` - Returns 200 if process is running
- `/health/ready` - Returns 200 if Bluesky API is reachable

```bash
# Check if running
curl http://localhost:8080/health/live

# Check if ready
curl http://localhost:8080/health/ready
```

## Kubernetes Deployment

The `helm/` directory contains a Helm chart for Kubernetes deployment.

### Chart Structure

```
helm/skeet-longer/
├── Chart.yaml           # Chart metadata
├── values.yaml          # Default configuration
└── templates/
    ├── deployment.yaml  # Pod specification
    ├── service.yaml     # ClusterIP service
    ├── configmap.yaml   # Environment variables
    ├── hpa.yaml         # Horizontal Pod Autoscaler (optional)
    └── ingress.yaml     # Ingress (optional)
```

### Installing the Chart

```bash
# Lint the chart first
helm lint ./helm/skeet-longer

# Preview what will be deployed
helm template my-release ./helm/skeet-longer

# Install
helm install my-release ./helm/skeet-longer

# Install with custom values
helm install my-release ./helm/skeet-longer \
  --set config.logLevel=debug \
  --set replicaCount=3
```

### Customizing Values

Create a `values-prod.yaml` file:

```yaml
replicaCount: 3

image:
  repository: your-registry.com/skeet-longer
  tag: v1.0.0

config:
  logLevel: info

resources:
  limits:
    cpu: 500m
    memory: 256Mi
  requests:
    cpu: 100m
    memory: 128Mi

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: skeet.example.com
      paths:
        - path: /
          pathType: Prefix
```

Then install:

```bash
helm install prod ./helm/skeet-longer -f values-prod.yaml
```

### Upgrading

```bash
# Upgrade with new image
helm upgrade my-release ./helm/skeet-longer \
  --set image.tag=v1.1.0

# Rollback if needed
helm rollback my-release 1
```

### Uninstalling

```bash
helm uninstall my-release
```

## Kubernetes Resources

### Deployment

The deployment includes:
- **Replicas**: 2 by default for high availability
- **Health probes**: Liveness and readiness checks
- **Resource limits**: CPU and memory constraints
- **Security context**: Non-root user, read-only filesystem

### Service

A ClusterIP service exposes port 80, routing to container port 8080.

### ConfigMap

All configuration is stored in a ConfigMap and mounted as environment variables.

### HPA (Horizontal Pod Autoscaler)

Optional autoscaling based on CPU usage:
- Min replicas: 2
- Max replicas: 10
- Target CPU: 80%

Enable with:
```bash
helm install my-release ./helm/skeet-longer \
  --set autoscaling.enabled=true
```

### Ingress

Optional ingress for external access. Disabled by default.

## Monitoring

### Logs

Logs are written to stdout in a structured format:

```
2024-01-15T10:30:00.000Z  INFO skeet_longer::handlers: fetching thread url="https://bsky.app/..."
2024-01-15T10:30:01.000Z  INFO skeet_longer::handlers: thread fetched successfully author="user.bsky.social" post_count=5
```

With Kubernetes, use:
```bash
kubectl logs -f deployment/my-release-skeet-longer
```

### Metrics

The application doesn't expose Prometheus metrics by default, but you can add them by:
1. Adding the `axum-prometheus` crate
2. Exposing a `/metrics` endpoint
3. Configuring a ServiceMonitor in Kubernetes

### Health Endpoints

| Endpoint | Purpose | Success | Failure |
|----------|---------|---------|---------|
| `/health/live` | Process alive? | 200 OK | (process dead) |
| `/health/ready` | Can serve traffic? | 200 OK | 503 Service Unavailable |

Configure these in Kubernetes probes:

```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: http
  initialDelaySeconds: 5
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health/ready
    port: http
  initialDelaySeconds: 5
  periodSeconds: 10
```

## Troubleshooting

### Application Won't Start

**Symptom**: Error on startup about port binding

**Cause**: Port already in use or insufficient permissions

**Solution**:
```bash
# Check what's using the port
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows

# Use a different port
PORT=3000 cargo run
```

### Can't Reach Bluesky API

**Symptom**: All requests return 503, `/health/ready` fails

**Cause**: Network issue or API unavailable

**Solution**:
```bash
# Test API connectivity
curl https://public.api.bsky.app/xrpc/com.atproto.identity.resolveHandle?handle=bsky.app

# Check your network/proxy settings
# If behind corporate firewall, may need to configure proxy
```

### Threads Not Loading Fully

**Symptom**: Thread only shows partial posts

**Cause**: The thread might include posts from other authors (not a self-reply chain)

**Solution**: This is expected behavior. skeet-longer only includes posts by the original author in their self-reply chain.

### High Memory Usage

**Symptom**: Container OOMKilled in Kubernetes

**Cause**: Resource limits too low for traffic

**Solution**:
```yaml
resources:
  limits:
    memory: 256Mi  # Increase if needed
  requests:
    memory: 128Mi
```

### Rate Limiting

**Symptom**: 429 errors from application

**Cause**: Too many requests to Bluesky API

**Solution**: Bluesky rate limits are per-IP. In production:
- Use caching (not currently implemented)
- Distribute across multiple IPs
- Implement request queuing

## Security Considerations

### Input Validation

All user input (URLs) is validated before use. The URL parser rejects:
- Non-Bluesky domains
- Non-post URLs
- Malformed URLs

### HTML Escaping

All user content from Bluesky is HTML-escaped before rendering, preventing XSS attacks.

### Container Security

The container runs with:
- Non-root user (UID 65532)
- Read-only root filesystem
- No capabilities beyond defaults
- Distroless base (minimal attack surface)

### Network Security

- No outbound connections except to Bluesky API
- No cookies or session state
- No persistent storage

## Performance Tuning

### Tokio Runtime

The async runtime uses sensible defaults. For high-load scenarios, you can tune:

```rust
// In main.rs, if needed
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() { ... }
```

### Connection Pooling

The HTTP client (reqwest) uses connection pooling by default. For high-load:
- Increase pool size if needed
- Monitor connection reuse

### Response Size

HTML responses are typically 50-200KB depending on thread length. Consider:
- Enabling gzip compression at the ingress level
- CDN caching for static assets (if added later)

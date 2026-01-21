# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source for dependency caching
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies only (cached unless Cargo.toml/lock changes)
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Touch main.rs to invalidate cache and rebuild with real code
RUN touch src/main.rs && \
    cargo build --release

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy the compiled binary
COPY --from=builder /app/target/release/skeet-longer /app/skeet-longer

WORKDIR /app

# Expose the default port (actual port configured via env var)
EXPOSE 8080

# Run as non-root (distroless:nonroot already runs as uid 65532)
USER nonroot

# Set default environment variables
ENV PORT=8080 \
    LOG_LEVEL=info \
    BLUESKY_API_URL=https://public.api.bsky.app \
    REQUEST_TIMEOUT_SECONDS=10

# Run the binary
ENTRYPOINT ["/app/skeet-longer"]

# Multi-stage Dockerfile for DeepWiki-RS
FROM rust:1.75 AS builder

WORKDIR /app

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Build the application with caching
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u deepwiki

# Copy the binary from builder
COPY --from=builder /app/target/release/deepwiki-rs /usr/local/bin/

# Set permissions
RUN chmod +x /usr/local/bin/deepwiki-rs

# Switch to app user
USER deepwiki

# Set working directory
WORKDIR /app

# Create data directory
RUN mkdir -p /app/data

# Default command
CMD ["deepwiki-rs"]

# Expose port (if needed)
# EXPOSE 8080
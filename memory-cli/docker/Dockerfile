# Multi-stage build for memory-cli
FROM rust:1.75-slim AS builder

# Install required dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy memory-core
COPY memory-core/ memory-core/

# Copy storage crates
COPY memory-storage-turso/ memory-storage-turso/
COPY memory-storage-redb/ memory-storage-redb/

# Copy memory-cli
COPY memory-cli/ memory-cli/

# Build the application
RUN cd memory-cli && cargo build --release --features full

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd --create-home --shell /bin/bash memory

# Set working directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/memory-cli/target/release/memory-cli /usr/local/bin/memory-cli

# Create directories for configuration and data
RUN mkdir -p /app/config /app/data /app/backups /app/logs

# Set ownership
RUN chown -R memory:memory /app

# Switch to non-root user
USER memory

# Set environment variables
ENV RUST_LOG=info
ENV MEMORY_CONFIG=/app/config/memory-cli.toml

# Expose metrics port (if we add HTTP server later)
# EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD memory-cli health check --format json | grep -q '"overall_status":"Healthy"'

# Default command
ENTRYPOINT ["memory-cli"]
CMD ["--help"]
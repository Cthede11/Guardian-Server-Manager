# Host Daemon Dockerfile
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/hostd /app/hostd

# Create necessary directories
RUN mkdir -p /data /configs && chown appuser:appuser /data /configs

# Switch to app user
USER appuser

# Expose ports
EXPOSE 8080 9090

# Set environment variables
ENV RUST_LOG=info
ENV HOSTD_CONFIG=/configs/hostd.yaml

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/status || exit 1

# Run the application
CMD ["./hostd", "--config", "/configs/hostd.yaml", "--daemon"]

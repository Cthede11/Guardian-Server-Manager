# GPU Worker Dockerfile
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libvulkan-dev \
    vulkan-tools \
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
    libvulkan1 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/gpu-worker /app/gpu-worker

# Create cache directory
RUN mkdir -p /cache && chown appuser:appuser /cache

# Switch to app user
USER appuser

# Expose port (if needed for health checks)
EXPOSE 8081

# Set environment variables
ENV RUST_LOG=info
ENV GPU_WORKER_IPC=shm

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep gpu-worker || exit 1

# Run the application
CMD ["./gpu-worker"]
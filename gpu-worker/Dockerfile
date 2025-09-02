FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    libudev-dev \
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
    libasound2 \
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false gpuworker

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/gpu-worker /app/gpu-worker

# Change ownership
RUN chown -R gpuworker:gpuworker /app

# Switch to app user
USER gpuworker

# Expose port (if needed for health checks)
EXPOSE 8081

# Run the application
CMD ["./gpu-worker"]

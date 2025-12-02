# ============================================
# Stage 1: Build Cache Layer
# ============================================
FROM rust:1.83-slim-bookworm AS chef

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef
WORKDIR /app

# ============================================
# Stage 2: Prepare Recipe (Dependency Analysis)
# ============================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ============================================
# Stage 3: Build Dependencies
# ============================================
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build dependencies first (this layer will be cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and build the application
COPY .  .

# Build the actual binary
RUN cargo build --release --bin ms1

# ============================================
# Stage 4: Runtime Image (Minimal & Secure)
# ============================================
FROM debian:bookworm-slim

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1001 -s /bin/bash appuser

WORKDIR /app

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/ms1 /app/ms1

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose the application port
EXPOSE 3000

# Health check endpoint (using /ping route from your code)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/ping || exit 1

# Run the binary
ENTRYPOINT ["/app/ms1"]
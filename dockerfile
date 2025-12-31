# ============================================
# Production-Ready Multi-Stage Dockerfile
# Optimized with cargo-chef + sccache
# 10-20x faster than QEMU emulation
# ============================================

# ============================================
# Stage 1: Prepare cargo-chef
# ============================================
FROM rust:1.84-slim-bookworm AS chef

# Install cargo-chef for optimal layer caching
RUN cargo install cargo-chef --version 0.1.67

WORKDIR /app

# ============================================
# Stage 2: Analyze dependencies
# ============================================
FROM chef AS planner

COPY .  .

# Generate dependency "recipe"
RUN cargo chef prepare --recipe-path recipe.json

# ============================================
# Stage 3: Build dependencies (CACHED LAYER)
# ============================================
FROM chef AS builder

# Install build-time dependencies + sccache
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install sccache for compilation caching
RUN wget -q https://github.com/mozilla/sccache/releases/download/v0.7.7/sccache-v0.7.7-$(uname -m)-unknown-linux-musl.tar.gz && \
    tar xzf sccache-v0.7.7-$(uname -m)-unknown-linux-musl.tar.gz && \
    mv sccache-v0.7.7-$(uname -m)-unknown-linux-musl/sccache /usr/local/bin/ && \
    chmod +x /usr/local/bin/sccache && \
    rm -rf sccache-v0.7.7-$(uname -m)-unknown-linux-musl*

# Configure sccache
ENV RUSTC_WRAPPER=/usr/local/bin/sccache
ENV SCCACHE_DIR=/sccache
ENV CARGO_INCREMENTAL=0

WORKDIR /app

# Copy the recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies ONLY (this layer is heavily cached)
# This is the magic of cargo-chef - it builds deps without your source code
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache \
    cargo chef cook --release --recipe-path recipe. json

# ============================================
# Stage 4: Build application
# ============================================
# Copy actual source code
COPY . .

# Build the application (dependencies already compiled)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache \
    cargo build --release --bin ms1

# Show compilation cache statistics (for debugging)
RUN sccache --show-stats || true

# ============================================
# Stage 5: Runtime Image (Minimal & Secure)
# ============================================
FROM debian:bookworm-slim

# Install ONLY runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# ============================================
# Security: Non-root User
# ============================================
RUN useradd -m -u 1001 -s /bin/bash appuser

WORKDIR /app

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/ms1 /app/ms1

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# ============================================
# Runtime Configuration
# ============================================
EXPOSE 3000

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/ping || exit 1

# ============================================
# Entrypoint
# ============================================
ENTRYPOINT ["/app/ms1"]
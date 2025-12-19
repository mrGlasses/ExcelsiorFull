# ============================================
# Production-Ready Multi-Stage Dockerfile
# Works for: Local Development + Oracle VPS Production
# ============================================

# ============================================
# Stage 1: Dependency Builder (Cached Layer)
# ============================================
FROM rust:1.84-slim-bookworm AS builder

# Install build-time dependencies
# These are needed to compile Rust code with native dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ============================================
# CRITICAL: Dependency Caching Strategy
# ============================================
# Copy only Cargo manifests first to create a cached dependency layer
# This layer will only rebuild if Cargo.toml or Cargo.lock changes
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file to build dependencies
# This allows Docker to cache the compiled dependencies separately from your code
RUN mkdir -p src && \
    echo "fn main() {println!(\"Dummy for dependency caching\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# ============================================
# Stage 2: Application Builder
# ============================================
# Copy the actual source code
COPY .  .

# Touch main.rs to ensure it's rebuilt with actual code
# This forces Cargo to recognize the file change
RUN touch src/main. rs

# Build the actual application binary
# Dependencies are already compiled from Stage 1 (cached)
RUN cargo build --release --bin ms1

# ============================================
# Stage 3: Runtime Image (Minimal & Secure)
# ============================================
FROM debian:bookworm-slim

# Install ONLY runtime dependencies (no build tools)
# This keeps the image small and secure
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# ============================================
# Security: Non-root User
# ============================================
# Create a dedicated user to run the application
# This follows the principle of least privilege
RUN useradd -m -u 1001 -s /bin/bash appuser

WORKDIR /app

# Copy the compiled binary from the builder stage
# Only the final binary is included, not source code or build artifacts
COPY --from=builder /app/target/release/ms1 /app/ms1

# Change ownership to the non-root user
RUN chown -R appuser:appuser /app

# Switch to the non-root user
USER appuser

# ============================================
# Runtime Configuration
# ============================================
# Expose the application port
# This should match MS_PORT in your . env file
EXPOSE 3000

# Health check endpoint
# Uses the /ping route defined in your Axum router
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/ping || exit 1

# ============================================
# Entrypoint
# ============================================
# Run the binary
ENTRYPOINT ["/app/ms1"]
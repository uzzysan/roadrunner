# Builder stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install dependencies (build tools + PostgreSQL dev libs)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    libpq-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli using binstall (faster) or cargo install with retry
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres \
    || (sleep 5 && cargo install sqlx-cli --no-default-features --features native-tls,postgres)

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Set offline mode for sqlx
ENV SQLX_OFFLINE=true

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/roadrunner /app/roadrunner

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Copy sqlx-cli for migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run migrations and start app
CMD ["/bin/sh", "-c", "sqlx migrate run && /app/roadrunner"]

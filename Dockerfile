# Builder stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy all source code at once
COPY . .

# Build release binary (with all dependencies)
RUN cargo build --release

# Install sqlx-cli
RUN cargo install sqlx-cli --version 0.7.3 --no-default-features --features postgres

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy sqlx-cli from builder
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

# Copy binary from builder
COPY --from=builder /app/target/release/roadrunner /app/roadrunner

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run migrations and start app
CMD ["/bin/sh", "-c", "until pg_isready -h postgres -U ${DB_USER:-roadrunner}; do echo 'Waiting for postgres...'; sleep 2; done; sqlx migrate run && /app/roadrunner"]

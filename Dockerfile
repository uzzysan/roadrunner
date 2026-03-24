# Builder stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    libpq-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

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

# Download pre-compiled sqlx-cli instead of building
RUN curl -L https://github.com/launchbadge/sqlx/releases/download/v0.7.3/sqlx-cli-v0.7.3-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin

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

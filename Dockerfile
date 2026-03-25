# Stage 1: Build the Rust binary
FROM rust:1-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies by building a dummy project first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() { println!(\"dummy\"); }" > src/main.rs \
    && echo "" > src/lib.rs \
    && cargo build --release \
    && rm -rf src

# Copy the actual source code and .sqlx directory
COPY . .

# Build the real binary (forced to build offline using .sqlx)
ENV SQLX_OFFLINE=true
RUN touch src/main.rs \
    && cargo build --release

# Stage 2: Minimal runtime image
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

# Download precompiled sqlx-cli for running migrations
RUN curl -L https://github.com/launchbadge/sqlx/releases/download/v0.7.3/sqlx-cli-v0.7.3-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin || \
    (apt-get update && apt-get install -y cargo && cargo install sqlx-cli --version 0.7.3 --no-default-features --features postgres)

# Copy the pre-built binary from the builder stage
COPY --from=builder /app/target/release/roadrunner /app/roadrunner
RUN chmod +x /app/roadrunner

# Copy migrations
COPY migrations /app/migrations

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run migrations and start app
CMD ["/bin/sh", "-c", "until pg_isready -h postgres -U ${DB_USER:-roadrunner}; do echo 'Waiting for postgres...'; sleep 2; done; sqlx migrate run && /app/roadrunner"]

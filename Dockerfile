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
RUN cargo clean -p roadrunner \
    && cargo build --release

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the pre-built binary from the builder stage.
# Migrations are embedded into the binary at compile time by sqlx::migrate!() (src/main.rs) and
# run automatically on every startup — no separate migration step, CLI, or copied SQL files
# needed at runtime. This also means migrations run correctly for a bare (non-Docker) deploy of
# this binary, not just the container path.
COPY --from=builder /app/target/release/roadrunner /app/roadrunner

RUN chmod +x /app/roadrunner

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["/app/roadrunner"]

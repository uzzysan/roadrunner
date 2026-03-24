FROM rust:1.75-slim-bookworm

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release

# Install sqlx-cli
RUN cargo install sqlx-cli --version 0.7.3 --no-default-features --features postgres

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run migrations and start app
CMD ["/bin/sh", "-c", "until pg_isready -h postgres -U ${DB_USER:-roadrunner}; do echo 'Waiting for postgres...'; sleep 2; done; sqlx migrate run && /app/target/release/roadrunner"]

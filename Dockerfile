# Build stage
FROM rust:1.83-bookworm AS builder

# Install build dependencies including PostgreSQL development libraries
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Diesel CLI with PostgreSQL support only
RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Copy source code and migrations
COPY src ./src
COPY migrations ./migrations
COPY diesel.toml ./

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies for PostgreSQL client, Chrome headless, and Diesel
RUN apt-get update && apt-get install -y \
    # PostgreSQL client libraries
    libpq5 \
    # Dependencies for headless Chrome
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libatspi2.0-0 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libwayland-client0 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxkbcommon0 \
    libxrandr2 \
    xdg-utils \
    libu2f-udev \
    libvulkan1 \
    && rm -rf /var/lib/apt/lists/*

# Copy Diesel CLI from builder
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# Copy the built application
COPY --from=builder /app/target/release/gmaps_review_notif /usr/local/bin/gmaps_review_notif

# Copy migrations for Diesel
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/diesel.toml /app/diesel.toml

WORKDIR /app

# Ensure required environment variables are set (will fail at runtime if not provided)
# DATABASE_URL and DISCORD_TOKEN must be set via docker run -e or docker-compose

# Run database setup (migrations) and then start the application
CMD diesel setup --migration-dir /app/migrations && \
    gmaps_review_notif

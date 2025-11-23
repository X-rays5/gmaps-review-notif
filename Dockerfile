# Diesel CLI builder stage
FROM rust:1.91-slim-trixie AS diesel-builder

# Install PostgreSQL dev libraries needed for diesel_cli
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Diesel CLI with PostgreSQL support only using cache mounts
# Using specific version for reproducibility
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install diesel_cli --version 2.3.0 --no-default-features --features postgres

# Cargo chef stage for recipe generation
FROM rust:1.91-slim-trixie AS chef

# Install dependencies needed for cargo-chef compilation
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install cargo-chef --locked
WORKDIR /app

# Generate cargo-chef recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Application builder stage
FROM rust:1.91-slim-trixie AS builder

# Install build dependencies including PostgreSQL development libraries
# These are needed for both sccache compilation and the application build
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install sccache for compilation caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install sccache --locked

# Set up sccache as the compiler wrapper
ENV RUSTC_WRAPPER=/usr/local/cargo/bin/sccache

WORKDIR /app

# Copy cargo-chef recipe
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies using cargo-chef - this is the caching Docker layer!
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cache/sccache \
    cargo chef cook --release --recipe-path recipe.json

# Copy actual source code and migrations
COPY src ./src
COPY migrations ./migrations
COPY diesel.toml ./
COPY Cargo.toml Cargo.lock build.rs ./

# Build the application in release mode with the actual source
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cache/sccache \
    cargo build --release && \
    cp /app/target/release/gmaps_review_notif /tmp/gmaps_review_notif

# Copy binary from temp location to final location (separate layer to persist after cache unmount)
RUN cp /tmp/gmaps_review_notif /usr/local/bin/gmaps_review_notif && chmod +x /usr/local/bin/gmaps_review_notif

# Runtime stage
FROM debian:trixie-slim

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

# Copy Diesel CLI from diesel-builder stage
COPY --from=diesel-builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# Copy the built application from builder stage
COPY --from=builder /usr/local/bin/gmaps_review_notif /usr/local/bin/gmaps_review_notif

# Copy migrations for Diesel
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/diesel.toml /app/diesel.toml

WORKDIR /app

# Ensure required environment variables are set (will fail at runtime if not provided)
# DATABASE_URL and DISCORD_TOKEN must be set via docker run -e or docker-compose

# Run database setup (migrations) and then start the application
# Using exec form to properly handle signals
CMD ["/bin/sh", "-c", "diesel setup --migration-dir /app/migrations && gmaps_review_notif"]

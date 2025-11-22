# Google Maps Review Notifier

A Discord bot that monitors Google Maps reviews and sends notifications to Discord channels. The bot uses web scraping to check for new reviews and can be configured to run on a schedule.

## Features

- Monitor Google Maps user profiles for new reviews
- Send notifications to Discord channels via webhooks
- Scheduled review fetching with configurable intervals
- PostgreSQL database for tracking reviews and followed users
- Docker support for easy deployment

## Docker Deployment

### Quick Start with Docker Compose

1. Copy the environment template:
```bash
cp .env.example .env
```

2. Edit `.env` and set your Discord bot token:
```bash
DISCORD_TOKEN=your_discord_bot_token_here
```

3. Start the services:
```bash
docker-compose up -d
```

The bot will automatically:
- Set up the PostgreSQL database
- Run Diesel migrations
- Start monitoring for reviews

### Using Pre-built Images

Pull the latest image from GitHub Container Registry:
```bash
docker pull ghcr.io/x-rays5/gmaps-review-notif:latest
```

Run with required environment variables:
```bash
docker run -d \
  -e DISCORD_TOKEN=your_token_here \
  -e DATABASE_URL=postgres://user:password@host:5432/dbname \
  ghcr.io/x-rays5/gmaps-review-notif:latest
```

### Building Locally

Build the Docker image:
```bash
docker build -t gmaps-review-notif .
```

### Multi-Architecture Support

The published images support both x64 (amd64) and ARM64 (arm64) architectures, suitable for:
- x64: Most cloud providers, desktop computers
- ARM64: Raspberry Pi 4/5, AWS Graviton, Apple Silicon (via Docker Desktop)

## Environment Variables

### Required
- `DISCORD_TOKEN`: Your Discord bot token
- `DATABASE_URL`: PostgreSQL connection string (format: `postgres://user:password@host:port/database`)

### Optional
- `STAR_TEXT`: Text to use for star ratings (default: ⭐)
- `FETCH_REVIEWS_ON_STARTUP`: Fetch reviews when bot starts (default: true)
- `NEW_REVIEW_FETCH_INTERVAL`: Cron schedule for review checks (default: `0 0 */6 * * *` - every 6 hours)
- `REVIEW_AGE_LIMIT_HOURS`: Only notify about reviews newer than this (default: 24)
- `RUST_LOG`: Logging level (default: info, options: error, warn, info, debug, trace)

## Local Development

### Prerequisites
- Rust 1.83 or later
- PostgreSQL 18
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`

### Setup
1. Install dependencies:
```bash
cargo build
```

2. Set up the database:
```bash
export DATABASE_URL=postgres://user:password@localhost/dbname
diesel setup
```

3. Run the application:
```bash
export DISCORD_TOKEN=your_token_here
cargo run
```

## CI/CD

The project includes two GitHub Actions workflows:

1. **CI** (`ci.yml`): Runs on every push/PR
   - Tests Diesel migrations
   - Builds the project

2. **Docker Publish** (`docker-publish.yml`): Runs on tag pushes (e.g., `v1.0.0`)
   - Builds multi-architecture Docker images (x64 and ARM64)
   - Publishes to GitHub Container Registry
   - Creates a GitHub release with automatic release notes

### Creating a Release

To trigger a new release:
```bash
git tag v1.0.0
git push origin v1.0.0
```

This will automatically build and publish Docker images and create a GitHub release.

## License

See LICENSE file for details.

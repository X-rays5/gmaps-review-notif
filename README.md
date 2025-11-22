# Google Maps Review Notifier

A Discord bot that monitors Google Maps user profiles for new reviews and sends notifications to Discord channels. The bot uses headless Chrome for web scraping and runs on a configurable schedule to check for new reviews.

> [!NOTE]
> Only the Docker environment is officially tested for deployment functionality. Local development on Windows and Linux should work fine for development purposes.

## Features

- Monitor Google Maps user profiles for new reviews
- Send notifications to Discord channels
- Scheduled review fetching with configurable cron intervals
- PostgreSQL database for tracking reviews and followed users
- Headless Chrome web scraping
- Multi-architecture Docker support (x64 and ARM64)

## Quick Start

See [DOCKER.md](DOCKER.md) for detailed Docker deployment instructions.

**TL;DR:**
1. Edit `DISCORD_TOKEN` in `docker-compose.yml`
2. Run `docker-compose up -d`

The bot will automatically set up the database, run migrations, and start monitoring.

## Configuration

The bot is configured via environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DISCORD_TOKEN` | **Yes** | - | Your Discord bot token |
| `DATABASE_URL` | **Yes** | - | PostgreSQL connection string |
| `STAR_TEXT` | No | ⭐ | Text to use for star ratings |
| `FETCH_REVIEWS_ON_STARTUP` | No | `true` | Fetch reviews when bot starts |
| `NEW_REVIEW_FETCH_INTERVAL` | No | `0 0 */6 * * *` | Cron schedule for review checks (every 6 hours) |
| `REVIEW_AGE_LIMIT_HOURS` | No | `24` | Only notify about reviews newer than this |
| `RUST_LOG` | No | `info` | Logging level (error, warn, info, debug, trace) |

Database URL format: `postgres://user:password@host:port/database`

## Architecture

- **Language:** Rust
- **Database:** PostgreSQL with Diesel ORM
- **Discord:** poise framework
- **Web Scraping:** headless_chrome
- **Scheduling:** tokio-cron-scheduler

## Development

### Local Development

For local development:

1. Install Rust 1.83+, PostgreSQL 18, and Diesel CLI
2. Create a `.env` file with your configuration:
   ```env
   DATABASE_URL=postgres://user:password@localhost/dbname
   DISCORD_TOKEN=your_token_here
   ```
3. Run database setup: `diesel setup`
4. Build and run: `cargo run`

Docker is recommended for deployment.

## CI/CD

See [RELEASING.md](RELEASING.md) for information about the release process and CI/CD workflows.

## License

See LICENSE file for details.

# Docker Deployment Guide

This guide provides detailed instructions for deploying the Google Maps Review Notifier using Docker.

## Quick Start

### Prerequisites
- Docker and Docker Compose installed
- A Discord bot token

### Start Services

```bash
docker-compose up -d
```

This will:
1. Pull the pre-built application image from GitHub Container Registry
2. Pull the PostgreSQL image
3. Start both services
4. Run database migrations automatically
5. Start the bot

> [!IMPORTANT]
> Edit the `DISCORD_TOKEN` value in `docker-compose.yml` before starting.

### Verify Deployment

Check logs:
```bash
docker-compose logs -f app
```

Check if containers are running:
```bash
docker-compose ps
```

## Configuration

The `docker-compose.yml` file uses the pre-built image from GitHub Container Registry by default. All configuration is done through environment variables in the compose file.

### Configure PostgreSQL

The included `docker-compose.yml` provides a PostgreSQL container for convenience, but you can use any PostgreSQL instance by updating the `DATABASE_URL`:

```yaml
environment:
  DATABASE_URL: postgres://username:password@your-postgres-host:5432/dbname
```

If using a different PostgreSQL instance, you can remove the `postgres` service from `docker-compose.yml`.

### Building Locally

To build the image locally instead of using the pre-built one, uncomment the `build` section in `docker-compose.yml`:

```yaml
app:
  # image: ghcr.io/x-rays5/gmaps-review-notif:latest
  build:
    context: .
    dockerfile: Dockerfile
```

Or build manually with BuildKit (recommended for optimal caching):
```bash
DOCKER_BUILDKIT=1 docker build -t gmaps-review-notif:custom .
```

> [!NOTE]
> The Dockerfile uses BuildKit features including cache mounts for optimal build performance. BuildKit is enabled by default in Docker 23.0+. For older versions, set `DOCKER_BUILDKIT=1` explicitly.

#### Build Optimizations

The Dockerfile includes several optimizations for faster builds:
- **cargo-chef**: Smart dependency caching that works correctly with workspace dependencies
- **sccache**: Compilation caching to speed up incremental builds
- **BuildKit cache mounts**: Persistent caches for cargo registry and build artifacts across builds
- **Slim base images**: Faster downloads using `rust:1.91-slim-trixie`

These optimizations significantly reduce build times, especially on CI/CD pipelines and when building repeatedly during development.

## Troubleshooting

### Container Exits Immediately

Check logs for error messages:
```bash
docker-compose logs app
```

Common issues:
- "DISCORD_TOKEN must be set" - Update the token in docker-compose.yml
- "DATABASE_URL must be set" - Verify database connection string
- Database connection refused - Ensure PostgreSQL container is healthy (`docker-compose ps`)

### Chrome/Browser Issues

If you see Chrome-related errors, the Dockerfile includes all necessary libraries for headless Chrome with `--headless=new` support. If issues persist, check container logs for specific missing dependencies.

### Migration Failures

Run migrations manually if automatic migration fails:
```bash
docker-compose exec app diesel migration run
```

Revert and re-run if needed:
```bash
docker-compose exec app diesel migration revert
docker-compose exec app diesel migration run
```

## Rollback

To rollback to a previous version:

1. Update docker-compose.yml to use the specific version tag:
   ```yaml
   image: ghcr.io/x-rays5/gmaps-review-notif:v1.0.0
   ```

2. Restart services:
   ```bash
   docker-compose pull
   docker-compose up -d
   ```

> [!NOTE]
> Database migrations are not automatically rolled back. You'll need to manually revert them:
```bash
docker-compose exec app diesel migration revert
```

> [!CAUTION]
> Reverting migrations may cause data loss if the migrations contain destructive operations (e.g., dropping columns or tables).

## Multi-Architecture Support

The published images support:
- **linux/amd64**: Intel/AMD 64-bit processors
- **linux/arm64**: ARM 64-bit processors (Raspberry Pi 4/5, Apple Silicon, AWS Graviton)

Docker automatically pulls the correct architecture for your system.

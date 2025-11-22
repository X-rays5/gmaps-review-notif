# Docker Deployment Guide

This guide provides detailed instructions for deploying the Google Maps Review Notifier using Docker.

## Quick Start

### Prerequisites
- Docker and Docker Compose installed
- A Discord bot token
- PostgreSQL connection (provided by docker-compose)

### Step 1: Environment Configuration

Create a `.env` file from the template:
```bash
cp .env.example .env
```

Edit the `.env` file and set your Discord token:
```env
DISCORD_TOKEN=your_discord_bot_token_here
```

### Step 2: Start Services

```bash
docker-compose up -d
```

This will:
1. Pull the PostgreSQL image
2. Build the application image (first time only)
3. Start both services
4. Run database migrations automatically
5. Start the bot

### Step 3: Verify Deployment

Check logs:
```bash
docker-compose logs -f app
```

Check if containers are running:
```bash
docker-compose ps
```

## Using Pre-built Images

Instead of building locally, you can use pre-built images from GitHub Container Registry:

```yaml
# docker-compose.yml
services:
  app:
    image: ghcr.io/x-rays5/gmaps-review-notif:latest
    # Remove the 'build' section
```

Then just run:
```bash
docker-compose up -d
```

## Advanced Configuration

### Custom Database Configuration

To use an external PostgreSQL database, modify the `DATABASE_URL` in docker-compose.yml:

```yaml
environment:
  DATABASE_URL: postgres://username:password@external-host:5432/dbname
```

And remove or comment out the postgres service.

### Environment Variables

All available environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DISCORD_TOKEN` | Yes | - | Your Discord bot token |
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `STAR_TEXT` | No | ⭐ | Text representation for star ratings |
| `FETCH_REVIEWS_ON_STARTUP` | No | true | Fetch reviews when bot starts |
| `NEW_REVIEW_FETCH_INTERVAL` | No | 0 0 */6 * * * | Cron schedule for review checks |
| `REVIEW_AGE_LIMIT_HOURS` | No | 24 | Only notify for reviews newer than this |
| `RUST_LOG` | No | info | Logging level (error, warn, info, debug, trace) |

### Custom Build

Build with specific tag:
```bash
docker build -t gmaps-review-notif:custom .
```

Build for specific platform:
```bash
docker buildx build --platform linux/arm64 -t gmaps-review-notif:arm64 .
```

## Troubleshooting

### Container Exits Immediately

Check if required environment variables are set:
```bash
docker-compose logs app
```

Look for error messages like "DISCORD_TOKEN must be set" or "DATABASE_URL must be set".

### Database Connection Issues

1. Ensure PostgreSQL container is healthy:
```bash
docker-compose ps
```

2. Check PostgreSQL logs:
```bash
docker-compose logs postgres
```

3. Verify the DATABASE_URL format:
```
postgres://username:password@host:port/database
```

### Chrome/Browser Issues

If you see Chrome-related errors, ensure all dependencies are installed. The Dockerfile includes all necessary libraries for headless Chrome operation.

### Migration Failures

If migrations fail, you can run them manually:
```bash
docker-compose exec app diesel migration run
```

Or revert and rerun:
```bash
docker-compose exec app diesel migration revert
docker-compose exec app diesel migration run
```

## Maintenance

### Viewing Logs

```bash
# All services
docker-compose logs -f

# Just the app
docker-compose logs -f app

# Just the database
docker-compose logs -f postgres
```

### Updating the Application

1. Pull the latest image (if using pre-built):
```bash
docker-compose pull app
```

2. Restart services:
```bash
docker-compose up -d
```

### Backing Up the Database

```bash
docker-compose exec postgres pg_dump -U gmaps_user gmaps_reviews > backup.sql
```

### Restoring the Database

```bash
docker-compose exec -T postgres psql -U gmaps_user gmaps_reviews < backup.sql
```

### Stopping Services

```bash
# Stop but keep containers
docker-compose stop

# Stop and remove containers
docker-compose down

# Stop, remove containers, and delete volumes (WARNING: deletes data!)
docker-compose down -v
```

## Production Deployment

### Resource Limits

Add resource limits to docker-compose.yml:

```yaml
services:
  app:
    # ... existing config ...
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
```

### Security Hardening

1. Use secrets for sensitive data instead of environment variables
2. Run containers as non-root user
3. Use Docker secrets or external secret management
4. Enable Docker Content Trust
5. Regularly update base images

### Monitoring

Consider adding monitoring tools:
- Prometheus for metrics
- Grafana for visualization
- Loki for log aggregation

## Multi-Architecture Support

The published images support:
- **linux/amd64**: Intel/AMD 64-bit processors
- **linux/arm64**: ARM 64-bit processors (Raspberry Pi 4/5, Apple Silicon, AWS Graviton)

Docker automatically pulls the correct architecture for your system.

## CI/CD Integration

### Automated Releases

To trigger a new release:

1. Tag the commit:
```bash
git tag v1.0.0
git push origin v1.0.0
```

2. GitHub Actions will automatically:
   - Build multi-architecture images
   - Push to GitHub Container Registry
   - Create a GitHub release

### Version Tags

Images are tagged as:
- `v1.0.0` - Specific version
- `v1.0` - Minor version
- `v1` - Major version
- `latest` - Latest release

Choose the appropriate tag for your deployment strategy.

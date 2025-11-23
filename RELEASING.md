# Release Process

This document describes how to create new releases for the Google Maps Review Notifier.

## Automated Releases

The project uses GitHub Actions to automatically build, test, and publish releases.

### Creating a Release

To trigger a new release:

1. Ensure all changes are committed and pushed to the main branch
2. Tag the commit with a version number:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

3. GitHub Actions will automatically:
   - Build multi-architecture Docker images (linux/amd64 and linux/arm64)
   - Push images to GitHub Container Registry (ghcr.io)
   - Create a GitHub release with auto-generated release notes

### Version Tags

The CI/CD pipeline generates multiple Docker tags for each release:

| Git Tag | Docker Tags Generated |
|---------|----------------------|
| `v1.2.3` | `v1.2.3`, `v1.2`, `v1`, `latest` |
| `v2.0.0` | `v2.0.0`, `v2.0`, `v2`, `latest` |

**Note:** The `latest` tag always points to the most recently published release. When `v2.0.0` is released, `latest` moves from `v1.2.3` to `v2.0.0`.

**Recommendation:** Use semantic versioning (MAJOR.MINOR.PATCH) for all releases.

### Using Released Images

Pull a specific version:
```bash
docker pull ghcr.io/x-rays5/gmaps-review-notif:v1.0.0
```

Or use latest:
```bash
docker pull ghcr.io/x-rays5/gmaps-review-notif:latest
```

## CI/CD Workflows

### CI Workflow (`ci.yml`)

Runs on every push and pull request to `main`:
- Tests Diesel migrations (setup, run, rollback, re-run)
- Builds the project
- Validates code compiles successfully

### Docker Build and Publish Workflow (`docker-publish.yml`)

Triggers on version tag pushes (e.g., `v1.0.0`):
- Sets up QEMU for multi-architecture builds
- Builds Docker images for `linux/amd64` and `linux/arm64`
- Pushes to GitHub Container Registry
- Creates GitHub release with automatic release notes

## Manual Release Steps

If you need to create a release manually:

1. Build multi-arch images locally:
   ```bash
   docker buildx create --use
   docker buildx build --platform linux/amd64,linux/arm64 \
     -t ghcr.io/x-rays5/gmaps-review-notif:v1.0.0 \
     -t ghcr.io/x-rays5/gmaps-review-notif:latest \
     --push .
   ```

2. Create GitHub release manually via the web interface or CLI

## Release Checklist

Before creating a release:

- [ ] Documentation is up to date
- [ ] Version number in `Cargo.toml` matches the tag (e.g., tag `v1.0.0` should have version `1.0.0` in Cargo.toml)
- [ ] Version follows semantic versioning
- [ ] Database migrations tested

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

Note: Database migrations are not automatically rolled back. You'll need to manually revert them:
```bash
docker-compose exec app diesel migration revert
```

**Warning:** Reverting migrations may cause data loss if the migrations contain destructive operations (e.g., dropping columns or tables).

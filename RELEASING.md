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
- Calls the reusable `diesel-test.yml` workflow to test Diesel migrations
- Builds the Docker image to validate it compiles successfully

### Diesel Test Workflow (`diesel-test.yml`)

Reusable workflow for testing Diesel migrations:
- Sets up PostgreSQL service
- Installs Diesel CLI
- Tests migrations: setup, run, rollback, and re-run
- Can be called from any workflow that needs database migration testing

### Build and Release Workflow (`build-and-release.yml`)

Unified workflow that handles both releases and nightly builds:

#### Release Mode
Triggers on version tag pushes (e.g., `v1.0.0`):
- Automatically detects release build from tag push
- Builds multi-architecture Docker images (`linux/amd64` and `linux/arm64`)
- Pushes to GitHub Container Registry with semantic versioning tags
- Creates GitHub release with automatic release notes

#### Nightly Mode
Runs daily at 2 AM UTC (or via manual trigger):
- Checks for commits since the last nightly release
- Only builds if there are new changes
- Extracts version from `Cargo.toml`
- Creates versioned tag: `{version}-nightly-YYYYMMDD` (e.g., `0.1.0-nightly-20231123`)
- Publishes Docker images with both versioned tag and `nightly` tag
- Supports multi-architecture builds (linux/amd64 and linux/arm64)

To use nightly releases:
```bash
docker pull ghcr.io/x-rays5/gmaps-review-notif:nightly
```

Or pull a specific nightly version:
```bash
docker pull ghcr.io/x-rays5/gmaps-review-notif:0.1.0-nightly-20231123
```

**Note:** The workflow uses conditional job execution to run either release or nightly build logic based on the trigger type, eliminating code duplication between the two build types.

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

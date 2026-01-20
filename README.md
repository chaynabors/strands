# Strands Personal Monorepo

Personal monorepo for Strands projects. To later be moved into the strands-labs org.

## Repository Structure

### strands-metrics/

Rust CLI tool for syncing GitHub organization data to SQLite. Collects issues, pull requests, commits, stars, and CI workflow runs. Computes daily aggregated metrics per repository.

### strands-grafana/

Grafana configuration with SQLite datasource for visualizing GitHub metrics. Includes health dashboard (DORA-style metrics) and triage dashboard (operational views).

### strands-rs/

Experimental Strands SDK implementation in Rust.

### filament-sys/

Rust FFI bindings for the Filament specification. Filament is a specification for autonomous AI agents with deterministic execution, WebAssembly sandboxing, and resource limits.

### metrics.db

SQLite database tracked via Git LFS. Contains synced GitHub metrics and pre-computed daily aggregates.

## Prerequisites for Grafana

### Git LFS

Required to clone the metrics.db file. Install and initialize:

```bash
# macOS
brew install git-lfs

# Ubuntu/Debian
sudo apt-get install git-lfs

# Initialize
git lfs install
git lfs pull
```

Verify with `git lfs ls-files` - should show metrics.db.

### Other

- Docker and Docker Compose (Or `podman` which I prefer)

## Quick Start

```bash
# Clone and setup Git LFS
git clone <repo-url>
cd strands-personal-mono
git lfs install
git lfs pull

# OPTIONAL: Sync metrics (requires GitHub token and Rust toolchain)
cd strands-metrics
export GITHUB_TOKEN="token"
cargo run --release -- sync

# Launch Grafana
cd ../strands-grafana
docker-compose up # or podman compose up
# Navigate to http://localhost:3000
```

## License

Licensed under Apache-2.0 OR MIT.

# Strands Metrics

Monorepo for Strands GitHub metrics collection and Grafana dashboards.

## Repository Structure

### strands-metrics/

Rust CLI tool for syncing GitHub organization data and package download stats to SQLite. Collects issues, pull requests, commits, stars, CI workflow runs, and download metrics from PyPI/npm. Computes daily aggregated metrics per repository.

### strands-grafana/

Grafana configuration with SQLite datasource for visualizing GitHub metrics. Includes:

**General Dashboards:**
- **Health Dashboard** - DORA-style metrics with dynamic goal lines for tracking targets
- **Executive Dashboard** - High-level org metrics, cycle time trends, stale PR tracking

**Operations Dashboards:**
- **Team Dashboard** - Team performance scorecard, review balance, category leaders
- **Triage Dashboard** - Operational views for daily/weekly triage (open PRs, issues, stale items)

**SDK Dashboards:**
- **Python SDK** - Metrics specific to sdk-python repository
- **TypeScript SDK** - Metrics specific to sdk-typescript repository
- **Evals** - Metrics specific to evals repository

**Configuration files:**
- `goals.yaml` - Configurable goal thresholds for dashboard alerts and goal lines
- `packages.yaml` - Package-to-repo mappings for download tracking

### strands-rs/

Experimental Strands SDK implementation in Rust.

### filament-sys/

Rust FFI bindings for the Filament specification. Filament is a specification for autonomous AI agents with deterministic execution, WebAssembly sandboxing, and resource limits.

### metrics.db

SQLite database tracked via Git LFS. Contains synced GitHub metrics, package downloads, and pre-computed daily aggregates.

## Prerequisites

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

### Other Requirements

- Docker and Docker Compose (or Podman)
- Rust toolchain (for building strands-metrics)
- GitHub Personal Access Token (for syncing data)

## Quick Start

```bash
# Clone and setup Git LFS
git clone <repo-url>
cd strands
git lfs install
git lfs pull

# Launch Grafana
cd strands-grafana
docker-compose up  # or: podman compose up

# Navigate to http://localhost:3000
```

## CLI Commands

The `strands-metrics` CLI provides commands for managing metrics data. Run from the repository root:

```bash
cd strands-metrics
cargo run --release -- <command>
```

### GitHub Data Sync

```bash
# Sync GitHub data (requires GITHUB_TOKEN)
export GITHUB_TOKEN="your_token"
cargo run --release -- sync

# Garbage collection - marks deleted items
cargo run --release -- sweep
```

### Package Download Stats

```bash
# Sync recent download stats (default: 30 days)
cargo run --release -- sync-downloads

# Sync with custom day range
cargo run --release -- sync-downloads --days 7

# Backfill historical data (PyPI: 180 days, npm: 365 days)
cargo run --release -- backfill-downloads
```

Download stats are fetched from:
- **PyPI** (pypistats.org) - Updated daily ~01:00 UTC, 180 days retention
- **npm** (api.npmjs.org) - Updated daily, 365+ days retention

Note: Data has ~24h delay due to upstream API update schedules.

### Goals Management

```bash
# Load goals from config into database
cargo run --release -- load-goals

# Load from custom path
cargo run --release -- load-goals path/to/goals.yaml

# List all configured goals
cargo run --release -- list-goals
```

Goals control both threshold colors (green/yellow/red) and goal lines on dashboard charts. See [Adding New Goals](#adding-new-goals) for details.

### Team Management

The Team Dashboard uses a `team_members` table to track specific team members for performance metrics.

```bash
# Load team from config file (default: strands-grafana/team.yaml)
cargo run --release -- load-team

# Load from custom config file
cargo run --release -- load-team path/to/team.yaml

# Or load directly via command line (overrides config file)
cargo run --release -- load-team --members=alice,bob,charlie
```

**Note:** This is different from how other dashboards determine internal vs external contributors:
- **Team Dashboard** - Uses `team_members` table (manual, specific people you want to track)
- **Other Dashboards** - Uses GitHub's `author_association` field (automatic, all org members/collaborators)

### Raw SQL Queries

```bash
# Run arbitrary SQL against the database
cargo run --release -- query "SELECT * FROM daily_metrics LIMIT 5"

# Check goal thresholds
cargo run --release -- query "SELECT * FROM goal_thresholds"

# Check team members
cargo run --release -- query "SELECT * FROM team_members"
```

## Configuration Files

### strands-grafana/goals.yaml

Defines goal thresholds for metrics. Goals appear as horizontal lines on time series charts and control stat panel colors.

#### Configuration Reference

Each goal requires:
- `value` - The target threshold value
- `label` - Display label for the goal line (shown in Grafana legend)
- `direction` - How to interpret values relative to the goal:
  - `lower_is_better` - Green below goal, red above (e.g., merge time, failure rate)
  - `higher_is_better` - Green above goal, red below (e.g., community %, retention)
- `warning_ratio` (optional) - Multiplier for warning threshold
  - Default: 0.75 for `lower_is_better`, 0.70 for `higher_is_better`
  - Must be between 0 and 1 (exclusive)

#### Example Configuration

```yaml
goals:
  # Time metrics (lower is better)
  avg_merge_time_hours:
    value: 24
    label: "Goal (24h)"
    direction: lower_is_better
    # warning at 18h (default 0.75 ratio)

  # Community metrics (higher is better)
  community_pr_percent_min:
    value: 20
    label: "Goal (20%)"
    direction: higher_is_better
    warning_ratio: 0.70  # warning at 14%
```

### Adding New Goals

To add a new goal that appears on dashboards:

#### Step 1: Add the goal to goals.yaml

```yaml
goals:
  # ... existing goals ...

  my_new_metric:
    value: 50
    label: "Goal (50)"
    direction: lower_is_better  # or higher_is_better
    warning_ratio: 0.80         # optional, defaults based on direction
```

#### Step 2: Load the goal into the database

```bash
cd strands-metrics
cargo run --release -- load-goals
```

#### Step 3: Add the goal query to the dashboard

In the relevant dashboard JSON file, add a query to the panel's `targets` array:

**For time series panels (goal line):**
```json
{
  "queryText": "SELECT $__from / 1000 as time, label as metric, value FROM goals WHERE metric = 'my_new_metric'",
  "queryType": "time series",
  "refId": "Goal",
  "hide": false
}
```

**For stat panels (dynamic thresholds):**

1. Add a hidden threshold query:
```json
{
  "queryText": "SELECT warning_value as \"Warning\", goal_value as \"Critical\" FROM goal_thresholds WHERE metric = 'my_new_metric'",
  "queryType": "table",
  "refId": "Thresholds",
  "hide": true
}
```

2. Add the `configFromData` transformation to the panel:
```json
"transformations": [
  {
    "id": "configFromData",
    "options": {
      "configRefId": "Thresholds",
      "mappings": [
        {"fieldName": "Warning", "handlerKey": "threshold1", "reducerId": "lastNotNull"},
        {"fieldName": "Critical", "handlerKey": "threshold2", "reducerId": "lastNotNull"}
      ]
    }
  }
]
```

#### Step 4: Verify in Grafana

Refresh Grafana (or restart if needed) and check that:
- The goal line appears on the chart
- The label matches what you specified
- Threshold colors work correctly (if using stat panels)

#### Available Metrics

The following metrics are pre-configured in goals.yaml:

| Metric | Description | Direction |
|--------|-------------|-----------|
| `avg_merge_time_hours` | Average time from PR open to merge | lower_is_better |
| `cycle_time_hours` | Time from first commit to merge | lower_is_better |
| `time_to_first_review_hours` | Time until first review on PR | lower_is_better |
| `time_to_first_response_hours` | Time until first comment/review | lower_is_better |
| `ci_failure_rate_percent` | Percentage of PRs with failed CI | lower_is_better |
| `pr_acceptance_rate_min` | Percentage of PRs merged vs closed | higher_is_better |
| `community_pr_percent_min` | Percentage of merged PRs from community | higher_is_better |
| `contributor_retention_min` | Percentage of returning contributors | higher_is_better |
| `stale_prs_max` | Maximum acceptable stale PRs | lower_is_better |

### strands-grafana/team.yaml

Defines team members for the Team Dashboard. These are the specific people whose performance you want to track.

```yaml
members:
  - username: alice
  - username: bob
    display_name: "Bob Smith"  # optional
  - username: charlie
```

**Note:** This is separate from GitHub org membership. Use this to track a specific subset of contributors (e.g., core maintainers) rather than everyone in the org.

### strands-grafana/packages.yaml

Maps GitHub repos to published packages for download tracking:

```yaml
repo_mappings:
  sdk-python:
    - package: strands-agents
      registry: pypi
  sdk-typescript:
    - package: "@strands-agents/sdk"
      registry: npm
```

## Database Schema

Key tables in `metrics.db`:

| Table | Description |
|-------|-------------|
| `pull_requests` | All PRs with full JSON data |
| `issues` | All issues with full JSON data |
| `pr_reviews` | PR review events |
| `commits` | Commit metadata |
| `daily_metrics` | Pre-computed daily aggregates per repo |
| `goals` | Goal thresholds from goals.yaml |
| `goal_thresholds` | SQL view with calculated warning values |
| `team_members` | Team members for Team Dashboard |
| `package_downloads` | Daily download counts from PyPI/npm |

## GitHub Action (Automated Updates)

The repository includes a GitHub Action that runs daily at 6 AM UTC to sync all metrics.

### Required Secret

Create a repository secret named `METRICS_PAT` containing a GitHub Personal Access Token with:
- `repo` scope (for accessing repository data)
- `read:org` scope (for organization membership)

### Workflow

The action (`.github/workflows/metrics.yaml`) runs:
1. `sync` - Incrementally fetches new issues, PRs, commits, stars, and CI runs
2. `sweep` - Garbage collection to mark deleted items
3. `sync-downloads` - Fetches package download stats from PyPI and npm
4. Commits and pushes updated `metrics.db` to `main` branch

### Manual Trigger

Run manually via GitHub Actions UI or CLI:
```bash
gh workflow run metrics.yaml
```

## Troubleshooting

### Goals not appearing on dashboards

1. Verify the goal is in the database:
   ```bash
   cargo run --release -- query "SELECT * FROM goals WHERE metric = 'my_metric'"
   ```

2. Check the goal_thresholds view:
   ```bash
   cargo run --release -- query "SELECT * FROM goal_thresholds WHERE metric = 'my_metric'"
   ```

3. Ensure `direction` is set (goals without direction don't appear in the view)

### Team Dashboard shows no data

Load team members first:
```bash
cargo run --release -- load-team --members=user1,user2,user3
```

### Invalid goals.yaml

The CLI validates:
- `direction` must be `lower_is_better` or `higher_is_better`
- `warning_ratio` must be between 0 and 1 (exclusive)

Errors will be shown when running `load-goals`.

## License

Licensed under Apache-2.0 OR MIT.

use anyhow::{bail, Result};
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    LowerIsBetter,
    HigherIsBetter,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::LowerIsBetter => f.write_str("lower_is_better"),
            Direction::HigherIsBetter => f.write_str("higher_is_better"),
        }
    }
}

impl FromStr for Direction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "lower_is_better" => Ok(Direction::LowerIsBetter),
            "higher_is_better" => Ok(Direction::HigherIsBetter),
            other => bail!("unknown direction: {other}"),
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct GoalEntry {
    value: f64,
    label: Option<String>,
    direction: Direction,
    warning_ratio: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct TeamMemberEntry {
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RepoMapping {
    pub repo: String,
    pub package: String,
    pub registry: String,
}

/// Unified config loaded from config.toml.
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub goals: HashMap<String, GoalEntry>,
    #[serde(default)]
    pub repo_mappings: Vec<RepoMapping>,
    #[serde(default)]
    pub members: Vec<TeamMemberEntry>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn packages_for_registry(&self, registry: &str) -> Vec<String> {
        self.repo_mappings
            .iter()
            .filter(|m| m.registry == registry)
            .map(|m| m.package.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Goal {
    pub metric: String,
    pub value: f64,
    pub label: Option<String>,
    pub direction: Direction,
    pub warning_ratio: Option<f64>,
}


pub fn init_goals_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goals (
            metric TEXT PRIMARY KEY,
            value REAL NOT NULL,
            label TEXT,
            direction TEXT,
            warning_ratio REAL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    let _ = conn.execute("ALTER TABLE goals ADD COLUMN label TEXT", []);
    let _ = conn.execute("ALTER TABLE goals ADD COLUMN direction TEXT", []);
    let _ = conn.execute("ALTER TABLE goals ADD COLUMN warning_ratio REAL", []);

    conn.execute("DROP VIEW IF EXISTS goal_thresholds", [])?;
    conn.execute(
        "CREATE VIEW IF NOT EXISTS goal_thresholds AS
        SELECT
            metric,
            value as goal_value,
            label,
            direction,
            warning_ratio,
            CASE
                WHEN direction = 'lower_is_better' THEN
                    value * COALESCE(warning_ratio, 0.75)
                WHEN direction = 'higher_is_better' THEN
                    value * COALESCE(warning_ratio, 0.70)
                ELSE value * 0.75
            END as warning_value
        FROM goals
        WHERE direction IS NOT NULL",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS team_members (
            username TEXT PRIMARY KEY,
            display_name TEXT,
            added_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

pub fn load_goals(conn: &Connection, config: &Config) -> Result<usize> {
    let mut count = 0;
    for (metric, entry) in &config.goals {
        if let Some(ratio) = entry.warning_ratio
            && (ratio <= 0.0 || ratio >= 1.0)
        {
            bail!(
                "warning_ratio must be between 0 and 1 (exclusive), got {ratio} for metric '{metric}'"
            );
        }

        conn.execute(
            "INSERT INTO goals (metric, value, label, direction, warning_ratio, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
             ON CONFLICT(metric) DO UPDATE SET
                value = excluded.value,
                label = excluded.label,
                direction = excluded.direction,
                warning_ratio = excluded.warning_ratio,
                updated_at = datetime('now')",
            params![
                metric,
                entry.value,
                entry.label,
                entry.direction.to_string(),
                entry.warning_ratio
            ],
        )?;
        count += 1;
    }
    Ok(count)
}

pub fn list_goals(conn: &Connection) -> Result<Vec<Goal>> {
    let mut stmt = conn.prepare(
        "SELECT metric, value, label, direction, warning_ratio FROM goals ORDER BY metric",
    )?;

    let rows = stmt.query_map([], |row| {
        let direction_str: Option<String> = row.get(3)?;
        let direction = direction_str
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(Direction::LowerIsBetter);

        Ok(Goal {
            metric: row.get(0)?,
            value: row.get(1)?,
            label: row.get(2)?,
            direction,
            warning_ratio: row.get(4)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn load_team(conn: &Connection, members: &[TeamMemberEntry]) -> Result<usize> {
    let mut count = 0;
    for member in members {
        conn.execute(
            "INSERT INTO team_members (username, display_name, added_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(username) DO UPDATE SET
                display_name = excluded.display_name,
                added_at = datetime('now')",
            params![member.username, member.display_name],
        )?;
        count += 1;
    }
    Ok(count)
}

pub fn load_repo_mappings(conn: &Connection, config: &Config) -> Result<usize> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS repo_package_mapping (
            repo TEXT NOT NULL,
            package TEXT NOT NULL,
            registry TEXT NOT NULL,
            PRIMARY KEY (repo, package)
        )",
        [],
    )?;

    let mut count = 0;
    for mapping in &config.repo_mappings {
        conn.execute(
            "INSERT OR REPLACE INTO repo_package_mapping (repo, package, registry)
             VALUES (?1, ?2, ?3)",
            params![mapping.repo, mapping.package, mapping.registry],
        )?;
        count += 1;
    }
    Ok(count)
}
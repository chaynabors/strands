use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct GoalsConfig {
    goals: HashMap<String, f64>,
}

pub fn init_goals_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goals (
            metric TEXT PRIMARY KEY,
            value REAL NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;
    Ok(())
}

pub fn load_goals<P: AsRef<Path>>(conn: &Connection, yaml_path: P) -> Result<usize> {
    let content = fs::read_to_string(yaml_path)?;
    let config: GoalsConfig = serde_yaml::from_str(&content)?;

    let mut count = 0;
    for (metric, value) in config.goals {
        conn.execute(
            "INSERT INTO goals (metric, value, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(metric) DO UPDATE SET
                value = excluded.value,
                updated_at = datetime('now')",
            params![metric, value],
        )?;
        count += 1;
    }

    Ok(count)
}

pub fn get_goal(conn: &Connection, metric: &str) -> Result<Option<f64>> {
    let result = conn.query_row(
        "SELECT value FROM goals WHERE metric = ?1",
        params![metric],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn list_goals(conn: &Connection) -> Result<Vec<(String, f64)>> {
    let mut stmt = conn.prepare("SELECT metric, value FROM goals ORDER BY metric")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut goals = Vec::new();
    for row in rows {
        goals.push(row?);
    }
    Ok(goals)
}

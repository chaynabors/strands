use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub fn init_db<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS pull_requests (
            id INTEGER PRIMARY KEY,
            repo TEXT NOT NULL,
            number INTEGER NOT NULL,
            state TEXT NOT NULL,
            author TEXT NOT NULL,
            title TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            merged_at TEXT,
            closed_at TEXT,
            data JSON NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS issues (
            id INTEGER PRIMARY KEY,
            repo TEXT NOT NULL,
            number INTEGER NOT NULL,
            state TEXT NOT NULL,
            author TEXT NOT NULL,
            title TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            closed_at TEXT,
            data JSON NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS reviews (
            id INTEGER PRIMARY KEY,
            repo TEXT NOT NULL,
            pr_number INTEGER NOT NULL,
            state TEXT NOT NULL,
            author TEXT NOT NULL,
            submitted_at TEXT NOT NULL,
            data JSON NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY,
            repo TEXT NOT NULL,
            issue_number INTEGER NOT NULL,
            author TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            data JSON NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS daily_metrics (
            date TEXT NOT NULL,
            repo TEXT NOT NULL,
            prs_opened INTEGER DEFAULT 0,
            prs_merged INTEGER DEFAULT 0,
            issues_opened INTEGER DEFAULT 0,
            issues_closed INTEGER DEFAULT 0,
            stars INTEGER DEFAULT 0,
            open_issues_count INTEGER DEFAULT 0,
            
            time_to_first_response REAL DEFAULT 0,
            time_to_merge_internal REAL DEFAULT 0,
            time_to_merge_external REAL DEFAULT 0,
            time_to_close_issue_external REAL DEFAULT 0,
            
            PRIMARY KEY (date, repo)
        )",
        [],
    )?;

    // Minor perf bump
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pr_repo_updated ON pull_requests(repo, updated_at)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_issues_repo_updated ON issues(repo, updated_at)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_comments_repo_issue ON comments(repo, issue_number)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reviews_repo_pr ON reviews(repo, pr_number)",
        [],
    )?;

    Ok(conn)
}

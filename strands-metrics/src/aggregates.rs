use anyhow::Result;
use chrono::{Duration, Utc};
use rusqlite::{params, Connection};

pub fn compute_daily_metrics(conn: &Connection, days_back: i64) -> Result<()> {
    for i in (0..=days_back).rev() {
        let date = Utc::now() - Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();

        conn.execute(
            "INSERT OR IGNORE INTO daily_metrics (date, repo)
             SELECT DISTINCT ?1, repo FROM (
                 SELECT repo FROM pull_requests
                 UNION
                 SELECT repo FROM issues
             )",
            params![date_str],
        )?;

        conn.execute(
            "UPDATE daily_metrics 
             SET prs_opened = (
                SELECT count(*) FROM pull_requests 
                WHERE repo = daily_metrics.repo AND date(created_at) = date(daily_metrics.date)
             ),
             prs_merged = (
                SELECT count(*) FROM pull_requests 
                WHERE repo = daily_metrics.repo AND merged_at IS NOT NULL AND date(merged_at) = date(daily_metrics.date)
             ),
             issues_opened = (
                SELECT count(*) FROM issues 
                WHERE repo = daily_metrics.repo AND date(created_at) = date(daily_metrics.date)
             ),
             issues_closed = (
                SELECT count(*) FROM issues 
                WHERE repo = daily_metrics.repo AND closed_at IS NOT NULL AND date(closed_at) = date(daily_metrics.date)
             )
             WHERE date = ?1",
            params![date_str],
        )?;

        conn.execute(
            "UPDATE daily_metrics
             SET time_to_first_response = (
                SELECT AVG((julianday(first_response) - julianday(created_at)) * 24)
                FROM (
                    SELECT 
                        items.created_at,
                        (
                            SELECT MIN(activity_at)
                            FROM (
                                SELECT created_at as activity_at 
                                FROM comments c 
                                WHERE c.repo = items.repo AND c.issue_number = items.number AND c.author != items.author
                                UNION ALL
                                SELECT submitted_at as activity_at
                                FROM reviews r
                                WHERE r.repo = items.repo AND r.pr_number = items.number AND r.author != items.author
                            ) sub
                            WHERE sub.activity_at > items.created_at
                        ) as first_response
                    FROM (
                        SELECT repo, number, created_at, author FROM issues
                        UNION ALL
                        SELECT repo, number, created_at, author FROM pull_requests
                    ) items
                    WHERE items.repo = daily_metrics.repo
                      AND date(items.created_at) = date(daily_metrics.date)
                )
                WHERE first_response IS NOT NULL
             )
             WHERE date = ?1",
            params![date_str],
        )?;

        conn.execute(
            "UPDATE daily_metrics
             SET time_to_merge_internal = (
                SELECT AVG((julianday(merged_at) - julianday(created_at)) * 24)
                FROM pull_requests
                WHERE repo = daily_metrics.repo
                  AND merged_at IS NOT NULL
                  AND date(merged_at) = date(daily_metrics.date)
                  AND json_extract(data, '$.author_association') IN ('OWNER', 'MEMBER')
             )
             WHERE date = ?1",
            params![date_str],
        )?;

        conn.execute(
            "UPDATE daily_metrics
             SET time_to_merge_external = (
                SELECT AVG((julianday(merged_at) - julianday(created_at)) * 24)
                FROM pull_requests
                WHERE repo = daily_metrics.repo
                  AND merged_at IS NOT NULL
                  AND date(merged_at) = date(daily_metrics.date)
                  AND json_extract(data, '$.author_association') NOT IN ('OWNER', 'MEMBER')
             )
             WHERE date = ?1",
            params![date_str],
        )?;

        conn.execute(
            "UPDATE daily_metrics
             SET time_to_close_issue_external = (
                SELECT AVG((julianday(closed_at) - julianday(created_at)) * 24)
                FROM issues
                WHERE repo = daily_metrics.repo
                  AND closed_at IS NOT NULL
                  AND date(closed_at) = date(daily_metrics.date)
                  AND json_extract(data, '$.author_association') NOT IN ('OWNER', 'MEMBER')
             )
             WHERE date = ?1",
            params![date_str],
        )?;
    }

    Ok(())
}

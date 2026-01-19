use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use octocrab::{models, Octocrab};
use rusqlite::{params, Connection};
use serde_json::Value;

pub struct GitHubClient<'a> {
    gh: Octocrab,
    db: &'a mut Connection,
}

impl<'a> GitHubClient<'a> {
    pub fn new(gh: Octocrab, db: &'a mut Connection) -> Self {
        Self { gh, db }
    }

    async fn check_limits(&self) -> Result<()> {
        let rate = self.gh.ratelimit().get().await?;
        let core = rate.resources.core;

        if core.remaining < 10 {
            let reset = core.reset;
            let now = Utc::now().timestamp() as u64;
            let wait_secs = reset.saturating_sub(now);

            tracing::warn!(
                "Rate limit low ({}). Sleeping {}s...",
                core.remaining,
                wait_secs
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(wait_secs + 5)).await;
        }
        Ok(())
    }

    pub async fn sync_org(&mut self, org: &str, days_back: i64) -> Result<()> {
        self.check_limits().await?;
        let repos = self.fetch_repos(org).await?;
        for repo in repos {
            tracing::info!("Syncing Repo: {}", repo.name);
            self.sync_repo(org, &repo.name, days_back).await?;
        }
        Ok(())
    }

    async fn fetch_repos(&self, org: &str) -> Result<Vec<models::Repository>> {
        let mut repos = Vec::new();
        let mut page = self.gh.orgs(org).list_repos().per_page(100).send().await?;
        repos.extend(page.items);
        while let Some(next) = page.next {
            self.check_limits().await?;
            page = self.gh.get_page(&Some(next)).await?.unwrap();
            repos.extend(page.items);
        }
        repos.retain(|r| !r.archived.unwrap_or(false));
        Ok(repos)
    }

    async fn sync_repo(&mut self, org: &str, repo: &str, default_days: i64) -> Result<()> {
        let last_sync_key = format!("last_sync_{}_{}", org, repo);
        let target_start = Utc::now() - Duration::days(default_days);

        let since: DateTime<Utc> = self
            .db
            .query_row(
                "SELECT value FROM app_state WHERE key = ?1",
                params![last_sync_key],
                |row| {
                    let s: String = row.get(0)?;
                    Ok(DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()))
                },
            )
            .map(|saved| {
                if saved < target_start {
                    saved
                } else {
                    target_start
                }
            })
            .unwrap_or(target_start);

        let repo_model = self.gh.repos(org, repo).get().await?;
        self.record_snapshot(repo, &repo_model)?;

        self.sync_pulls(org, repo, since).await?;
        self.sync_issues(org, repo, since).await?;
        self.sync_comments(org, repo, since).await?;

        let now_str = Utc::now().to_rfc3339();
        self.db.execute(
            "INSERT OR REPLACE INTO app_state (key, value) VALUES (?1, ?2)",
            params![last_sync_key, now_str],
        )?;

        Ok(())
    }

    fn record_snapshot(&self, repo_name: &str, repo: &models::Repository) -> Result<()> {
        let date_str = Utc::now().format("%Y-%m-%d").to_string();
        self.db.execute(
            "INSERT OR IGNORE INTO daily_metrics (date, repo) VALUES (?1, ?2)",
            params![date_str, repo_name],
        )?;
        self.db.execute(
            "UPDATE daily_metrics SET stars = ?1, open_issues_count = ?2 WHERE date = ?3 AND repo = ?4",
            params![repo.stargazers_count.unwrap_or(0), repo.open_issues_count.unwrap_or(0), date_str, repo_name],
        )?;
        Ok(())
    }

    async fn sync_pulls(&self, org: &str, repo: &str, since: DateTime<Utc>) -> Result<()> {
        self.check_limits().await?;
        let mut page = self
            .gh
            .pulls(org, repo)
            .list()
            .state(octocrab::params::State::All)
            .sort(octocrab::params::pulls::Sort::Updated)
            .direction(octocrab::params::Direction::Descending)
            .per_page(100)
            .send()
            .await?;

        let mut keep_fetching = true;
        loop {
            let next_page = page.next;
            for pr in page.items {
                if let Some(updated) = pr.updated_at {
                    if updated < since {
                        keep_fetching = false;
                        break;
                    }
                }
                let json = serde_json::to_string(&pr)?;
                let pr_id: i64 = pr.id.0.try_into()?;
                let pr_number: i64 = pr.number.try_into()?;
                let state_str = match pr.state {
                    Some(models::IssueState::Open) => "open",
                    Some(models::IssueState::Closed) => "closed",
                    _ => "unknown",
                };
                let created_at_str = pr
                    .created_at
                    .map(|d| d.to_rfc3339())
                    .unwrap_or_else(|| Utc::now().to_rfc3339());
                let updated_at_str = pr
                    .updated_at
                    .map(|d| d.to_rfc3339())
                    .unwrap_or_else(|| created_at_str.clone());

                self.db.execute(
                    "INSERT OR REPLACE INTO pull_requests 
                    (id, repo, number, state, author, title, created_at, updated_at, merged_at, closed_at, data) 
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        pr_id, repo, pr_number, state_str,
                        pr.user.as_ref().map(|u| u.login.clone()).unwrap_or_default(),
                        pr.title.unwrap_or_default(),
                        created_at_str, updated_at_str,
                        pr.merged_at.map(|t| t.to_rfc3339()),
                        pr.closed_at.map(|t| t.to_rfc3339()),
                        json
                    ],
                )?;

                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                self.sync_reviews(org, repo, pr.number).await?;
            }
            if !keep_fetching {
                break;
            }
            if let Some(next) = next_page {
                self.check_limits().await?;
                page = self.gh.get_page(&Some(next)).await?.unwrap();
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn sync_issues(&self, org: &str, repo: &str, since: DateTime<Utc>) -> Result<()> {
        self.check_limits().await?;
        let route = format!("/repos/{}/{}/issues", org, repo);
        let mut page: octocrab::Page<Value> = self
            .gh
            .get(
                &route,
                Some(&serde_json::json!({
                    "state": "all",
                    "sort": "updated",
                    "direction": "desc",
                    "since": since.to_rfc3339(),
                    "per_page": 100
                })),
            )
            .await?;

        let mut keep_fetching = true;
        loop {
            let next_page = page.next.clone();
            for issue in page.items {
                let updated_at_str = issue
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let updated_at = DateTime::parse_from_rfc3339(updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                if updated_at < since {
                    keep_fetching = false;
                    break;
                }

                if issue.get("pull_request").is_some() {
                    continue;
                }

                let json = serde_json::to_string(&issue)?;
                let issue_id = issue.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                let issue_number = issue.get("number").and_then(|v| v.as_i64()).unwrap_or(0);
                let state_str = issue
                    .get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let author = issue
                    .get("user")
                    .and_then(|u| u.get("login"))
                    .and_then(|l| l.as_str())
                    .unwrap_or("unknown");
                let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let created_at_str = issue
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let closed_at_str = issue.get("closed_at").and_then(|v| v.as_str());

                self.db.execute(
                    "INSERT OR REPLACE INTO issues 
                    (id, repo, number, state, author, title, created_at, updated_at, closed_at, data) 
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        issue_id, repo, issue_number, state_str, author,
                        title, created_at_str, updated_at_str, closed_at_str,
                        json
                    ],
                )?;
            }
            if !keep_fetching {
                break;
            }
            if let Some(next) = next_page {
                self.check_limits().await?;
                page = self.gh.get_page(&Some(next)).await?.unwrap();
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn sync_reviews(&self, org: &str, repo: &str, pr_number: u64) -> Result<()> {
        let mut page = self
            .gh
            .pulls(org, repo)
            .list_reviews(pr_number)
            .per_page(100)
            .send()
            .await?;
        loop {
            let next_page = page.next;
            for review in page.items {
                let json = serde_json::to_string(&review)?;
                let review_id: i64 = review.id.0.try_into()?;
                let pr_number_i64: i64 = pr_number.try_into()?;
                let state_str = review
                    .state
                    .map(|s| format!("{:?}", s).to_uppercase())
                    .unwrap_or_else(|| "UNKNOWN".to_string());
                let author = review
                    .user
                    .as_ref()
                    .map(|u| u.login.clone())
                    .unwrap_or_default();

                self.db.execute(
                    "INSERT OR REPLACE INTO reviews (id, repo, pr_number, state, author, submitted_at, data)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![review_id, repo, pr_number_i64, state_str, author, review.submitted_at.map(|t| t.to_rfc3339()).unwrap_or_default(), json],
                )?;
            }
            if let Some(next) = next_page {
                self.check_limits().await?;
                page = self.gh.get_page(&Some(next)).await?.unwrap();
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn sync_comments(&self, org: &str, repo: &str, since: DateTime<Utc>) -> Result<()> {
        self.check_limits().await?;
        let route = format!("/repos/{}/{}/issues/comments", org, repo);
        let mut page: octocrab::Page<Value> = self.gh
            .get(&route, Some(&serde_json::json!({
                "sort": "updated", "direction": "desc", "since": since.to_rfc3339(), "per_page": 100
            })))
            .await?;

        let mut keep_fetching = true;
        loop {
            let next_page = page.next.clone();
            for comment in page.items {
                let updated_at_str = comment
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let updated_at = DateTime::parse_from_rfc3339(updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                if updated_at < since {
                    keep_fetching = false;
                    break;
                }

                let issue_url = comment
                    .get("issue_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let issue_number: i64 = issue_url
                    .split('/')
                    .next_back()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let id = comment.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                let created_at_str = comment
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let author = comment
                    .get("user")
                    .and_then(|u| u.get("login"))
                    .and_then(|l| l.as_str())
                    .unwrap_or("unknown");
                let json = serde_json::to_string(&comment)?;

                self.db.execute(
                    "INSERT OR REPLACE INTO comments (id, repo, issue_number, author, created_at, updated_at, data)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![id, repo, issue_number, author, created_at_str, updated_at_str, json],
                )?;
            }
            if !keep_fetching {
                break;
            }
            if let Some(next) = next_page {
                self.check_limits().await?;
                page = self.gh.get_page(&Some(next)).await?.unwrap();
            } else {
                break;
            }
        }
        Ok(())
    }
}

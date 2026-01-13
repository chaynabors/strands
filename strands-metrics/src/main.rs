mod aggregates;
mod client;
mod db;

use anyhow::Result;
use clap::{Parser, Subcommand};
use client::GitHubClient;
use db::init_db;
use octocrab::OctocrabBuilder;
use std::path::PathBuf;

const ORG: &str = "strands-agents";

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long, short, default_value = "metrics.db")]
    db_path: PathBuf,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sync {
        #[clap(long, default_value = "7")]
        days_back: i64,
    },
    Query {
        sql: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args = Cli::parse();
    let mut conn = init_db(&args.db_path)?;

    match args.command {
        Commands::Sync { days_back } => {
            let gh_token = std::env::var("GITHUB_TOKEN")?;
            let octocrab = OctocrabBuilder::new().personal_token(gh_token).build()?;
            let mut client = GitHubClient::new(octocrab, &mut conn);

            tracing::info!("Syncing data from GitHub");
            client.sync_org(ORG, days_back).await?;
            tracing::info!("Calculating aggregates");
            aggregates::compute_daily_metrics(&conn, days_back)?;
            tracing::info!("Sync complete");
        }
        Commands::Query { sql } => {
            let mut stmt = conn.prepare(&sql)?;
            let column_count = stmt.column_count();

            let names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
            println!("{}", names.join(" | "));
            println!("{}", "-".repeat(names.len() * 15));

            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let mut row_values = Vec::new();
                for i in 0..column_count {
                    let val = row.get_ref(i)?;
                    let text = match val {
                        rusqlite::types::ValueRef::Null => "NULL".to_string(),
                        rusqlite::types::ValueRef::Integer(i) => i.to_string(),
                        rusqlite::types::ValueRef::Real(f) => f.to_string(),
                        rusqlite::types::ValueRef::Text(t) => {
                            String::from_utf8_lossy(t).to_string()
                        }
                        rusqlite::types::ValueRef::Blob(_) => "<BLOB>".to_string(),
                    };
                    row_values.push(text);
                }
                println!("{}", row_values.join(" | "));
            }
        }
    }

    Ok(())
}

use anyhow::{Error, Result};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use crb::agent::{Agent, AgentSession, DoAsync, DoSync, Next, Standalone};
use csv::Writer;
use db_dump::{crates::Row, Loader};
use futures::StreamExt;
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

const URL: &str = "https://static.crates.io/db-dump.tar.gz";
const PATH: &str = "db-dump.tar.gz";

pub struct CratesLoader {
    path: PathBuf,
}

impl CratesLoader {
    pub fn new() -> Self {
        Self { path: PATH.into() }
    }
}

impl Standalone for CratesLoader {}

impl Agent for CratesLoader {
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(DownloadDump)
    }
}

struct DownloadDump;

#[async_trait]
impl DoAsync<DownloadDump> for CratesLoader {
    async fn once(&mut self, _: &mut DownloadDump) -> Result<Next<Self>> {
        if !self.path.exists() {
            println!("Downloading a crates index...");
            let mut stream = reqwest::get(URL).await?.error_for_status()?.bytes_stream();
            let mut dump_file = File::create(&self.path).await?;
            while let Some(chunk) = stream.next().await {
                dump_file.write_all(&chunk?).await?;
            }
        }
        Ok(Next::do_sync(ExtractLatest))
    }

    async fn fallback(&mut self, err: Error) -> Next<Self> {
        fs::remove_file(&self.path).await.ok();
        Next::fail(err)
    }
}

struct ExtractLatest;

impl DoSync<ExtractLatest> for CratesLoader {
    fn once(&mut self, _: &mut ExtractLatest) -> Result<Next<Self>> {
        println!("Fetching data...");
        let week_ago = Utc::now().date_naive() - Duration::days(7);
        let mut latest = Vec::new();
        Loader::new()
            .crates(|row| {
                let created_at = row.created_at.naive_utc().date();
                if created_at >= week_ago {
                    latest.push(row.into());
                }
            })
            .load(&self.path)?;
        Ok(Next::do_sync(PrintCsv { latest }))
    }
}

#[derive(Serialize, Clone, Debug)]
struct CrateInfo {
    name: String,
    info: String,
}

impl From<Row> for CrateInfo {
    fn from(row: Row) -> Self {
        Self {
            name: row.name,
            info: row.description,
        }
    }
}

struct PrintCsv {
    latest: Vec<CrateInfo>,
}

impl DoSync<PrintCsv> for CratesLoader {
    fn once(&mut self, state: &mut PrintCsv) -> Result<Next<Self>> {
        println!("Printing the output...");
        let mut wtr = Writer::from_writer(std::io::stdout());
        for crate_into in &state.latest {
            wtr.serialize(crate_into)?;
        }
        wtr.flush()?;
        Ok(Next::done())
    }
}

use crate::model::TarcieEvent;
use crate::util::paths::{queue_dir, sent_dir};
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct JsonlQueue {
    lock: Mutex<()>,
    queue_path: PathBuf,
}

impl JsonlQueue {
    pub fn new() -> Result<Self> {
        let qdir = queue_dir()?;
        fs::create_dir_all(&qdir)?;
        fs::create_dir_all(sent_dir()?)?;
        let queue_path = qdir.join("queue.jsonl");
        Ok(Self { lock: Mutex::new(()), queue_path })
    }

    pub fn append(&self, event: &TarcieEvent, queue_max_events: usize) -> Result<()> {
        let _g = self.lock.lock().unwrap();

        if self.line_count()? >= queue_max_events {
            self.rotate_locked("queue.cap")?;
        }

        let json = serde_json::to_string(event).context("serialize event")?;
        let _: Value = serde_json::from_str(&json).context("sanity parse json")?;

        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.queue_path)
            .context("open queue.jsonl for append")?;

        f.write_all(json.as_bytes()).context("write json")?;
        f.write_all(b"\n").context("write newline")?;
        f.flush().context("flush queue file")?;

        // Proper fsync for durability
        f.sync_all().context("fsync queue file")?;

        Ok(())
    }

    pub fn read_all_tolerant(&self) -> Result<Vec<TarcieEvent>> {
        let _g = self.lock.lock().unwrap();

        if !self.queue_path.exists() {
            return Ok(vec![]);
        }

        let f = File::open(&self.queue_path).context("open queue.jsonl for read")?;
        let reader = BufReader::new(f);

        let mut out = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("tarcie: queue read error at line {}: {}", idx + 1, e);
                    continue;
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<TarcieEvent>(&line) {
                Ok(ev) => out.push(ev),
                Err(e) => {
                    eprintln!("tarcie: malformed json at line {}: {}", idx + 1, e);
                    continue;
                }
            }
        }

        Ok(out)
    }

    pub fn rotate_on_success(&self) -> Result<()> {
        let _g = self.lock.lock().unwrap();
        self.rotate_locked("queue.sent")
    }

    fn rotate_locked(&self, prefix: &str) -> Result<()> {
        if !self.queue_path.exists() {
            return Ok(());
        }
        let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let sent = sent_dir()?.join(format!("{}.{}.jsonl", prefix, ts));
        fs::rename(&self.queue_path, &sent).context("rotate queue file")?;
        Ok(())
    }

    fn line_count(&self) -> Result<usize> {
        if !self.queue_path.exists() {
            return Ok(0);
        }
        let f = File::open(&self.queue_path)?;
        let reader = BufReader::new(f);
        Ok(reader.lines().count())
    }
}

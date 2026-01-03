use crate::model::TarcieEvent;
use crate::queue::jsonl::JsonlQueue;
use crate::sink::client::SinkClient;
use crate::sink::config::SinkConfig;
use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Serialize)]
struct IngestPayload<'a> {
    source: &'static str,
    events: &'a [TarcieEvent],
}

pub struct Flusher {
    queue: Arc<JsonlQueue>,
    sink: SinkClient,
    cfg: SinkConfig,
    lock: Mutex<()>,
}

pub enum FlushResult {
    Empty,
    Success { count: usize },
    Deferred { reason: String },
}

impl Flusher {
    pub fn new(queue: Arc<JsonlQueue>, sink: SinkClient, cfg: SinkConfig) -> Self {
        Self { queue, sink, cfg, lock: Mutex::new(()) }
    }

    pub async fn flush_with_retry(&self) -> Result<FlushResult> {
        let _g = self.lock.lock().await;

        let events = self.queue.read_all_tolerant()?;
        if events.is_empty() {
            return Ok(FlushResult::Empty);
        }

        let batch_max = self.cfg.batch_max;
        for chunk in events.chunks(batch_max) {
            let payload = IngestPayload { source: "tarcie", events: chunk };

            let mut retries = 0u32;
            loop {
                match self.sink.post_json(&payload).await {
                    Ok(_) => break,
                    Err(e) if retries < 3 => {
                        retries += 1;
                        let backoff = 2u64.pow(retries);
                        sleep(Duration::from_secs(backoff)).await;
                        continue;
                    }
                    Err(e) => {
                        return Ok(FlushResult::Deferred { reason: e.to_string() });
                    }
                }
            }
        }

        self.queue.rotate_on_success()?;
        Ok(FlushResult::Success { count: events.len() })
    }

    pub fn cfg(&self) -> &SinkConfig {
        &self.cfg
    }

    pub fn queue(&self) -> &JsonlQueue {
        &self.queue
    }
}

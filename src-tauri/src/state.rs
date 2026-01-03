use crate::flusher::Flusher;
use crate::queue::jsonl::JsonlQueue;
use crate::sink::config::SinkConfig;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

pub struct AppState {
    pub cfg: SinkConfig,
    pub queue: Arc<JsonlQueue>,
    pub flusher: Arc<Flusher>,
    pub device_id: Uuid,
    pub mono_start: Instant,
}

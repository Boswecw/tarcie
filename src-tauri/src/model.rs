use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Note,
    Marker { reason: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarcieEvent {
    pub id: Uuid,
    pub device_id: Uuid,

    pub timestamp_utc: DateTime<Utc>,
    pub timestamp_mono_ms: u64,

    pub event_type: EventType,

    pub content: String,      // constrained to MAX_CONTENT_BYTES
    pub app_context: String,  // constrained to MAX_CONTEXT_CHARS
    pub source_version: String,
}

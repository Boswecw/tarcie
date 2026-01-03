use crate::constraints::*;
use crate::model::{EventType, TarcieEvent};
use crate::state::AppState;
use regex::Regex;
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use uuid::Uuid;

fn clamp_bytes(mut s: String, max: usize) -> String {
    if s.as_bytes().len() <= max {
        return s.trim().to_string();
    }
    while s.as_bytes().len() > max {
        s.pop();
    }
    let mut out = s.trim().to_string();
    out.push_str(" […truncated]");
    out
}

fn extract_tag(content: &str) -> (String, String) {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"#([a-zA-Z0-9_-]{1,32})").unwrap());

    if let Some(m) = re.find(content) {
        let tag = &content[m.start() + 1..m.end()];
        let cleaned = content.replacen(m.as_str(), "", 1).trim().to_string();
        (tag.to_string(), cleaned)
    } else {
        (DEFAULT_CONTEXT.to_string(), content.trim().to_string())
    }
}

fn now_mono_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

fn build_event(
    device_id: Uuid,
    mono_start: Instant,
    event_type: EventType,
    content: String,
    app_context: String,
) -> TarcieEvent {
    TarcieEvent {
        id: Uuid::new_v4(),
        device_id,
        timestamp_utc: chrono::Utc::now(),
        timestamp_mono_ms: now_mono_ms(mono_start),
        event_type,
        content,
        app_context,
        source_version: SOURCE_VERSION.to_string(),
    }
}

#[tauri::command]
pub async fn capture_note(content: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mono_start = state.mono_start;
    let device_id = state.device_id;

    let content = clamp_bytes(content, MAX_CONTENT_BYTES);
    let (tag, cleaned) = extract_tag(&content);

    let ctx = clamp_bytes(tag, MAX_CONTEXT_CHARS);
    let cleaned = clamp_bytes(cleaned, MAX_CONTENT_BYTES);

    let ev = build_event(device_id, mono_start, EventType::Note, cleaned, ctx);

    state
        .queue
        .append(&ev, state.cfg.queue_max_events)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn capture_marker(reason: Option<String>, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mono_start = state.mono_start;
    let device_id = state.device_id;

    let reason = reason.map(|r| clamp_bytes(r, MAX_CONTENT_BYTES));
    let ev = build_event(
        device_id,
        mono_start,
        EventType::Marker { reason },
        String::new(),
        DEFAULT_CONTEXT.to_string(),
    );

    state
        .queue
        .append(&ev, state.cfg.queue_max_events)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn flush_now(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    match state.flusher.flush_with_retry().await {
        Ok(crate::flusher::FlushResult::Empty) => Ok("empty".into()),
        Ok(crate::flusher::FlushResult::Success { count }) => Ok(format!("ok:{}", count)),
        Ok(crate::flusher::FlushResult::Deferred { reason }) => Ok(format!("deferred:{}", reason)),
        Err(e) => Err(e.to_string()),
    }
}

use crate::constraints::*;
use anyhow::{Context, Result};
use std::env;
use url::Url;

#[derive(Clone)]
pub struct SinkConfig {
    pub url: Url,
    pub auth: Option<String>,
    pub allow_remote: bool,
    pub flush_interval_secs: u64,
    pub batch_max: usize,
    pub queue_max_events: usize,
}

impl SinkConfig {
    pub fn from_env() -> Result<Self> {
        let url_raw = env::var("TARCIE_SINK_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080/ingest/tarcie".to_string());

        let url = Url::parse(&url_raw).context("parse TARCIE_SINK_URL")?;

        let allow_remote = env::var("TARCIE_ALLOW_REMOTE_SINK")
            .map(|v| v == "true" || v == "1" || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false);

        if !allow_remote && !is_localhost(&url) {
            anyhow::bail!(
                "remote sink disallowed. Set TARCIE_ALLOW_REMOTE_SINK=true to allow: {}",
                url
            );
        }

        let auth = env::var("TARCIE_SINK_AUTH").ok();

        let flush_interval_secs = env::var("TARCIE_FLUSH_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_FLUSH_INTERVAL_SECS);

        let batch_max = env::var("TARCIE_BATCH_MAX")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_BATCH_MAX);

        let queue_max_events = env::var("TARCIE_QUEUE_MAX_EVENTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_QUEUE_MAX_EVENTS);

        Ok(Self {
            url,
            auth,
            allow_remote,
            flush_interval_secs,
            batch_max: batch_max.max(1),
            queue_max_events: queue_max_events.max(100),
        })
    }
}

fn is_localhost(url: &Url) -> bool {
    match url.host_str() {
        Some("127.0.0.1") | Some("localhost") => true,
        Some(host) if host.starts_with("127.") => true,
        _ => false,
    }
}

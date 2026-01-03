use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use reqwest::Client;
use serde::Serialize;
use url::Url;

#[derive(Clone)]
pub struct SinkClient {
    client: Client,
    url: Url,
    auth: Option<String>,
}

impl SinkClient {
    pub fn new(url: Url, auth: Option<String>) -> Result<Self> {
        let client = Client::builder().build().context("build reqwest client")?;
        Ok(Self { client, url, auth })
    }

    pub async fn post_json<T: Serialize>(&self, body: &T) -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("tarcie-v1"));

        if let Some(a) = &self.auth {
            let v = if a.starts_with("Bearer ") || a.starts_with("ApiKey ") {
                a.clone()
            } else {
                format!("Bearer {}", a)
            };
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&v).context("auth header")?);
        }

        let resp = self
            .client
            .post(self.url.clone())
            .headers(headers)
            .json(body)
            .send()
            .await
            .context("POST to sink")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("sink error {}: {}", status, text);
        }

        Ok(())
    }
}

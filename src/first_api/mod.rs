pub mod types;

use std::sync::Arc;

use anyhow::{Context, Result as AnyhowResult};
use base64::Engine;
use log::warn;
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;

use crate::first_api::types::{FirstEvent, FirstEventsResponse};

const FIRST_API_BASE: &str = "https://frc-api.firstinspires.org/v3.0/";

pub struct FirstApiClient {
    client: Arc<ClientWithMiddleware>,
    auth_header: String,
}

impl FirstApiClient {
    /// `token` should be in the format `username:auth_key` — it will be base64-encoded.
    pub fn new(client: Arc<ClientWithMiddleware>, token: &str) -> Self {
        let encoded = base64::engine::general_purpose::STANDARD.encode(token.as_bytes());
        Self {
            client,
            auth_header: format!("Basic {}", encoded),
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> AnyhowResult<T> {
        let url = format!("{}{}", FIRST_API_BASE, path);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .with_context(|| format!("FIRST API request failed: {}", url))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("FIRST API error {} for {}: {}", status, url, body);
        }

        resp.json::<T>()
            .await
            .with_context(|| format!("Failed to parse FIRST API response from {}", url))
    }

    /// Fetch event details from the FIRST API.
    pub async fn get_event(&self, year: i64, code: &str) -> AnyhowResult<Option<FirstEvent>> {
        let resp: FirstEventsResponse = self
            .get(&format!("{}/events?eventCode={}", year, code))
            .await?;

        if let Some(events) = resp.events {
            Ok(events.into_iter().next())
        } else {
            Ok(None)
        }
    }

    /// Pull venue/address from FIRST API and merge into our event's fields
    /// (mirrors Python `enhance_event_data`).
    pub async fn enhance_event_data(
        &self,
        year: i64,
        first_event_code: &str,
        venue: &mut Option<String>,
        address: &mut Option<String>,
    ) -> AnyhowResult<()> {
        match self.get_event(year, first_event_code).await {
            Ok(Some(first_event)) => {
                if let Some(v) = first_event.venue {
                    *venue = Some(v);
                }
                if let Some(a) = first_event.address {
                    *address = Some(a);
                }
            }
            Ok(None) => {
                warn!(
                    "No FIRST event data for year={} code={}",
                    year, first_event_code
                );
            }
            Err(e) => {
                warn!(
                    "Failed to fetch FIRST event data for year={} code={}: {}",
                    year, first_event_code, e
                );
            }
        }
        Ok(())
    }
}

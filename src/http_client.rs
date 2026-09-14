use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use anyhow::Result as AnyhowResult;
use async_trait::async_trait;
use http::Extensions;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use log::info;
use reqwest::{Client, Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next, Result};

#[derive(Default)]
struct RequestCounter {
    total: AtomicUsize,
    hosts: Mutex<BTreeMap<String, usize>>,
}

impl RequestCounter {
    fn record(&self, request: &Request) {
        self.total.fetch_add(1, Ordering::Relaxed);
        let host = request.url().host_str().unwrap_or("<unknown>").to_owned();
        let mut hosts = self.hosts.lock().expect("HTTP metrics lock poisoned");
        *hosts.entry(host).or_default() += 1;
    }

    fn total(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }

    fn host_counts(&self) -> BTreeMap<String, usize> {
        self.hosts
            .lock()
            .expect("HTTP metrics lock poisoned")
            .clone()
    }
}

/// Counts cache lookups and requests that reached the upstream HTTP client.
#[derive(Default)]
pub struct HttpMetrics {
    lookups: RequestCounter,
    upstream_requests: RequestCounter,
}

impl HttpMetrics {
    pub fn log_summary(&self) {
        let lookups = self.lookups.total();
        let requests = self.upstream_requests.total();
        let hits = lookups.saturating_sub(requests);
        let hit_rate = if lookups == 0 {
            0.0
        } else {
            hits as f64 / lookups as f64 * 100.0
        };

        info!(
            "HTTP cache: {} hits / {} lookups ({:.1}% hit rate)",
            hits, lookups, hit_rate
        );
        info!("Actual upstream HTTP requests made: {}", requests);

        let lookup_hosts = self.lookups.host_counts();
        let request_hosts = self.upstream_requests.host_counts();
        let hosts: BTreeSet<&String> = lookup_hosts.keys().chain(request_hosts.keys()).collect();
        for host in hosts {
            let lookups = lookup_hosts.get(host).copied().unwrap_or_default();
            let requests = request_hosts.get(host).copied().unwrap_or_default();
            let hits = lookups.saturating_sub(requests);
            let hit_rate = if lookups == 0 {
                0.0
            } else {
                hits as f64 / lookups as f64 * 100.0
            };
            info!(
                "HTTP cache [{}]: {} hits / {} lookups ({:.1}%); {} upstream requests",
                host, hits, lookups, hit_rate, requests
            );
        }
    }
}

enum CounterPosition {
    BeforeCache,
    AfterCache,
}

struct CountingMiddleware {
    metrics: Arc<HttpMetrics>,
    position: CounterPosition,
}

#[async_trait]
impl Middleware for CountingMiddleware {
    async fn handle(
        &self,
        request: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        match self.position {
            CounterPosition::BeforeCache => self.metrics.lookups.record(&request),
            CounterPosition::AfterCache => self.metrics.upstream_requests.record(&request),
        }
        next.run(request, extensions).await
    }
}

/// Build a shared reqwest client with persistent filesystem-backed HTTP caching.
pub fn build_cached_client(
    cache_dir: &Path,
) -> AnyhowResult<(Arc<ClientWithMiddleware>, Arc<HttpMetrics>)> {
    let raw_client = Client::builder()
        .user_agent("frc_season_map/0.1.0")
        .build()?;

    let metrics = Arc::new(HttpMetrics::default());
    let client = ClientBuilder::new(raw_client)
        .with(CountingMiddleware {
            metrics: Arc::clone(&metrics),
            position: CounterPosition::BeforeCache,
        })
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager {
                path: cache_dir.to_path_buf(),
            },
            options: HttpCacheOptions::default(),
        }))
        .with(CountingMiddleware {
            metrics: Arc::clone(&metrics),
            position: CounterPosition::AfterCache,
        })
        .build();

    Ok((Arc::new(client), metrics))
}

#[cfg(test)]
mod tests {
    use super::RequestCounter;

    #[test]
    fn counts_requests_by_host() {
        let counter = RequestCounter::default();
        let request = reqwest::Client::new()
            .get("https://example.com/data")
            .build()
            .unwrap();

        counter.record(&request);

        assert_eq!(counter.total(), 1);
        assert_eq!(counter.host_counts().get("example.com"), Some(&1));
    }
}

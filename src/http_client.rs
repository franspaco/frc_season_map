use std::{path::Path, sync::Arc};

use anyhow::Result as AnyhowResult;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

/// Build a shared reqwest client with persistent filesystem-backed HTTP caching.
pub fn build_cached_client(cache_dir: &Path) -> AnyhowResult<Arc<ClientWithMiddleware>> {
    let raw_client = Client::builder()
        .user_agent("frc_season_map/0.1.0")
        .build()?;

    let client = ClientBuilder::new(raw_client)
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager {
                path: cache_dir.to_path_buf(),
            },
            options: HttpCacheOptions::default(),
        }))
        .build();

    Ok(Arc::new(client))
}

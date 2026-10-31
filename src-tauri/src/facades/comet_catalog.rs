//! Catalogue des comètes : éléments MPC mis en cache sur disque, rafraîchis
//! en arrière-plan tous les 7 jours, avec repli sur le dernier cache.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::MpcClient;
use crate::clients::mpc_client::parse_comet_elements;
use crate::models::{Comet, CometCatalogStatus, CometDataSource};

const REFRESH_AFTER: TimeDelta = TimeDelta::days(7);
/// Délai minimal entre deux tentatives réseau, pour ne pas insister hors ligne.
const RETRY_BACKOFF: TimeDelta = TimeDelta::minutes(10);
const CACHE_FILE: &str = "CometEls.txt";
const META_FILE: &str = "CometEls.meta.json";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheMeta {
    fetched_at: DateTime<Utc>,
}

struct CatalogState {
    comets: Arc<Vec<Comet>>,
    fetched_at: Option<DateTime<Utc>>,
    source: CometDataSource,
    last_attempt: Option<DateTime<Utc>>,
    refreshing: bool,
}

pub struct CometCatalog {
    /// `None` : catalogue figé (tests), jamais rafraîchi.
    client: Option<MpcClient>,
    cache_dir: Option<PathBuf>,
    state: Mutex<CatalogState>,
}

impl CometCatalog {
    pub fn new(client: MpcClient, cache_dir: PathBuf) -> Self {
        let (comets, fetched_at) = load_cache(&cache_dir).unwrap_or_default();
        let source = if comets.is_empty() { CometDataSource::Loading } else { CometDataSource::Cache };
        Self {
            client: Some(client),
            cache_dir: Some(cache_dir),
            state: Mutex::new(CatalogState {
                comets: Arc::new(comets),
                fetched_at,
                source,
                last_attempt: None,
                refreshing: false,
            }),
        }
    }

    /// Catalogue fixe, sans réseau ni disque.
    #[cfg(test)]
    pub fn fixed(comets: Vec<Comet>) -> Self {
        Self {
            client: None,
            cache_dir: None,
            state: Mutex::new(CatalogState {
                comets: Arc::new(comets),
                fetched_at: None,
                source: CometDataSource::Cache,
                last_attempt: None,
                refreshing: false,
            }),
        }
    }

    /// Comètes connues et état du catalogue. Ne bloque jamais : si les
    /// éléments sont périmés, un rafraîchissement part en tâche de fond.
    pub fn snapshot(self: &Arc<Self>) -> (Arc<Vec<Comet>>, CometCatalogStatus) {
        let now = Utc::now();
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let outdated = state.fetched_at.is_none_or(|at| now - at > REFRESH_AFTER);
        let may_retry = state.last_attempt.is_none_or(|at| now - at > RETRY_BACKOFF);
        if self.client.is_some() && outdated && may_retry && !state.refreshing {
            state.refreshing = true;
            state.last_attempt = Some(now);
            let this = Arc::clone(self);
            tauri::async_runtime::spawn(async move { this.refresh().await });
        }
        let status = CometCatalogStatus {
            updated_at: state.fetched_at,
            source: state.source,
            comet_count: state.comets.len(),
        };
        (Arc::clone(&state.comets), status)
    }

    async fn refresh(self: Arc<Self>) {
        let Some(client) = &self.client else { return };
        let result = client.fetch_comet_elements().await;
        let now = Utc::now();
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.refreshing = false;
        match result {
            Ok(text) => {
                if let Some(dir) = &self.cache_dir
                    && let Err(error) = save_cache(dir, &text, now)
                {
                    eprintln!("vigie: cache des comètes non écrit : {error}");
                }
                state.comets = Arc::new(parse_comet_elements(&text));
                state.fetched_at = Some(now);
                state.source = CometDataSource::Network;
            }
            Err(error) => {
                eprintln!("vigie: éléments des comètes indisponibles : {error}");
                state.source = CometDataSource::Offline;
            }
        }
    }
}

fn load_cache(dir: &Path) -> Option<(Vec<Comet>, Option<DateTime<Utc>>)> {
    let text = fs::read_to_string(dir.join(CACHE_FILE)).ok()?;
    let meta: Option<CacheMeta> =
        fs::read_to_string(dir.join(META_FILE)).ok().and_then(|json| serde_json::from_str(&json).ok());
    Some((parse_comet_elements(&text), meta.map(|m| m.fetched_at)))
}

fn save_cache(dir: &Path, text: &str, fetched_at: DateTime<Utc>) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join(CACHE_FILE), text)?;
    let meta = serde_json::to_string(&CacheMeta { fetched_at }).map_err(std::io::Error::other)?;
    fs::write(dir.join(META_FILE), meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = "0002P         2027 02 10.2274  0.338598  0.847321  187.2875  334.0199   11.3480  20261030  14.4  4.0  2P/Encke                                                 MPEC 2026-UA3\n";

    #[test]
    fn round_trips_disk_cache() {
        let dir = std::env::temp_dir().join(format!("vigie-cache-test-{}", std::process::id()));
        let at = Utc::now() - TimeDelta::days(2);
        save_cache(&dir, LINE, at).unwrap();
        let (comets, fetched_at) = load_cache(&dir).unwrap();
        assert_eq!(comets.len(), 1);
        assert_eq!(fetched_at.unwrap().timestamp(), at.timestamp());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn fixed_catalog_never_refreshes() {
        let catalog = Arc::new(CometCatalog::fixed(parse_comet_elements(LINE)));
        let (comets, status) = catalog.snapshot();
        assert_eq!(comets.len(), 1);
        assert_eq!(status.source, CometDataSource::Cache);
        assert!(!catalog.state.lock().unwrap().refreshing);
    }
}

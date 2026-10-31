use std::sync::Arc;

use tauri::Runtime;
use tauri_plugin_store::Store;

use crate::error::{AppError, AppResult};
use crate::models::Observer;

const OBSERVER_KEY: &str = "observer";

/// Persistance de l'observateur, abstraite pour pouvoir tester la facade.
pub trait ObserverStorage: Send + Sync {
    fn load(&self) -> Option<Observer>;
    fn save(&self, observer: &Observer) -> AppResult<()>;
}

/// Stockage via `tauri-plugin-store` (fichier JSON dans le dossier de l'application).
pub struct StoreObserverStorage<R: Runtime> {
    store: Arc<Store<R>>,
}

impl<R: Runtime> StoreObserverStorage<R> {
    pub fn new(store: Arc<Store<R>>) -> Self {
        Self { store }
    }
}

impl<R: Runtime> ObserverStorage for StoreObserverStorage<R> {
    fn load(&self) -> Option<Observer> {
        self.store.get(OBSERVER_KEY).and_then(|value| serde_json::from_value(value).ok())
    }

    fn save(&self, observer: &Observer) -> AppResult<()> {
        let value = serde_json::to_value(observer).map_err(|e| AppError::Storage(e.to_string()))?;
        self.store.set(OBSERVER_KEY, value);
        self.store.save().map_err(|e| AppError::Storage(e.to_string()))
    }
}

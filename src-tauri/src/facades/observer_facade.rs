use std::sync::{Arc, PoisonError, RwLock};

use crate::astronomy::angles::normalize_signed_degrees;
use crate::dto::ObserverDto;
use crate::error::{AppError, AppResult};
use crate::facades::observer_storage::ObserverStorage;
use crate::mappers::ObserverMapper;
use crate::models::Observer;

const ALTITUDE_RANGE_M: std::ops::RangeInclusive<f64> = -500.0..=9000.0;
const MAX_LABEL_CHARS: usize = 32;

/// Position de l'observateur courant, partagée par les autres facades.
#[derive(Clone)]
pub struct ObserverFacade {
    storage: Arc<dyn ObserverStorage>,
    current: Arc<RwLock<Observer>>,
}

impl ObserverFacade {
    /// Reprend la position mémorisée, ou Paris au premier lancement.
    pub fn new(storage: Arc<dyn ObserverStorage>) -> Self {
        let current = storage.load().unwrap_or_else(Observer::paris);
        Self { storage, current: Arc::new(RwLock::new(current)) }
    }

    pub fn current(&self) -> Observer {
        self.current.read().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub fn get_observer(&self) -> ObserverDto {
        ObserverMapper::to_dto(&self.current())
    }

    pub fn set_observer(&self, lat: f64, lon: f64, altitude_m: f64, label: &str) -> AppResult<ObserverDto> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(AppError::InvalidInput(format!("latitude {lat} outside [-90, 90]")));
        }
        if !lon.is_finite() {
            return Err(AppError::InvalidInput(format!("longitude {lon} is not a number")));
        }
        if !ALTITUDE_RANGE_M.contains(&altitude_m) {
            return Err(AppError::InvalidInput(format!("altitude {altitude_m} m outside [-500, 9000]")));
        }
        let longitude_deg = normalize_signed_degrees(lon);
        let label = match label.trim() {
            "" => default_label(lat, longitude_deg),
            text => text.chars().take(MAX_LABEL_CHARS).collect(),
        };
        let observer = Observer { latitude_deg: lat, longitude_deg, altitude_m, label };
        self.storage.save(&observer)?;
        *self.current.write().unwrap_or_else(PoisonError::into_inner) = observer.clone();
        Ok(ObserverMapper::to_dto(&observer))
    }
}

/// « 48.86N 2.35E », pour un point choisi sur le globe.
fn default_label(lat: f64, lon: f64) -> String {
    let ns = if lat < 0.0 { 'S' } else { 'N' };
    let ew = if lon < 0.0 { 'O' } else { 'E' };
    format!("{:.2}{ns} {:.2}{ew}", lat.abs(), lon.abs())
}

#[cfg(test)]
pub mod tests {
    use std::sync::Mutex;

    use super::*;

    /// Stockage en mémoire, réutilisé par les tests des autres facades.
    #[derive(Default)]
    pub struct MemoryStorage(pub Mutex<Option<Observer>>);

    impl ObserverStorage for MemoryStorage {
        fn load(&self) -> Option<Observer> {
            self.0.lock().unwrap().clone()
        }

        fn save(&self, observer: &Observer) -> AppResult<()> {
            *self.0.lock().unwrap() = Some(observer.clone());
            Ok(())
        }
    }

    pub fn facade_at(observer: Observer) -> ObserverFacade {
        ObserverFacade::new(Arc::new(MemoryStorage(Mutex::new(Some(observer)))))
    }

    #[test]
    fn defaults_to_paris() {
        let facade = ObserverFacade::new(Arc::new(MemoryStorage::default()));
        assert_eq!(facade.get_observer().label, "Paris");
    }

    #[test]
    fn persists_and_normalizes() {
        let storage = Arc::new(MemoryStorage::default());
        let facade = ObserverFacade::new(storage.clone());
        let dto = facade.set_observer(-33.8688, 511.2093, 58.0, "  Sydney  ").unwrap();
        assert_eq!(dto.label, "Sydney");
        assert!((dto.longitude_deg - 151.2093).abs() < 1e-9);
        assert_eq!(storage.load().unwrap().label, "Sydney");
        // Une nouvelle facade relit la position mémorisée.
        assert_eq!(ObserverFacade::new(storage).current().label, "Sydney");
    }

    #[test]
    fn labels_picked_points() {
        let facade = ObserverFacade::new(Arc::new(MemoryStorage::default()));
        assert_eq!(facade.set_observer(-33.45, -70.66, 0.0, "").unwrap().label, "33.45S 70.66O");
    }

    #[test]
    fn rejects_invalid_coordinates() {
        let facade = ObserverFacade::new(Arc::new(MemoryStorage::default()));
        assert!(facade.set_observer(91.0, 0.0, 0.0, "x").is_err());
        assert!(facade.set_observer(0.0, f64::NAN, 0.0, "x").is_err());
        assert!(facade.set_observer(0.0, 0.0, 20_000.0, "x").is_err());
        assert_eq!(facade.current().label, "Paris");
    }
}

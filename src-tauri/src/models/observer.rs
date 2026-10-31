use serde::{Deserialize, Serialize};

use crate::astronomy::GeoSite;

/// Point d'observation, persisté entre deux lancements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observer {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub altitude_m: f64,
    pub label: String,
}

impl Observer {
    /// Position par défaut au premier lancement.
    pub fn paris() -> Self {
        Self {
            latitude_deg: 48.8566,
            longitude_deg: 2.3522,
            altitude_m: 35.0,
            label: "Paris".into(),
        }
    }

    pub fn site(&self) -> GeoSite {
        GeoSite {
            latitude_deg: self.latitude_deg,
            longitude_deg: self.longitude_deg,
            altitude_m: self.altitude_m,
        }
    }
}

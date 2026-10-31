pub mod celestial_body_mapper;
pub mod comet_catalog_mapper;
pub mod observer_mapper;
pub mod sky_mapper;
pub mod solar_mapper;
pub mod space_object_mapper;
pub mod telemetry_mapper;
pub mod trajectory_mapper;

pub use celestial_body_mapper::CelestialBodyMapper;
pub use comet_catalog_mapper::CometCatalogMapper;
pub use observer_mapper::ObserverMapper;
pub use sky_mapper::SkyMapper;
pub use solar_mapper::SolarMapper;
pub use space_object_mapper::SpaceObjectMapper;
pub use telemetry_mapper::TelemetryMapper;
pub use trajectory_mapper::TrajectoryMapper;

use chrono::{DateTime, SecondsFormat, Utc};

fn iso(instant: &DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Arrondi à 1e-4 près : 0.36″, bien en deçà de la précision des calculs,
/// et des réponses JSON deux fois plus légères.
fn round4(value: f64) -> f64 {
    (value * 1e4).round() / 1e4
}

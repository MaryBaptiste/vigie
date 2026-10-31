pub mod celestial_body_facade;
pub mod comet_catalog;
pub mod ephemeris;
pub mod observer_facade;
pub mod observer_storage;
pub mod orbit_math;
pub mod sky_facade;
pub mod solar_facade;
pub mod solar_math;
pub mod space_object_facade;
pub mod telemetry_facade;

pub use celestial_body_facade::CelestialBodyFacade;
pub use comet_catalog::CometCatalog;
pub use observer_facade::ObserverFacade;
pub use observer_storage::StoreObserverStorage;
pub use sky_facade::SkyFacade;
pub use solar_facade::SolarFacade;
pub use space_object_facade::SpaceObjectFacade;
pub use telemetry_facade::TelemetryFacade;

use chrono::{DateTime, Datelike, Utc};

use crate::error::{AppError, AppResult};

/// Années couvertes avec la précision annoncée (troncature VSOP87, ΔT).
const SUPPORTED_YEARS: std::ops::RangeInclusive<i32> = 1800..=2200;

/// Instant demandé par le front (ISO 8601), ou l'instant présent.
pub fn parse_instant(value: Option<&str>) -> AppResult<DateTime<Utc>> {
    let instant = match value {
        None => Utc::now(),
        Some(text) => DateTime::parse_from_rfc3339(text)
            .map_err(|e| AppError::InvalidInput(format!("datetime {text}: {e}")))?
            .with_timezone(&Utc),
    };
    if !SUPPORTED_YEARS.contains(&instant.year()) {
        return Err(AppError::InvalidInput(format!(
            "year {} outside the supported range {}-{}",
            instant.year(),
            SUPPORTED_YEARS.start(),
            SUPPORTED_YEARS.end()
        )));
    }
    Ok(instant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iso_instants() {
        let instant = parse_instant(Some("2026-10-30T21:44:12+01:00")).unwrap();
        assert_eq!(instant.to_rfc3339(), "2026-10-30T20:44:12+00:00");
        assert!(parse_instant(None).is_ok());
        assert!(matches!(parse_instant(Some("hier")), Err(AppError::InvalidInput(_))));
        assert!(matches!(parse_instant(Some("3026-01-01T00:00:00Z")), Err(AppError::InvalidInput(_))));
    }
}

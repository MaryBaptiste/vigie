use chrono::SecondsFormat;

use crate::dto::TelemetryDto;
use crate::models::OrbitalState;

pub struct TelemetryMapper;

impl TelemetryMapper {
    pub fn to_dto(state: &OrbitalState) -> TelemetryDto {
        TelemetryDto {
            norad_id: state.norad_id,
            timestamp: state.computed_at.to_rfc3339_opts(SecondsFormat::Millis, true),
            latitude_deg: state.position.latitude_deg,
            longitude_deg: state.position.longitude_deg,
            altitude_km: state.position.altitude_km,
            speed_km_s: state.speed_km_s,
            inclination_deg: state.inclination_deg,
            period_min: state.period_min,
            apogee_km: state.apogee_km,
            perigee_km: state.perigee_km,
            eccentricity: state.eccentricity,
            revolution_number: state.revolution_number,
            tle_epoch: state.tle_epoch.to_rfc3339_opts(SecondsFormat::Secs, true),
            tle_age_hours: state.tle_age_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::models::GeodeticPosition;

    fn sample_state() -> OrbitalState {
        OrbitalState {
            norad_id: 25544,
            computed_at: Utc.with_ymd_and_hms(2026, 10, 30, 12, 0, 0).unwrap(),
            position: GeodeticPosition {
                latitude_deg: 51.62,
                longitude_deg: -12.4,
                altitude_km: 418.0,
            },
            speed_km_s: 7.66,
            inclination_deg: 51.63,
            period_min: 92.9,
            apogee_km: 421.0,
            perigee_km: 412.0,
            eccentricity: 0.0007,
            revolution_number: 59270,
            tle_epoch: Utc.with_ymd_and_hms(2026, 10, 29, 13, 31, 42).unwrap(),
            tle_age_hours: 22.47,
        }
    }

    #[test]
    fn maps_numeric_fields() {
        let dto = TelemetryMapper::to_dto(&sample_state());
        assert_eq!(dto.norad_id, 25544);
        assert_eq!(dto.latitude_deg, 51.62);
        assert_eq!(dto.longitude_deg, -12.4);
        assert_eq!(dto.altitude_km, 418.0);
        assert_eq!(dto.speed_km_s, 7.66);
        assert_eq!(dto.revolution_number, 59270);
        assert_eq!(dto.tle_age_hours, 22.47);
    }

    #[test]
    fn formats_timestamps_as_utc_iso8601() {
        let dto = TelemetryMapper::to_dto(&sample_state());
        assert_eq!(dto.timestamp, "2026-10-30T12:00:00.000Z");
        assert_eq!(dto.tle_epoch, "2026-10-29T13:31:42Z");
    }

    #[test]
    fn serializes_in_camel_case() {
        let json = serde_json::to_value(TelemetryMapper::to_dto(&sample_state())).unwrap();
        for key in ["altitudeKm", "speedKmS", "tleEpoch", "tleAgeHours", "revolutionNumber"] {
            assert!(json.get(key).is_some(), "missing key {key}");
        }
    }
}

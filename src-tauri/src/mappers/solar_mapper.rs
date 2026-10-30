use chrono::SecondsFormat;

use crate::dto::{SolarDto, TerminatorPointDto};
use crate::models::{SolarState, TerminatorPoint};

pub struct SolarMapper;

impl SolarMapper {
    pub fn to_dto(state: &SolarState) -> SolarDto {
        SolarDto {
            timestamp: state.computed_at.to_rfc3339_opts(SecondsFormat::Millis, true),
            subsolar_latitude_deg: state.subsolar.latitude_deg,
            subsolar_longitude_deg: state.subsolar.longitude_deg,
            terminator: state.terminator.iter().map(Self::point_to_dto).collect(),
        }
    }

    fn point_to_dto(point: &TerminatorPoint) -> TerminatorPointDto {
        TerminatorPointDto {
            latitude_deg: point.point.latitude_deg,
            longitude_deg: point.point.longitude_deg,
            sunrise: point.sunrise,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::models::GeoPoint;

    fn sample() -> SolarState {
        SolarState {
            computed_at: Utc.with_ymd_and_hms(2026, 10, 30, 9, 0, 0).unwrap(),
            subsolar: GeoPoint {
                latitude_deg: -14.2,
                longitude_deg: 43.9,
            },
            terminator: vec![
                TerminatorPoint {
                    point: GeoPoint {
                        latitude_deg: 70.0,
                        longitude_deg: -180.0,
                    },
                    sunrise: true,
                },
                TerminatorPoint {
                    point: GeoPoint {
                        latitude_deg: 65.0,
                        longitude_deg: 179.0,
                    },
                    sunrise: false,
                },
            ],
        }
    }

    #[test]
    fn maps_subsolar_point_and_terminator() {
        let dto = SolarMapper::to_dto(&sample());
        assert_eq!(dto.timestamp, "2026-10-30T09:00:00.000Z");
        assert_eq!(dto.subsolar_latitude_deg, -14.2);
        assert_eq!(dto.subsolar_longitude_deg, 43.9);
        assert_eq!(dto.terminator.len(), 2);
        assert!(dto.terminator[0].sunrise);
        assert!(!dto.terminator[1].sunrise);
        assert_eq!(dto.terminator[1].longitude_deg, 179.0);
    }

    #[test]
    fn serializes_in_camel_case() {
        let json = serde_json::to_value(SolarMapper::to_dto(&sample())).unwrap();
        assert!(json.get("subsolarLatitudeDeg").is_some());
        assert!(json["terminator"][0].get("sunrise").is_some());
    }
}

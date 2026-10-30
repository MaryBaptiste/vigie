use chrono::SecondsFormat;

use crate::dto::{TrajectoryDto, TrajectoryPointDto};
use crate::models::{Trajectory, TrajectoryPoint};

pub struct TrajectoryMapper;

impl TrajectoryMapper {
    pub fn to_dto(trajectory: &Trajectory) -> TrajectoryDto {
        TrajectoryDto {
            norad_id: trajectory.norad_id,
            computed_at: trajectory
                .computed_at
                .to_rfc3339_opts(SecondsFormat::Millis, true),
            points: trajectory.points.iter().map(Self::point_to_dto).collect(),
        }
    }

    fn point_to_dto(point: &TrajectoryPoint) -> TrajectoryPointDto {
        TrajectoryPointDto {
            timestamp: point.time.to_rfc3339_opts(SecondsFormat::Secs, true),
            latitude_deg: point.position.latitude_deg,
            longitude_deg: point.position.longitude_deg,
            altitude_km: point.position.altitude_km,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, TimeZone, Utc};

    use super::*;
    use crate::models::GeodeticPosition;

    fn sample() -> Trajectory {
        let t0 = Utc.with_ymd_and_hms(2026, 10, 30, 12, 0, 0).unwrap();
        Trajectory {
            norad_id: 25544,
            computed_at: t0,
            points: (0..3)
                .map(|i| TrajectoryPoint {
                    time: t0 + TimeDelta::seconds(30 * i),
                    position: GeodeticPosition {
                        latitude_deg: i as f64,
                        longitude_deg: -(i as f64),
                        altitude_km: 418.0,
                    },
                })
                .collect(),
        }
    }

    #[test]
    fn maps_points_in_order() {
        let dto = TrajectoryMapper::to_dto(&sample());
        assert_eq!(dto.norad_id, 25544);
        assert_eq!(dto.computed_at, "2026-10-30T12:00:00.000Z");
        assert_eq!(dto.points.len(), 3);
        assert_eq!(dto.points[2].timestamp, "2026-10-30T12:01:00Z");
        assert_eq!(dto.points[2].latitude_deg, 2.0);
        assert_eq!(dto.points[2].longitude_deg, -2.0);
        assert_eq!(dto.points[2].altitude_km, 418.0);
    }

    #[test]
    fn serializes_in_camel_case() {
        let json = serde_json::to_value(TrajectoryMapper::to_dto(&sample())).unwrap();
        assert!(json.get("computedAt").is_some());
        assert!(json["points"][0].get("latitudeDeg").is_some());
    }
}

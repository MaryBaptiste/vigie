//! Position du Soleil (algorithme basse précision de l'Astronomical Almanac,
//! ~0,01°) et terminateur jour/nuit.

use chrono::{DateTime, Utc};

use crate::facades::orbit_math::{gmst_rad, julian_date};
use crate::models::{GeoPoint, TerminatorPoint};

const JD_J2000: f64 = 2_451_545.0;
/// Déclinaison minimale utilisée pour le terminateur : aux équinoxes (δ = 0)
/// la formule dégénère en deux méridiens verticaux.
const MIN_DECLINATION_RAD: f64 = 1e-6;

/// Point subsolaire (où le Soleil est au zénith).
pub fn subsolar_point(datetime: DateTime<Utc>) -> GeoPoint {
    let n = julian_date(datetime) - JD_J2000;
    let mean_longitude = (280.460 + 0.985_647_4 * n).to_radians();
    let mean_anomaly = (357.528 + 0.985_600_3 * n).to_radians();
    let ecliptic_longitude = mean_longitude
        + 1.915f64.to_radians() * mean_anomaly.sin()
        + 0.020f64.to_radians() * (2.0 * mean_anomaly).sin();
    let obliquity = (23.439 - 0.000_000_4 * n).to_radians();

    let (sin_lambda, cos_lambda) = ecliptic_longitude.sin_cos();
    let right_ascension = (obliquity.cos() * sin_lambda).atan2(cos_lambda);
    let declination = (obliquity.sin() * sin_lambda).asin();

    GeoPoint {
        latitude_deg: declination.to_degrees(),
        longitude_deg: normalize_deg((right_ascension - gmst_rad(datetime)).to_degrees()),
    }
}

/// Terminateur échantillonné tous les `step_deg` de longitude, de −180° à +180°.
pub fn terminator(subsolar: GeoPoint, step_deg: f64) -> Vec<TerminatorPoint> {
    let declination = subsolar.latitude_deg.to_radians();
    let declination = if declination.abs() < MIN_DECLINATION_RAD {
        MIN_DECLINATION_RAD.copysign(declination)
    } else {
        declination
    };
    let count = (360.0 / step_deg).round() as usize;

    (0..=count)
        .map(|i| {
            let longitude_deg = -180.0 + i as f64 * step_deg;
            // Angle horaire : négatif à l'ouest du point subsolaire, donc le matin.
            let hour_angle = normalize_deg(longitude_deg - subsolar.longitude_deg);
            let latitude = (-hour_angle.to_radians().cos() / declination.tan()).atan();
            TerminatorPoint {
                point: GeoPoint {
                    latitude_deg: latitude.to_degrees(),
                    longitude_deg,
                },
                sunrise: hour_angle < 0.0,
            }
        })
        .collect()
}

/// Ramène un angle dans ]−180°, 180°].
fn normalize_deg(angle: f64) -> f64 {
    let wrapped = (angle + 180.0).rem_euclid(360.0) - 180.0;
    if wrapped == -180.0 { 180.0 } else { wrapped }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn at(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }

    #[test]
    fn declination_follows_the_seasons() {
        assert!((subsolar_point(at(2026, 6, 21, 12)).latitude_deg - 23.44).abs() < 0.05);
        assert!((subsolar_point(at(2026, 12, 21, 12)).latitude_deg + 23.44).abs() < 0.05);
        assert!(subsolar_point(at(2026, 3, 20, 15)).latitude_deg.abs() < 0.1);
    }

    #[test]
    fn subsolar_longitude_tracks_utc_noon() {
        // À midi UTC le Soleil est près de Greenwich, à l'équation du temps près (< 4,5°).
        for month in 1..=12 {
            let lon = subsolar_point(at(2026, month, 15, 12)).longitude_deg;
            assert!(lon.abs() < 4.5, "month {month}: {lon}");
        }
        // Et il se déplace de 15° par heure vers l'ouest.
        let lon18 = subsolar_point(at(2026, 6, 15, 18)).longitude_deg;
        assert!((lon18 + 90.0).abs() < 4.5, "{lon18}");
    }

    #[test]
    fn terminator_is_90_degrees_from_subsolar_point() {
        let sun = subsolar_point(at(2026, 10, 30, 9));
        let (sin_d, cos_d) = sun.latitude_deg.to_radians().sin_cos();
        let points = terminator(sun, 1.0);
        assert_eq!(points.len(), 361);
        for p in points {
            let (sin_p, cos_p) = p.point.latitude_deg.to_radians().sin_cos();
            let h = (p.point.longitude_deg - sun.longitude_deg).to_radians();
            let cos_distance = sin_p * sin_d + cos_p * cos_d * h.cos();
            assert!(cos_distance.abs() < 1e-9, "{p:?}");
        }
    }

    #[test]
    fn sunrise_side_is_west_of_subsolar_point() {
        let sun = GeoPoint {
            latitude_deg: 10.0,
            longitude_deg: 0.0,
        };
        let points = terminator(sun, 1.0);
        let at_lon = |lon: f64| points.iter().find(|p| p.point.longitude_deg == lon).unwrap();
        assert!(at_lon(-90.0).sunrise);
        assert!(!at_lon(90.0).sunrise);
    }

    #[test]
    fn terminator_survives_equinox() {
        let sun = GeoPoint {
            latitude_deg: 0.0,
            longitude_deg: 0.0,
        };
        assert!(terminator(sun, 1.0).iter().all(|p| p.point.latitude_deg.is_finite()));
    }

    #[test]
    fn normalizes_degrees() {
        assert_eq!(normalize_deg(190.0), -170.0);
        assert_eq!(normalize_deg(-180.0), 180.0);
        assert_eq!(normalize_deg(540.0), 180.0);
    }
}

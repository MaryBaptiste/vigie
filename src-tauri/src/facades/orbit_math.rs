//! Calculs orbitaux purs : repères, géodésie WGS84, paramètres képlériens dérivés.

use std::f64::consts::{PI, TAU};

use chrono::{DateTime, Utc};

use crate::models::GeodeticPosition;

/// Rayon équatorial WGS84 (km).
pub const EARTH_EQUATORIAL_RADIUS_KM: f64 = 6378.137;
/// Aplatissement WGS84.
const WGS84_FLATTENING: f64 = 1.0 / 298.257_223_563;
/// Paramètre gravitationnel terrestre (km³/s²).
const EARTH_MU_KM3_S2: f64 = 398_600.441_8;

const MINUTES_PER_DAY: f64 = 1440.0;
const JD_UNIX_EPOCH: f64 = 2_440_587.5;
const JD_J2000: f64 = 2_451_545.0;

/// Paramètres de forme d'une orbite dérivés du mouvement moyen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitShape {
    pub period_min: f64,
    pub semi_major_axis_km: f64,
    pub apogee_km: f64,
    pub perigee_km: f64,
}

pub fn julian_date(datetime: DateTime<Utc>) -> f64 {
    datetime.timestamp_millis() as f64 / 86_400_000.0 + JD_UNIX_EPOCH
}

/// Temps sidéral moyen de Greenwich (IAU 1982), en radians dans [0, 2π[.
/// UTC est utilisé comme approximation de UT1 (écart < 0,9 s).
pub fn gmst_rad(datetime: DateTime<Utc>) -> f64 {
    let t = (julian_date(datetime) - JD_J2000) / 36_525.0;
    let seconds = 67_310.548_41 + (876_600.0 * 3600.0 + 8_640_184.812_866) * t
        + 0.093_104 * t * t
        - 6.2e-6 * t * t * t;
    // 1 s de temps sidéral = 1/240 de degré.
    (seconds.rem_euclid(86_400.0) / 240.0).to_radians()
}

/// Rotation TEME → ECEF (pseudo-fixe, mouvement du pôle négligé).
pub fn teme_to_ecef(teme: [f64; 3], gmst: f64) -> [f64; 3] {
    let (sin_g, cos_g) = gmst.sin_cos();
    [
        cos_g * teme[0] + sin_g * teme[1],
        -sin_g * teme[0] + cos_g * teme[1],
        teme[2],
    ]
}

/// Conversion ECEF (km) → géodésique WGS84, par itération sur la latitude.
pub fn ecef_to_geodetic(ecef: [f64; 3]) -> GeodeticPosition {
    let [x, y, z] = ecef;
    let a = EARTH_EQUATORIAL_RADIUS_KM;
    let e2 = WGS84_FLATTENING * (2.0 - WGS84_FLATTENING);
    let p = x.hypot(y);

    let mut lat = z.atan2(p * (1.0 - e2));
    for _ in 0..10 {
        let sin_lat = lat.sin();
        let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();
        let next = (z + e2 * n * sin_lat).atan2(p);
        let converged = (next - lat).abs() < 1e-12;
        lat = next;
        if converged {
            break;
        }
    }

    let (sin_lat, cos_lat) = lat.sin_cos();
    // Formule valable partout, y compris aux pôles (pas de division par cos φ).
    let altitude_km = p * cos_lat + z * sin_lat - a * (1.0 - e2 * sin_lat * sin_lat).sqrt();

    GeodeticPosition {
        latitude_deg: lat.to_degrees(),
        longitude_deg: normalize_longitude(y.atan2(x)).to_degrees(),
        altitude_km,
    }
}

pub fn teme_to_geodetic(teme: [f64; 3], datetime: DateTime<Utc>) -> GeodeticPosition {
    ecef_to_geodetic(teme_to_ecef(teme, gmst_rad(datetime)))
}

/// Forme de l'orbite à partir du mouvement moyen (rev/jour) et de l'excentricité.
pub fn orbit_shape(mean_motion_rev_per_day: f64, eccentricity: f64) -> OrbitShape {
    let period_min = MINUTES_PER_DAY / mean_motion_rev_per_day;
    let n_rad_s = mean_motion_rev_per_day * TAU / 86_400.0;
    let semi_major_axis_km = (EARTH_MU_KM3_S2 / (n_rad_s * n_rad_s)).cbrt();
    OrbitShape {
        period_min,
        semi_major_axis_km,
        apogee_km: semi_major_axis_km * (1.0 + eccentricity) - EARTH_EQUATORIAL_RADIUS_KM,
        perigee_km: semi_major_axis_km * (1.0 - eccentricity) - EARTH_EQUATORIAL_RADIUS_KM,
    }
}

/// Numéro de révolution courant, en extrapolant celui de l'époque du TLE.
/// Une révolution commence au nœud ascendant : la fraction déjà parcourue à
/// l'époque est approchée par l'argument de latitude moyen (ω + M).
pub fn revolution_number(
    rev_at_epoch: u64,
    argument_of_perigee_deg: f64,
    mean_anomaly_deg: f64,
    minutes_since_epoch: f64,
    period_min: f64,
) -> u64 {
    let fraction_at_epoch = ((argument_of_perigee_deg + mean_anomaly_deg) / 360.0).rem_euclid(1.0);
    let elapsed = (fraction_at_epoch + minutes_since_epoch / period_min).floor() as i64;
    rev_at_epoch.saturating_add_signed(elapsed)
}

pub fn norm(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize_longitude(lon: f64) -> f64 {
    (lon + PI).rem_euclid(TAU) - PI
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    const EPS_DEG: f64 = 1e-9;
    const EPS_KM: f64 = 1e-6;

    fn geodetic_to_ecef(lat_deg: f64, lon_deg: f64, alt_km: f64) -> [f64; 3] {
        let e2 = WGS84_FLATTENING * (2.0 - WGS84_FLATTENING);
        let (sin_lat, cos_lat) = lat_deg.to_radians().sin_cos();
        let (sin_lon, cos_lon) = lon_deg.to_radians().sin_cos();
        let n = EARTH_EQUATORIAL_RADIUS_KM / (1.0 - e2 * sin_lat * sin_lat).sqrt();
        [
            (n + alt_km) * cos_lat * cos_lon,
            (n + alt_km) * cos_lat * sin_lon,
            (n * (1.0 - e2) + alt_km) * sin_lat,
        ]
    }

    #[test]
    fn equator_prime_meridian_is_origin() {
        let pos = ecef_to_geodetic([EARTH_EQUATORIAL_RADIUS_KM, 0.0, 0.0]);
        assert!(pos.latitude_deg.abs() < EPS_DEG);
        assert!(pos.longitude_deg.abs() < EPS_DEG);
        assert!(pos.altitude_km.abs() < EPS_KM);
    }

    #[test]
    fn north_pole_has_zero_altitude_on_ellipsoid() {
        let polar_radius = EARTH_EQUATORIAL_RADIUS_KM * (1.0 - WGS84_FLATTENING);
        let pos = ecef_to_geodetic([0.0, 0.0, polar_radius + 400.0]);
        assert!((pos.latitude_deg - 90.0).abs() < EPS_DEG);
        assert!((pos.altitude_km - 400.0).abs() < EPS_KM);
    }

    #[test]
    fn geodetic_round_trip() {
        for &(lat, lon, alt) in &[
            (51.64, -12.4, 418.0),
            (-28.47, 179.9, 540.0),
            (98.2 - 180.0, -179.9, 705.0),
            (0.0, 90.0, 35_786.0),
        ] {
            let pos = ecef_to_geodetic(geodetic_to_ecef(lat, lon, alt));
            assert!((pos.latitude_deg - lat).abs() < EPS_DEG, "lat {lat}");
            assert!((pos.longitude_deg - lon).abs() < EPS_DEG, "lon {lon}");
            assert!((pos.altitude_km - alt).abs() < EPS_KM, "alt {alt}");
        }
    }

    #[test]
    fn gmst_at_j2000() {
        let j2000 = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        assert!((gmst_rad(j2000).to_degrees() - 280.460_618_37).abs() < 1e-6);
    }

    #[test]
    fn gmst_matches_vallado_example_3_5() {
        // Vallado, Fundamentals of Astrodynamics, ex. 3-5 : 1992-08-20 12:14 UT1.
        let dt = Utc.with_ymd_and_hms(1992, 8, 20, 12, 14, 0).unwrap();
        assert!((gmst_rad(dt).to_degrees() - 152.578_787_810).abs() < 1e-6);
    }

    #[test]
    fn teme_to_ecef_rotates_about_polar_axis() {
        let ecef = teme_to_ecef([1.0, 0.0, 2.0], 90f64.to_radians());
        assert!(ecef[0].abs() < 1e-12);
        assert!((ecef[1] + 1.0).abs() < 1e-12);
        assert_eq!(ecef[2], 2.0);
    }

    #[test]
    fn longitude_is_normalized() {
        let pos = ecef_to_geodetic([-EARTH_EQUATORIAL_RADIUS_KM, -1e-9, 0.0]);
        assert!(pos.longitude_deg >= -180.0 && pos.longitude_deg <= 180.0);
    }

    #[test]
    fn iss_like_orbit_shape() {
        let shape = orbit_shape(15.491_008_42, 0.000_704_97);
        assert!((shape.period_min - 92.956).abs() < 0.01);
        assert!(shape.perigee_km > 400.0 && shape.apogee_km < 430.0);
        assert!(shape.apogee_km > shape.perigee_km);
    }

    #[test]
    fn revolution_number_increments_per_period() {
        assert_eq!(revolution_number(100, 0.0, 0.0, 0.0, 90.0), 100);
        assert_eq!(revolution_number(100, 0.0, 0.0, 89.9, 90.0), 100);
        assert_eq!(revolution_number(100, 0.0, 0.0, 90.0, 90.0), 101);
        // À l'époque, l'objet est déjà aux 3/4 de sa révolution.
        assert_eq!(revolution_number(100, 200.0, 70.0, 30.0, 90.0), 101);
    }
}

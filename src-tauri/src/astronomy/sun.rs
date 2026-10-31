use std::f64::consts::PI;

use super::angles::normalize_degrees;
use super::coordinates::Equatorial;
use super::epoch::Epoch;
use super::linalg::{Vec3, scale};

/// Position géocentrique apparente du Soleil.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunPosition {
    pub equatorial: Equatorial,
    /// Longitude écliptique apparente (nutation et aberration incluses), en degrés.
    pub ecliptic_longitude_deg: f64,
    pub distance_au: f64,
    /// Vecteur géocentrique géométrique, écliptique moyen de la date (UA).
    pub geocentric_ecliptic: Vec3,
}

/// Soleil apparent, déduit de la position VSOP87 de la Terre (Meeus, ch. 25).
pub fn sun_position(epoch: &Epoch) -> SunPosition {
    let geocentric_ecliptic = scale(epoch.earth.to_vector(), -1.0);
    let distance_au = epoch.earth.distance_au;
    let aberration_deg = -20.4898 / 3600.0 / distance_au;
    let geometric_longitude = (epoch.earth.longitude_rad + PI).to_degrees();
    SunPosition {
        equatorial: epoch.apparent_from_ecliptic(geocentric_ecliptic, true),
        ecliptic_longitude_deg: normalize_degrees(
            geometric_longitude + epoch.nutation.longitude_deg + aberration_deg,
        ),
        distance_au,
        geocentric_ecliptic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::coordinates::angular_separation_deg;

    #[test]
    fn meeus_example_25b() {
        // 1992 octobre 13 0h TD : α = 13h13m30.749s, δ = −7°47′01.74″, λ = 199°54′21.818″.
        let sun = sun_position(&Epoch::from_terrestrial_time(2_448_908.5));
        let ra = 15.0 * (13.0 + 13.0 / 60.0 + 30.749 / 3600.0);
        let dec = -(7.0 + 47.0 / 60.0 + 1.74 / 3600.0);
        assert!((sun.equatorial.ra_deg - ra).abs() < 5e-4);
        assert!((sun.equatorial.dec_deg - dec).abs() < 5e-4);
        assert!((sun.ecliptic_longitude_deg - (199.0 + 54.0 / 60.0 + 21.818 / 3600.0)).abs() < 5e-4);
        assert!((sun.distance_au - 0.997_608).abs() < 1e-5);
    }

    #[test]
    fn matches_jpl_horizons() {
        // JPL Horizons, RA/Dec apparentes géocentriques.
        for (jd, ra, dec) in [
            (2_461_344.333_333_333, 215.059_105_800, -13.983_533_697),
            (2_455_197.5, 281.367_575_989, -23.027_292_536),
        ] {
            let sun = sun_position(&Epoch::from_julian_day(jd));
            let error = angular_separation_deg(sun.equatorial, Equatorial { ra_deg: ra, dec_deg: dec });
            assert!(error < 0.002, "écart {error}° à JD {jd}");
        }
    }
}

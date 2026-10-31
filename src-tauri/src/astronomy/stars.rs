use super::angles::ARCSEC;
use super::coordinates::{Equatorial, spherical_to_vector};
use super::epoch::Epoch;
use super::time::{DAYS_PER_YEAR, J2000};

/// Position catalogue d'une étoile : équateur et équinoxe J2000.0, époque J2000.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CatalogPosition {
    pub ra_deg: f64,
    pub dec_deg: f64,
    /// Mouvement propre en ascension droite, μα·cos δ (″/an).
    pub pm_ra_arcsec_per_year: f64,
    /// Mouvement propre en déclinaison (″/an).
    pub pm_dec_arcsec_per_year: f64,
}

/// Lieu apparent (Meeus, ch. 23) : mouvement propre linéaire, précession,
/// nutation et aberration annuelle. La parallaxe annuelle est négligée.
pub fn apparent_place(star: &CatalogPosition, epoch: &Epoch) -> Equatorial {
    let years = (epoch.jde - J2000) / DAYS_PER_YEAR;
    let dec = star.dec_deg + star.pm_dec_arcsec_per_year * years * ARCSEC;
    let cos_dec = star.dec_deg.to_radians().cos().max(1e-6);
    let ra = star.ra_deg + star.pm_ra_arcsec_per_year * years * ARCSEC / cos_dec;
    epoch.apparent_from_j2000(spherical_to_vector(ra, dec))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::coordinates::GeoSite;
    use crate::astronomy::time::julian_day;
    use chrono::{TimeZone, Utc};

    fn hms(h: f64, m: f64, s: f64) -> f64 {
        15.0 * (h + m / 60.0 + s / 3600.0)
    }

    fn dms(d: f64, m: f64, s: f64) -> f64 {
        d.signum() * (d.abs() + m / 60.0 + s / 3600.0)
    }

    #[test]
    fn meeus_example_23a() {
        // θ Persei, 2028 novembre 13.19 TD : α = 2h46m14.390s, δ = +49°21′07.45″.
        let theta_persei = CatalogPosition {
            ra_deg: hms(2.0, 44.0, 11.986),
            dec_deg: dms(49.0, 13.0, 42.48),
            pm_ra_arcsec_per_year: 0.034_25 * 15.0 * dms(49.0, 13.0, 42.48).to_radians().cos(),
            pm_dec_arcsec_per_year: -0.0895,
        };
        let place = apparent_place(&theta_persei, &Epoch::from_terrestrial_time(2_462_088.69));
        assert!((place.ra_deg - hms(2.0, 46.0, 14.390)).abs() < 5e-4);
        assert!((place.dec_deg - dms(49.0, 21.0, 7.45)).abs() < 5e-4);
    }

    /// Coordonnées BSC5 (J2000) et références Skyfield (DE421, sans réfraction).
    #[test]
    fn horizontal_positions_match_skyfield() {
        let paris = GeoSite { latitude_deg: 48.8566, longitude_deg: 2.3522, altitude_m: 0.0 };
        let sydney = GeoSite { latitude_deg: -33.8688, longitude_deg: 151.2093, altitude_m: 0.0 };
        let vega = CatalogPosition {
            ra_deg: hms(18.0, 36.0, 56.3),
            dec_deg: dms(38.0, 47.0, 1.0),
            pm_ra_arcsec_per_year: 0.201,
            pm_dec_arcsec_per_year: 0.287,
        };
        let polaris = CatalogPosition {
            ra_deg: hms(2.0, 31.0, 48.7),
            dec_deg: dms(89.0, 15.0, 51.0),
            pm_ra_arcsec_per_year: 0.038,
            pm_dec_arcsec_per_year: -0.015,
        };
        let acrux = CatalogPosition {
            ra_deg: hms(12.0, 26.0, 35.9),
            dec_deg: dms(-63.0, 5.0, 57.0),
            pm_ra_arcsec_per_year: -0.035,
            pm_dec_arcsec_per_year: -0.014,
        };
        let cases = [
            (&vega, &paris, (2026, 10, 30, 20), 45.4261, 281.2809),
            (&vega, &paris, (2026, 7, 15, 22), 76.1236, 131.4036),
            (&polaris, &paris, (2026, 10, 30, 20), 49.1098, 0.8687),
            (&acrux, &sydney, (2026, 4, 15, 12), 59.6564, 169.1854),
        ];
        for (star, site, (y, mo, d, h), alt, az) in cases {
            let instant = Utc.with_ymd_and_hms(y, mo, d, h, 0, 0).unwrap();
            let epoch = Epoch::from_julian_day(julian_day(&instant));
            let horizontal = epoch.horizontal(apparent_place(star, &epoch), site);
            assert!((horizontal.altitude_deg - alt).abs() < 0.01, "hauteur {horizontal:?}");
            let az_error = (horizontal.azimuth_deg - az).abs() * alt.to_radians().cos();
            assert!(az_error < 0.01, "azimut {horizontal:?}");
        }
    }
}

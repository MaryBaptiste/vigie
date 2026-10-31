use super::angles::{horner, normalize_degrees};
use super::coordinates::{Ecliptic, Equatorial};
use super::epoch::Epoch;
use super::moon_tables::{TABLE_B, TABLE_LR};
use super::sun::SunPosition;
use super::time::julian_centuries;

pub const KM_PER_AU: f64 = 149_597_870.7;

/// Position géocentrique apparente de la Lune.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoonPosition {
    pub equatorial: Equatorial,
    /// Longitude et latitude écliptiques apparentes (nutation incluse).
    pub ecliptic: Ecliptic,
    pub distance_km: f64,
}

impl MoonPosition {
    pub fn distance_au(&self) -> f64 {
        self.distance_km / KM_PER_AU
    }
}

/// Phase de la Lune.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoonPhase {
    /// Angle de phase Soleil–Lune–Terre, en degrés (0 = pleine lune).
    pub phase_angle_deg: f64,
    /// Fraction éclairée du disque, entre 0 et 1.
    pub illumination: f64,
    /// `true` entre la nouvelle et la pleine lune.
    pub waxing: bool,
    /// Âge en degrés : longitude de la Lune moins celle du Soleil, dans [0, 360[.
    pub elongation_deg: f64,
}

/// Coordonnées écliptiques géométriques moyennes de la date (λ, β en degrés,
/// Δ en km), Meeus ch. 47 : précision de l'ordre de 10″ en longitude.
pub fn geocentric_ecliptic(jde: f64) -> (f64, f64, f64) {
    let t = julian_centuries(jde);
    let l_prime = horner(t, &[218.316_447_7, 481_267.881_234_21, -0.001_578_6, 1.0 / 538_841.0, -1.0 / 65_194_000.0]);
    let d = horner(t, &[297.850_192_1, 445_267.111_403_4, -0.001_881_9, 1.0 / 545_868.0, -1.0 / 113_065_000.0]);
    let m = horner(t, &[357.529_109_2, 35_999.050_290_9, -0.000_153_6, 1.0 / 24_490_000.0]);
    let m_prime = horner(t, &[134.963_396_4, 477_198.867_505_5, 0.008_741_4, 1.0 / 69_699.0, -1.0 / 14_712_000.0]);
    let f = horner(t, &[93.272_095_0, 483_202.017_523_3, -0.003_653_9, -1.0 / 3_526_000.0, 1.0 / 863_310_000.0]);
    let a1 = 119.75 + 131.849 * t;
    let a2 = 53.09 + 479_264.290 * t;
    let a3 = 313.45 + 481_266.484 * t;
    let e = horner(t, &[1.0, -0.002_516, -0.000_007_4]);

    let [d, m, m_prime, f, l_prime, a1, a2, a3] =
        [d, m, m_prime, f, l_prime, a1, a2, a3].map(|angle| normalize_degrees(angle).to_radians());
    let eccentricity_factor = |m_multiple: i32| match m_multiple.abs() {
        0 => 1.0,
        1 => e,
        _ => e * e,
    };
    let argument = |row: &[i32]| {
        f64::from(row[0]) * d + f64::from(row[1]) * m + f64::from(row[2]) * m_prime + f64::from(row[3]) * f
    };

    let mut sum_l = 3958.0 * a1.sin() + 1962.0 * (l_prime - f).sin() + 318.0 * a2.sin();
    let mut sum_r = 0.0;
    for row in &TABLE_LR {
        let (sin_arg, cos_arg) = argument(row).sin_cos();
        let factor = eccentricity_factor(row[1]);
        sum_l += f64::from(row[4]) * factor * sin_arg;
        sum_r += f64::from(row[5]) * factor * cos_arg;
    }

    let mut sum_b = -2235.0 * l_prime.sin() + 382.0 * a3.sin() + 175.0 * (a1 - f).sin() + 175.0 * (a1 + f).sin()
        + 127.0 * (l_prime - m_prime).sin()
        - 115.0 * (l_prime + m_prime).sin();
    for row in &TABLE_B {
        sum_b += f64::from(row[4]) * eccentricity_factor(row[1]) * argument(row).sin();
    }

    (
        normalize_degrees(l_prime.to_degrees() + sum_l / 1e6),
        sum_b / 1e6,
        385_000.56 + sum_r / 1000.0,
    )
}

pub fn moon_position(epoch: &Epoch) -> MoonPosition {
    let (lon, lat, distance_km) = geocentric_ecliptic(epoch.jde);
    let mean = Ecliptic { lon_deg: lon, lat_deg: lat };
    MoonPosition {
        // L'aberration est déjà incluse dans la théorie : seule la nutation reste à ajouter.
        equatorial: epoch.apparent_from_ecliptic(mean.to_vector(), false),
        ecliptic: Ecliptic {
            lon_deg: normalize_degrees(lon + epoch.nutation.longitude_deg),
            lat_deg: lat,
        },
        distance_km,
    }
}

/// Phase et fraction éclairée (Meeus, ch. 48).
pub fn moon_phase(moon: &MoonPosition, sun: &SunPosition) -> MoonPhase {
    let elongation_deg = normalize_degrees(moon.ecliptic.lon_deg - sun.ecliptic_longitude_deg);
    let cos_psi = moon.ecliptic.lat_deg.to_radians().cos() * elongation_deg.to_radians().cos();
    let psi = cos_psi.clamp(-1.0, 1.0).acos();
    let sun_km = sun.distance_au * KM_PER_AU;
    let phase_angle = (sun_km * psi.sin()).atan2(moon.distance_km - sun_km * cos_psi);
    MoonPhase {
        phase_angle_deg: phase_angle.to_degrees(),
        illumination: (1.0 + phase_angle.cos()) / 2.0,
        waxing: elongation_deg < 180.0,
        elongation_deg,
    }
}

/// Magnitude visuelle approchée en fonction de l'angle de phase (Allen).
pub fn moon_magnitude(phase_angle_deg: f64) -> f64 {
    let i = phase_angle_deg.abs();
    -12.73 + 0.026 * i + 4e-9 * i.powi(4)
}

/// Demi-diamètre apparent, en degrés (Meeus, ch. 55).
pub fn moon_semidiameter_deg(distance_km: f64) -> f64 {
    358_473_400.0 / distance_km / 3600.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::coordinates::angular_separation_deg;
    use crate::astronomy::sun::sun_position;

    #[test]
    fn meeus_example_47a() {
        // 1992 avril 12 0h TD : λ = 133.162655°, β = −3.229126°, Δ = 368409.7 km,
        // α = 134.688470°, δ = 13.768368°.
        let (lon, lat, distance) = geocentric_ecliptic(2_448_724.5);
        assert!((lon - 133.162_655).abs() < 1e-5);
        assert!((lat + 3.229_126).abs() < 1e-5);
        assert!((distance - 368_409.7).abs() < 0.1);

        let moon = moon_position(&Epoch::from_terrestrial_time(2_448_724.5));
        assert!((moon.equatorial.ra_deg - 134.688_470).abs() < 5e-4);
        assert!((moon.equatorial.dec_deg - 13.768_368).abs() < 5e-4);
    }

    #[test]
    fn meeus_example_48a() {
        // 1992 avril 12 0h TD : fraction éclairée k = 0.6786, Lune croissante.
        let epoch = Epoch::from_terrestrial_time(2_448_724.5);
        let phase = moon_phase(&moon_position(&epoch), &sun_position(&epoch));
        assert!((phase.illumination - 0.6786).abs() < 0.001);
        assert!(phase.waxing);
    }

    #[test]
    fn matches_jpl_horizons() {
        // JPL Horizons, RA/Dec apparentes géocentriques.
        for (jd, ra, dec) in [
            (2_461_344.333_333_333, 102.091_471_883, 26.632_364_696),
            (2_455_197.5, 104.465_941_882, 23.499_328_300),
        ] {
            let moon = moon_position(&Epoch::from_julian_day(jd));
            let error = angular_separation_deg(moon.equatorial, Equatorial { ra_deg: ra, dec_deg: dec });
            assert!(error < 0.01, "écart {error}° à JD {jd}");
        }
    }

    #[test]
    fn full_moon_is_bright() {
        assert!((moon_magnitude(0.0) + 12.73).abs() < 1e-9);
        assert!(moon_magnitude(90.0) > -10.5);
    }
}

use super::angles::normalize_signed_degrees;
use super::coordinates::{Equatorial, vector_to_spherical};
use super::epoch::Epoch;
use super::linalg::{Vec3, norm, sub};
use super::time::julian_centuries;
use super::vsop87::{self, Heliocentric};

/// Temps de lumière pour 1 UA, en jours.
pub const LIGHT_TIME_DAYS_PER_AU: f64 = 0.005_775_518_3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Planet {
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

impl Planet {
    pub const ALL: [Planet; 7] = [
        Planet::Mercury,
        Planet::Venus,
        Planet::Mars,
        Planet::Jupiter,
        Planet::Saturn,
        Planet::Uranus,
        Planet::Neptune,
    ];
}

/// Position géocentrique apparente d'une planète et grandeurs photométriques.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetPosition {
    pub equatorial: Equatorial,
    /// Distance à la Terre Δ (UA).
    pub distance_au: f64,
    /// Distance au Soleil r (UA).
    pub heliocentric_distance_au: f64,
    /// Angle de phase Soleil–planète–Terre, en degrés.
    pub phase_angle_deg: f64,
    /// Fraction éclairée du disque, entre 0 et 1.
    pub illumination: f64,
    pub magnitude: f64,
}

/// Position apparente (Meeus, ch. 33) : temps de lumière itéré, puis
/// aberration annuelle et nutation. Magnitudes de Meeus, ch. 41.
pub fn planet_position(planet: Planet, epoch: &Epoch) -> PlanetPosition {
    let earth = epoch.earth.to_vector();
    let mut helio = vsop87::planet(planet, epoch.jde);
    let mut geocentric = sub(helio.to_vector(), earth);
    for _ in 0..2 {
        let tau = norm(geocentric) * LIGHT_TIME_DAYS_PER_AU;
        helio = vsop87::planet(planet, epoch.jde - tau);
        geocentric = sub(helio.to_vector(), earth);
    }

    let r = helio.distance_au;
    let delta = norm(geocentric);
    let big_r = epoch.earth.distance_au;
    let cos_i = ((r * r + delta * delta - big_r * big_r) / (2.0 * r * delta)).clamp(-1.0, 1.0);
    let phase_angle_deg = cos_i.acos().to_degrees();

    PlanetPosition {
        equatorial: epoch.apparent_from_ecliptic(geocentric, true),
        distance_au: delta,
        heliocentric_distance_au: r,
        phase_angle_deg,
        illumination: (1.0 + cos_i) / 2.0,
        magnitude: magnitude(planet, r, delta, phase_angle_deg, &helio, geocentric, epoch.jde),
    }
}

fn magnitude(planet: Planet, r: f64, delta: f64, i: f64, helio: &Heliocentric, geocentric: Vec3, jde: f64) -> f64 {
    let distance_term = 5.0 * (r * delta).log10();
    distance_term
        + match planet {
            Planet::Mercury => -0.42 + 0.0380 * i - 0.000_273 * i * i + 0.000_002 * i * i * i,
            Planet::Venus => -4.40 + 0.0009 * i + 0.000_239 * i * i - 0.000_000_65 * i * i * i,
            Planet::Mars => -1.52 + 0.016 * i,
            Planet::Jupiter => -9.40 + 0.005 * i,
            Planet::Saturn => {
                let rings = saturn_rings(helio, geocentric, jde);
                let sin_b = rings.tilt_deg.to_radians().sin().abs();
                -8.88 + 0.044 * rings.delta_u_deg - 2.60 * sin_b + 1.25 * sin_b * sin_b
            }
            Planet::Uranus => -7.19,
            Planet::Neptune => -6.87,
        }
}

struct SaturnRings {
    /// B, inclinaison des anneaux vue de la Terre.
    tilt_deg: f64,
    /// |U1 − U2|, écart des longitudes saturnicentriques du Soleil et de la Terre.
    delta_u_deg: f64,
}

/// Géométrie des anneaux de Saturne (Meeus, ch. 45).
fn saturn_rings(helio: &Heliocentric, geocentric: Vec3, jde: f64) -> SaturnRings {
    let t = julian_centuries(jde);
    let incl = (28.075_216 - 0.012_998 * t + 0.000_004 * t * t).to_radians();
    let node = (169.508_470 + 1.394_681 * t + 0.000_412 * t * t).to_radians();
    let (lambda_deg, beta_deg) = vector_to_spherical(geocentric);
    let (lambda, beta) = (lambda_deg.to_radians(), beta_deg.to_radians());
    let (l, b) = (helio.longitude_rad, helio.latitude_rad);

    let tilt = (incl.sin() * beta.cos() * (lambda - node).sin() - incl.cos() * beta.sin()).asin();
    let u1 = (incl.sin() * b.sin() + incl.cos() * b.cos() * (l - node).sin()).atan2(b.cos() * (l - node).cos());
    let u2 = (incl.sin() * beta.sin() + incl.cos() * beta.cos() * (lambda - node).sin())
        .atan2(beta.cos() * (lambda - node).cos());
    SaturnRings {
        tilt_deg: tilt.to_degrees(),
        delta_u_deg: normalize_signed_degrees((u1 - u2).to_degrees()).abs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::coordinates::angular_separation_deg;

    #[test]
    fn meeus_example_33a() {
        // Vénus, 1992 décembre 20 0h TD : α = 21h04m41.454s, δ = −18°53′16.84″, Δ = 0.910947 UA.
        let venus = planet_position(Planet::Venus, &Epoch::from_terrestrial_time(2_448_976.5));
        let ra = 15.0 * (21.0 + 4.0 / 60.0 + 41.454 / 3600.0);
        let dec = -(18.0 + 53.0 / 60.0 + 16.84 / 3600.0);
        assert!((venus.equatorial.ra_deg - ra).abs() < 5e-4);
        assert!((venus.equatorial.dec_deg - dec).abs() < 5e-4);
        assert!((venus.distance_au - 0.910_947).abs() < 1e-5);
        // Exemple 41.a : magnitude −4.2, fraction éclairée 0.647.
        assert!((venus.magnitude + 4.2).abs() < 0.05);
        assert!((venus.illumination - 0.647).abs() < 0.002);
    }

    #[test]
    fn matches_jpl_horizons() {
        // JPL Horizons, RA/Dec apparentes géocentriques (Mercure → Neptune).
        let cases: [(f64, [(f64, f64); 7]); 2] = [
            (
                2_461_344.333_333_333,
                [
                    (224.853_899_644, -18.956_676_343),
                    (202.988_200_792, -15.138_063_584),
                    (140.976_291_682, 16.925_096_603),
                    (146.721_781_530, 14.108_212_325),
                    (9.657_128_840, 1.237_896_526),
                    (62.790_742_365, 20.930_264_969),
                    (2.497_233_737, -0.459_780_145),
                ],
            ),
            (
                2_455_197.5,
                [
                    (290.322_570_451, -20.473_370_715),
                    (278.574_109_504, -23.642_011_994),
                    (142.475_968_380, 18.752_303_996),
                    (328.921_272_776, -13.609_060_093),
                    (185.043_528_081, 0.309_192_616),
                    (353.950_889_748, -3.428_285_922),
                    (327.020_787_200, -13.722_099_317),
                ],
            ),
        ];
        for (jd, expected) in cases {
            let epoch = Epoch::from_julian_day(jd);
            for (planet, (ra, dec)) in Planet::ALL.into_iter().zip(expected) {
                let position = planet_position(planet, &epoch);
                let error = angular_separation_deg(position.equatorial, Equatorial { ra_deg: ra, dec_deg: dec });
                assert!(error < 0.005, "{planet:?} : écart {error}° à JD {jd}");
            }
        }
    }

    #[test]
    fn magnitudes_are_plausible() {
        let epoch = Epoch::from_julian_day(2_461_344.333_333_333);
        let expected = [(Planet::Jupiter, -2.9..=-1.9), (Planet::Saturn, -0.5..=1.5), (Planet::Neptune, 7.6..=8.0)];
        for (planet, range) in expected {
            let m = planet_position(planet, &epoch).magnitude;
            assert!(range.contains(&m), "{planet:?} : magnitude {m}");
        }
    }
}

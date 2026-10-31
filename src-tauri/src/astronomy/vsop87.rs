//! Positions héliocentriques VSOP87D (écliptique et équinoxe dynamiques de la
//! date). Les séries, tronquées, sont générées par `tools/build_catalogs.py`.

use std::f64::consts::TAU;

use super::linalg::Vec3;
use super::planets::Planet;
use super::time::J2000;
use super::vsop87_data as data;

/// Un terme A·cos(B + C·τ).
pub type Term = (f64, f64, f64);
/// Une variable (L, B ou R) : un polynôme en τ dont chaque coefficient est une série.
pub type Series = &'static [&'static [Term]];

const DAYS_PER_MILLENNIUM: f64 = 365_250.0;

/// Évalue Σ τ^α · Σ A·cos(B + C·τ).
pub fn evaluate(series: Series, tau: f64) -> f64 {
    series
        .iter()
        .rev()
        .fold(0.0, |acc, terms| acc * tau + terms.iter().map(|(a, b, c)| a * (b + c * tau).cos()).sum::<f64>())
}

/// Position héliocentrique sphérique.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heliocentric {
    pub longitude_rad: f64,
    pub latitude_rad: f64,
    pub distance_au: f64,
}

impl Heliocentric {
    /// Vecteur rectangulaire écliptique (UA).
    pub fn to_vector(self) -> Vec3 {
        let (sin_l, cos_l) = self.longitude_rad.sin_cos();
        let (sin_b, cos_b) = self.latitude_rad.sin_cos();
        [
            self.distance_au * cos_b * cos_l,
            self.distance_au * cos_b * sin_l,
            self.distance_au * sin_b,
        ]
    }
}

fn heliocentric(series: [Series; 3], jde: f64) -> Heliocentric {
    let tau = (jde - J2000) / DAYS_PER_MILLENNIUM;
    Heliocentric {
        longitude_rad: evaluate(series[0], tau).rem_euclid(TAU),
        latitude_rad: evaluate(series[1], tau),
        distance_au: evaluate(series[2], tau),
    }
}

pub fn earth(jde: f64) -> Heliocentric {
    heliocentric([data::EARTH_L, data::EARTH_B, data::EARTH_R], jde)
}

pub fn planet(planet: Planet, jde: f64) -> Heliocentric {
    let series = match planet {
        Planet::Mercury => [data::MERCURY_L, data::MERCURY_B, data::MERCURY_R],
        Planet::Venus => [data::VENUS_L, data::VENUS_B, data::VENUS_R],
        Planet::Mars => [data::MARS_L, data::MARS_B, data::MARS_R],
        Planet::Jupiter => [data::JUPITER_L, data::JUPITER_B, data::JUPITER_R],
        Planet::Saturn => [data::SATURN_L, data::SATURN_B, data::SATURN_R],
        Planet::Uranus => [data::URANUS_L, data::URANUS_B, data::URANUS_R],
        Planet::Neptune => [data::NEPTUNE_L, data::NEPTUNE_B, data::NEPTUNE_R],
    };
    heliocentric(series, jde)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meeus_example_32a() {
        // Vénus, 1992 décembre 20 0h TD : L = 26.11428°, B = −2.62070°, R = 0.724603 UA
        // (Meeus utilise des séries plus courtes, d'où la tolérance de 1″).
        let venus = planet(Planet::Venus, 2_448_976.5);
        assert!((venus.longitude_rad.to_degrees() - 26.114_28).abs() < 3e-4);
        assert!((venus.latitude_rad.to_degrees() + 2.620_70).abs() < 1e-4);
        assert!((venus.distance_au - 0.724_603).abs() < 3e-6);
    }

    #[test]
    fn meeus_example_25b() {
        // Terre, 1992 octobre 13 0h TD : L = −43.63484796 rad, B = −0.00000312 rad, R = 0.99760775 UA.
        let earth = earth(2_448_908.5);
        assert!((earth.longitude_rad - (-43.634_847_96_f64).rem_euclid(TAU)).abs() < 2e-6);
        assert!((earth.latitude_rad + 0.000_003_12).abs() < 1e-6);
        assert!((earth.distance_au - 0.997_607_75).abs() < 1e-6);
    }
}

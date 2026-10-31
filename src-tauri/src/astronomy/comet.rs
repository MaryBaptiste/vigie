//! Comètes : mouvement képlérien à deux corps à partir des éléments au
//! périhélie (format du Minor Planet Center), valable pour toute
//! excentricité grâce aux variables universelles (fonctions de Stumpff).

use std::f64::consts::TAU;

use super::coordinates::Equatorial;
use super::epoch::Epoch;
use super::linalg::{Mat3, Vec3, norm, sub};
use super::planets::LIGHT_TIME_DAYS_PER_AU;

/// Constante de Gauss k, et μ = k² en UA³/jour².
const GAUSS_K: f64 = 0.017_202_098_95;
const MU: f64 = GAUSS_K * GAUSS_K;
/// Obliquité de l'écliptique J2000.0 (repère des éléments MPC).
const OBLIQUITY_J2000_DEG: f64 = 23.439_291_1;

/// Éléments orbitaux au périhélie, écliptique et équinoxe J2000.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitalElements {
    /// Instant du passage au périhélie (jour julien TT).
    pub perihelion_jde: f64,
    pub perihelion_distance_au: f64,
    pub eccentricity: f64,
    pub argument_of_perihelion_deg: f64,
    pub ascending_node_deg: f64,
    pub inclination_deg: f64,
}

/// Position géocentrique apparente d'une comète.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CometPosition {
    pub equatorial: Equatorial,
    /// Distance à la Terre Δ (UA).
    pub distance_au: f64,
    /// Distance au Soleil r (UA).
    pub heliocentric_distance_au: f64,
}

/// Fonctions de Stumpff c0…c3 de l'argument x = β·s².
fn stumpff(x: f64) -> [f64; 4] {
    if x.abs() < 1e-3 {
        // Développements limités : évitent les pertes de précision près de x = 0.
        return [
            1.0 - x / 2.0 + x * x / 24.0 - x * x * x / 720.0,
            1.0 - x / 6.0 + x * x / 120.0 - x * x * x / 5040.0,
            0.5 - x / 24.0 + x * x / 720.0 - x * x * x / 40_320.0,
            1.0 / 6.0 - x / 120.0 + x * x / 5040.0 - x * x * x / 362_880.0,
        ];
    }
    if x > 0.0 {
        let s = x.sqrt();
        [s.cos(), s.sin() / s, (1.0 - s.cos()) / x, (s - s.sin()) / (x * s)]
    } else {
        let s = (-x).sqrt();
        [s.cosh(), s.sinh() / s, (s.cosh() - 1.0) / -x, (s.sinh() - s) / (-x * s)]
    }
}

/// Position héliocentrique, équateur moyen J2000 (UA).
pub fn heliocentric_j2000(elements: &OrbitalElements, jde: f64) -> Vec3 {
    let q = elements.perihelion_distance_au;
    let e = elements.eccentricity;
    // β = μ/a : positif pour une ellipse, nul pour une parabole, négatif pour une hyperbole.
    let beta = MU * (1.0 - e) / q;

    let mut dt = jde - elements.perihelion_jde;
    if beta > 0.0 {
        // Orbite fermée : on se ramène à moins d'une demi-période du périhélie.
        let semi_major_axis = MU / beta;
        let period = TAU * (semi_major_axis.powi(3) / MU).sqrt();
        dt -= (dt / period).round() * period;
    }

    // Équation de Kepler universelle f(s) = q·s·c1 + μ·s³·c3 − Δt, résolue par
    // la méthode de Laguerre-Conway, qui converge quelle que soit l'orbite.
    let mut s = if dt == 0.0 {
        0.0
    } else {
        let near = dt / q;
        let far = (6.0 * dt.abs() / MU).cbrt() * dt.signum();
        if near.abs() < far.abs() { near } else { far }
    };
    for _ in 0..50 {
        let c = stumpff(beta * s * s);
        let f = q * s * c[1] + MU * s * s * s * c[3] - dt;
        let df = q * c[0] + MU * s * s * c[2];
        let ddf = MU * e * s * c[1];
        let root = (16.0 * df * df - 20.0 * f * ddf).abs().sqrt();
        let step = 5.0 * f / (df + df.signum() * root);
        s -= step;
        if step.abs() < 1e-14 * (1.0 + s.abs()) {
            break;
        }
    }

    let c = stumpff(beta * s * s);
    let f_coefficient = 1.0 - MU * s * s * c[2] / q;
    let g_coefficient = dt - MU * s * s * s * c[3];
    let perihelion_speed = (MU * (1.0 + e) / q).sqrt();

    // Repère orbital (P vers le périhélie, Q dans le sens du mouvement) en écliptique J2000.
    let orbit_to_ecliptic = Mat3::rotation_z(elements.ascending_node_deg)
        .mul(&Mat3::rotation_x(elements.inclination_deg))
        .mul(&Mat3::rotation_z(elements.argument_of_perihelion_deg));
    let in_plane = [f_coefficient * q, g_coefficient * perihelion_speed, 0.0];
    Mat3::rotation_x(OBLIQUITY_J2000_DEG).apply(orbit_to_ecliptic.apply(in_plane))
}

pub fn comet_position(elements: &OrbitalElements, epoch: &Epoch) -> CometPosition {
    let earth = epoch.ecliptic_to_j2000(epoch.earth.to_vector());
    let mut helio = heliocentric_j2000(elements, epoch.jde);
    let mut geocentric = sub(helio, earth);
    for _ in 0..2 {
        let tau = norm(geocentric) * LIGHT_TIME_DAYS_PER_AU;
        helio = heliocentric_j2000(elements, epoch.jde - tau);
        geocentric = sub(helio, earth);
    }
    CometPosition {
        equatorial: epoch.apparent_from_j2000(geocentric),
        distance_au: norm(geocentric),
        heliocentric_distance_au: norm(helio),
    }
}

/// Magnitude totale m = H + 5·log Δ + K·log r.
pub fn comet_magnitude(absolute_magnitude: f64, k: f64, distance_au: f64, heliocentric_distance_au: f64) -> f64 {
    absolute_magnitude + 5.0 * distance_au.log10() + k * heliocentric_distance_au.log10()
}

/// Vitesse héliocentrique par différence finie centrée (UA/jour), pour les tests.
#[cfg(test)]
fn velocity(elements: &OrbitalElements, jde: f64) -> Vec3 {
    let h = 1e-3;
    super::linalg::scale(
        sub(heliocentric_j2000(elements, jde + h), heliocentric_j2000(elements, jde - h)),
        1.0 / (2.0 * h),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::coordinates::angular_separation_deg;
    use crate::astronomy::linalg::dot;

    fn elements(q: f64, e: f64) -> OrbitalElements {
        OrbitalElements {
            perihelion_jde: 2_461_000.5,
            perihelion_distance_au: q,
            eccentricity: e,
            argument_of_perihelion_deg: 120.0,
            ascending_node_deg: 40.0,
            inclination_deg: 70.0,
        }
    }

    #[test]
    fn satisfies_vis_viva_for_every_conic() {
        // v² = μ(2/r − 1/a), avec 1/a = (1 − e)/q (nul pour la parabole).
        for e in [0.2, 0.85, 0.999, 1.0, 1.0001, 1.5] {
            let el = elements(0.6, e);
            for dt in [-900.0, -30.0, 0.0, 12.5, 400.0, 4000.0] {
                let jde = el.perihelion_jde + dt;
                let r = norm(heliocentric_j2000(&el, jde));
                let v = velocity(&el, jde);
                let expected = MU * (2.0 / r - (1.0 - e) / el.perihelion_distance_au);
                assert!((dot(v, v) / expected - 1.0).abs() < 1e-5, "e = {e}, Δt = {dt}");
            }
        }
    }

    #[test]
    fn starts_at_perihelion() {
        let el = elements(0.6, 0.7);
        assert!((norm(heliocentric_j2000(&el, el.perihelion_jde)) - 0.6).abs() < 1e-12);
    }

    #[test]
    fn encke_matches_jpl_horizons() {
        // 2P/Encke, éléments MPC (époque 2026-10-30) ; Horizons au 2026-10-30 0h UT :
        // α = 355.223518°, δ = 15.343986°, r = 1.838487 UA, Δ = 0.949845 UA.
        let encke = OrbitalElements {
            perihelion_jde: 2_461_446.5 + 0.2274,
            perihelion_distance_au: 0.338_598,
            eccentricity: 0.847_321,
            argument_of_perihelion_deg: 187.2875,
            ascending_node_deg: 334.0199,
            inclination_deg: 11.3480,
        };
        let position = comet_position(&encke, &Epoch::from_julian_day(2_461_343.5));
        let reference = Equatorial { ra_deg: 355.223_518, dec_deg: 15.343_986 };
        let error = angular_separation_deg(position.equatorial, reference);
        assert!(error < 0.01, "écart {error}°");
        assert!((position.heliocentric_distance_au - 1.838_487).abs() < 1e-3);
        assert!((position.distance_au - 0.949_845).abs() < 1e-3);
    }

    #[test]
    fn magnitude_formula() {
        assert!((comet_magnitude(10.0, 10.0, 1.0, 1.0) - 10.0).abs() < 1e-12);
        assert!((comet_magnitude(10.0, 10.0, 0.1, 10.0) - 15.0).abs() < 1e-12);
    }
}

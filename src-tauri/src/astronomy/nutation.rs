use super::angles::{ARCSEC, horner, normalize_degrees};
use super::time::julian_centuries;

/// Nutation et obliquité de l'écliptique, en degrés.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nutation {
    /// Δψ, nutation en longitude.
    pub longitude_deg: f64,
    /// Δε, nutation en obliquité.
    pub obliquity_deg: f64,
    /// ε0, obliquité moyenne.
    pub mean_obliquity_deg: f64,
}

impl Nutation {
    /// ε = ε0 + Δε, obliquité vraie.
    pub fn true_obliquity_deg(&self) -> f64 {
        self.mean_obliquity_deg + self.obliquity_deg
    }
}

/// Obliquité moyenne de l'écliptique (Meeus, 22.2, formule de l'IAU).
pub fn mean_obliquity_deg(jde: f64) -> f64 {
    let t = julian_centuries(jde);
    horner(t, &[84_381.448, -46.8150, -0.00059, 0.001813]) * ARCSEC
}

/// Nutation à 0.5″ près (Meeus, ch. 22, termes principaux).
pub fn nutation(jde: f64) -> Nutation {
    let t = julian_centuries(jde);
    let omega = normalize_degrees(horner(t, &[125.04452, -1934.136261, 0.0020708, 1.0 / 450_000.0])).to_radians();
    let sun = normalize_degrees(280.4665 + 36000.7698 * t).to_radians();
    let moon = normalize_degrees(218.3165 + 481267.8813 * t).to_radians();

    let d_psi = -17.20 * omega.sin() - 1.32 * (2.0 * sun).sin() - 0.23 * (2.0 * moon).sin()
        + 0.21 * (2.0 * omega).sin();
    let d_eps = 9.20 * omega.cos() + 0.57 * (2.0 * sun).cos() + 0.10 * (2.0 * moon).cos()
        - 0.09 * (2.0 * omega).cos();

    Nutation {
        longitude_deg: d_psi * ARCSEC,
        obliquity_deg: d_eps * ARCSEC,
        mean_obliquity_deg: mean_obliquity_deg(jde),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meeus_example_22a() {
        // 1987 avril 10, 0h TD : Δψ = −3.788″, Δε = +9.443″, ε0 = 23°26′27.407″.
        let n = nutation(2_446_895.5);
        assert!((n.longitude_deg / ARCSEC + 3.788).abs() < 0.5);
        assert!((n.obliquity_deg / ARCSEC - 9.443).abs() < 0.2);
        let eps0 = 23.0 + 26.0 / 60.0 + 27.407 / 3600.0;
        assert!((n.mean_obliquity_deg - eps0).abs() < 0.001 * ARCSEC);
    }
}

use super::angles::{horner, normalize_degrees};
use super::nutation::Nutation;
use super::time::{J2000, julian_centuries};

/// Temps sidéral moyen de Greenwich, en degrés (Meeus, 12.4).
pub fn mean_sidereal_time_deg(jd: f64) -> f64 {
    let t = julian_centuries(jd);
    let theta = 280.460_618_37
        + 360.985_647_366_29 * (jd - J2000)
        + horner(t, &[0.0, 0.0, 0.000_387_933, -1.0 / 38_710_000.0]);
    normalize_degrees(theta)
}

/// Temps sidéral apparent de Greenwich : ajoute l'équation des équinoxes Δψ·cos ε.
pub fn apparent_sidereal_time_deg(jd: f64, nutation: &Nutation) -> f64 {
    let equation_of_equinoxes = nutation.longitude_deg * nutation.true_obliquity_deg().to_radians().cos();
    normalize_degrees(mean_sidereal_time_deg(jd) + equation_of_equinoxes)
}

/// Temps sidéral local, longitude comptée positivement vers l'est.
pub fn local_sidereal_time_deg(greenwich_deg: f64, longitude_east_deg: f64) -> f64 {
    normalize_degrees(greenwich_deg + longitude_east_deg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::nutation::nutation;
    use crate::astronomy::time::terrestrial_time;

    /// Angle horaire en secondes de temps (1 s = 15″ = 1/240 °).
    fn seconds(deg: f64) -> f64 {
        deg * 240.0
    }

    #[test]
    fn meeus_example_12a() {
        // 1987 avril 10, 0h UT : θ0 = 13h10m46.3668s.
        let expected = 13.0 * 3600.0 + 10.0 * 60.0 + 46.3668;
        assert!((seconds(mean_sidereal_time_deg(2_446_895.5)) - expected).abs() < 0.001);
    }

    #[test]
    fn meeus_example_12b() {
        // 1987 avril 10, 19h21m00s UT : θ0 = 8h34m57.0896s.
        let jd = 2_446_896.306_25;
        let expected = 8.0 * 3600.0 + 34.0 * 60.0 + 57.0896;
        assert!((seconds(mean_sidereal_time_deg(jd)) - expected).abs() < 0.001);
    }

    #[test]
    fn apparent_sidereal_time_within_one_second() {
        // Même instant, temps apparent : 8h34m56.853s (Meeus 12.b).
        let jd = 2_446_896.306_25;
        let gast = apparent_sidereal_time_deg(jd, &nutation(terrestrial_time(jd)));
        let expected = 8.0 * 3600.0 + 34.0 * 60.0 + 56.853;
        assert!((seconds(gast) - expected).abs() < 0.05);
    }

    #[test]
    fn local_time_adds_east_longitude() {
        assert_eq!(local_sidereal_time_deg(350.0, 20.0), 10.0);
        assert_eq!(local_sidereal_time_deg(10.0, -77.0), 293.0);
    }
}

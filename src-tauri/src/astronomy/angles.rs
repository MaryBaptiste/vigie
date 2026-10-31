/// Une seconde d'arc, en degrés.
pub const ARCSEC: f64 = 1.0 / 3600.0;

/// Ramène un angle dans [0, 360[.
pub fn normalize_degrees(angle: f64) -> f64 {
    let wrapped = angle.rem_euclid(360.0);
    // rem_euclid peut renvoyer exactement 360 pour un très petit négatif.
    if wrapped >= 360.0 { 0.0 } else { wrapped }
}

/// Ramène un angle dans ]-180, 180].
pub fn normalize_signed_degrees(angle: f64) -> f64 {
    let wrapped = normalize_degrees(angle);
    if wrapped > 180.0 { wrapped - 360.0 } else { wrapped }
}

/// Évalue un polynôme `c[0] + c[1]·x + c[2]·x² + …` (schéma de Horner).
pub fn horner(x: f64, coefficients: &[f64]) -> f64 {
    coefficients.iter().rev().fold(0.0, |acc, c| acc * x + c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_into_range() {
        assert_eq!(normalize_degrees(-30.0), 330.0);
        assert_eq!(normalize_degrees(725.0), 5.0);
        assert_eq!(normalize_signed_degrees(270.0), -90.0);
        assert_eq!(normalize_signed_degrees(180.0), 180.0);
    }

    #[test]
    fn evaluates_polynomial() {
        assert_eq!(horner(2.0, &[1.0, 2.0, 3.0]), 17.0);
    }
}

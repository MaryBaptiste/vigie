use super::angles::ARCSEC;
use super::coordinates::Equatorial;
use super::linalg::Mat3;
use super::time::{DAYS_PER_CENTURY, J2000};

/// Matrice de précession (équateur et équinoxe moyens) d'une époque à une
/// autre, avec les angles ζ, z, θ de Lieske (Meeus, 21.2). Les époques sont
/// des jours juliens TT.
pub fn precession_matrix(from_jde: f64, to_jde: f64) -> Mat3 {
    let big_t = (from_jde - J2000) / DAYS_PER_CENTURY;
    let t = (to_jde - from_jde) / DAYS_PER_CENTURY;
    let base = 2306.2181 + 1.39656 * big_t - 0.000139 * big_t * big_t;
    let zeta = base * t + (0.30188 - 0.000344 * big_t) * t * t + 0.017998 * t * t * t;
    let z = base * t + (1.09468 + 0.000066 * big_t) * t * t + 0.018203 * t * t * t;
    let theta = (2004.3109 - 0.85330 * big_t - 0.000217 * big_t * big_t) * t
        - (0.42665 + 0.000217 * big_t) * t * t
        - 0.041833 * t * t * t;

    Mat3::rotation_z(z * ARCSEC)
        .mul(&Mat3::rotation_y(-theta * ARCSEC))
        .mul(&Mat3::rotation_z(zeta * ARCSEC))
}

pub fn precess(position: Equatorial, from_jde: f64, to_jde: f64) -> Equatorial {
    Equatorial::from_vector(precession_matrix(from_jde, to_jde).apply(position.to_vector()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meeus_example_21b() {
        // θ Persei, J2000.0 → 2028 novembre 13.19 TD, mouvement propre inclus :
        // α = 2h46m11.331s, δ = +49°20′54.54″.
        let years = (2_462_088.69 - J2000) / 365.25;
        let start = Equatorial {
            ra_deg: 41.049_942 + 0.034_25 * 15.0 * ARCSEC * years,
            dec_deg: 49.228_467 - 0.0895 * ARCSEC * years,
        };
        let result = precess(start, J2000, 2_462_088.69);
        assert!((result.ra_deg - 41.547_214).abs() < 1e-5);
        assert!((result.dec_deg - 49.348_483).abs() < 1e-5);
    }

    #[test]
    fn round_trip_is_identity() {
        let p = Equatorial { ra_deg: 279.23, dec_deg: 38.78 };
        let back = precess(precess(p, J2000, 2_405_889.258_55), 2_405_889.258_55, J2000);
        assert!((back.ra_deg - p.ra_deg).abs() < 1e-6);
        assert!((back.dec_deg - p.dec_deg).abs() < 1e-6);
    }
}
